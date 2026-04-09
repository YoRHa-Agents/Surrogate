use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use surrogate_contract::config::{load_and_validate, normalize};
use surrogate_contract::events::{Event, EventSink, Observability};
use surrogate_kernel::{Kernel, RunningKernel};
use tokio::sync::{Mutex, broadcast};

const EVENT_CHANNEL_CAPACITY: usize = 4096;

pub struct ProxyManager {
    config_path: Mutex<PathBuf>,
    inner: Mutex<ProxyInner>,
    event_tx: broadcast::Sender<Event>,
    total_events: Arc<AtomicU64>,
}

struct ProxyInner {
    kernel: Option<RunningKernel>,
    listen_addr: Option<SocketAddr>,
    start_time: Option<Instant>,
    config_summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub listen_addr: Option<String>,
    pub uptime_secs: Option<u64>,
    pub config_path: String,
    pub config_summary: String,
    pub total_events: u64,
}

struct BroadcastEventSink {
    tx: broadcast::Sender<Event>,
    counter: Arc<AtomicU64>,
}

impl EventSink for BroadcastEventSink {
    fn emit(&self, event: Event) {
        self.counter.fetch_add(1, Ordering::Relaxed);
        let _ = self.tx.send(event);
    }
}

impl ProxyManager {
    pub fn new(config_path: PathBuf) -> Self {
        let (tx, _rx) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self {
            config_path: Mutex::new(config_path),
            inner: Mutex::new(ProxyInner {
                kernel: None,
                listen_addr: None,
                start_time: None,
                config_summary: String::new(),
            }),
            event_tx: tx,
            total_events: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn start(&self) -> Result<SocketAddr> {
        let mut inner = self.inner.lock().await;
        if inner.kernel.is_some() {
            bail!("proxy is already running");
        }

        let config_path = self.config_path.lock().await.clone();
        let document = load_and_validate(&config_path)
            .with_context(|| format!("failed to load config at `{}`", config_path.display()))?;
        let normalized = normalize(&document);

        let config_summary = format!(
            "Default Outbound: {}\nOutbounds: {}\nRules: {}",
            normalized.default_outbound,
            normalized
                .outbounds
                .iter()
                .map(|o| o.id.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            normalized.rules.len()
        );

        let sink = BroadcastEventSink {
            tx: self.event_tx.clone(),
            counter: self.total_events.clone(),
        };
        let observability = Observability::new(Arc::new(sink));

        let kernel =
            Kernel::new(normalized, observability).context("failed to build kernel from config")?;
        let running = kernel
            .spawn()
            .await
            .context("failed to start proxy listener")?;
        let addr = running.local_addr();

        inner.kernel = Some(running);
        inner.listen_addr = Some(addr);
        inner.start_time = Some(Instant::now());
        inner.config_summary = config_summary;

        Ok(addr)
    }

    pub async fn stop(&self) -> Result<()> {
        let mut inner = self.inner.lock().await;
        match inner.kernel.take() {
            Some(kernel) => {
                kernel.shutdown().await.context("kernel shutdown failed")?;
                inner.listen_addr = None;
                inner.start_time = None;
                inner.config_summary.clear();
                Ok(())
            }
            None => bail!("proxy is not running"),
        }
    }

    pub async fn restart(&self) -> Result<SocketAddr> {
        if self.is_running().await {
            self.stop().await?;
        }
        self.start().await
    }

    pub async fn is_running(&self) -> bool {
        self.inner.lock().await.kernel.is_some()
    }

    pub async fn status(&self) -> ProxyStatus {
        let inner = self.inner.lock().await;
        let config_path = self.config_path.lock().await.display().to_string();
        ProxyStatus {
            running: inner.kernel.is_some(),
            listen_addr: inner.listen_addr.map(|a| a.to_string()),
            uptime_secs: inner.start_time.map(|t| t.elapsed().as_secs()),
            config_path,
            config_summary: inner.config_summary.clone(),
            total_events: self.total_events.load(Ordering::Relaxed),
        }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<Event> {
        self.event_tx.subscribe()
    }

    pub async fn set_config_path(&self, path: PathBuf) {
        *self.config_path.lock().await = path;
    }

    pub fn total_events(&self) -> u64 {
        self.total_events.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn test_config_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{nanos}.toml"))
    }

    fn write_valid_config(path: &std::path::Path) {
        std::fs::write(
            path,
            r#"listen = "127.0.0.1:0"
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"
"#,
        )
        .expect("write test config");
    }

    #[tokio::test]
    async fn new_manager_is_stopped() {
        let path = test_config_path("pm-new");
        let manager = ProxyManager::new(path);
        assert!(!manager.is_running().await);
        let status = manager.status().await;
        assert!(!status.running);
        assert!(status.listen_addr.is_none());
    }

    #[tokio::test]
    async fn start_and_stop_lifecycle() {
        let path = test_config_path("pm-lifecycle");
        write_valid_config(&path);

        let manager = ProxyManager::new(path.clone());
        let addr = manager.start().await.expect("start should succeed");
        assert!(manager.is_running().await);

        let status = manager.status().await;
        assert!(status.running);
        assert_eq!(status.listen_addr, Some(addr.to_string()));

        manager.stop().await.expect("stop should succeed");
        assert!(!manager.is_running().await);

        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn double_start_fails() {
        let path = test_config_path("pm-double-start");
        write_valid_config(&path);

        let manager = ProxyManager::new(path.clone());
        manager.start().await.expect("first start succeeds");

        let err = manager.start().await.expect_err("second start should fail");
        assert!(err.to_string().contains("already running"));

        manager.stop().await.expect("stop succeeds");
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn stop_when_not_running_fails() {
        let path = test_config_path("pm-stop-idle");
        let manager = ProxyManager::new(path);
        let err = manager.stop().await.expect_err("stop should fail");
        assert!(err.to_string().contains("not running"));
    }

    #[tokio::test]
    async fn restart_from_running() {
        let path = test_config_path("pm-restart");
        write_valid_config(&path);

        let manager = ProxyManager::new(path.clone());
        let addr1 = manager.start().await.expect("start succeeds");
        let addr2 = manager.restart().await.expect("restart succeeds");
        assert_ne!(addr1, addr2, "restart should bind a new port (port 0)");
        assert!(manager.is_running().await);

        manager.stop().await.expect("stop succeeds");
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn restart_from_stopped() {
        let path = test_config_path("pm-restart-cold");
        write_valid_config(&path);

        let manager = ProxyManager::new(path.clone());
        let addr = manager.restart().await.expect("cold restart succeeds");
        assert!(manager.is_running().await);
        assert_eq!(
            manager.status().await.listen_addr,
            Some(addr.to_string())
        );

        manager.stop().await.expect("stop succeeds");
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn event_subscriber_receives_events() {
        let path = test_config_path("pm-events");
        write_valid_config(&path);

        let manager = ProxyManager::new(path.clone());
        let mut rx = manager.subscribe_events();

        manager.start().await.expect("start succeeds");

        let addr = manager.status().await.listen_addr.unwrap();
        let connect_request =
            "CONNECT example.com:443 HTTP/1.1\r\nHost: example.com:443\r\n\r\n";
        if let Ok(mut stream) = tokio::net::TcpStream::connect(&addr).await {
            use tokio::io::AsyncWriteExt;
            let _ = stream.write_all(connect_request.as_bytes()).await;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let mut received = Vec::new();
        while let Ok(event) = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            rx.recv(),
        )
        .await
        {
            if let Ok(event) = event {
                received.push(event);
            }
        }
        assert!(!received.is_empty(), "should receive at least one event");
        assert!(manager.total_events() > 0);

        manager.stop().await.expect("stop succeeds");
        let _ = std::fs::remove_file(path);
    }
}
