use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use surrogate_app::{ProxyManager, ProxyStatus};
use surrogate_contract::events::Event;

#[derive(Clone)]
pub struct AppController {
    inner: Arc<AppControllerInner>,
}

struct AppControllerInner {
    manager: ProxyManager,
    rt: tokio::runtime::Runtime,
    recent_events: Arc<Mutex<VecDeque<Event>>>,
    ui_mode: Mutex<UiMode>,
    event_forwarder_started: Mutex<bool>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UiMode {
    #[default]
    Simple,
    Advanced,
    Expert,
}

impl AppController {
    pub fn new(config_path: PathBuf, rt: tokio::runtime::Runtime) -> Self {
        let manager = ProxyManager::new(config_path);
        Self {
            inner: Arc::new(AppControllerInner {
                manager,
                rt,
                recent_events: Arc::new(Mutex::new(VecDeque::new())),
                ui_mode: Mutex::new(UiMode::default()),
                event_forwarder_started: Mutex::new(false),
            }),
        }
    }

    pub fn auto_start_proxy(&self) {
        self.ensure_event_forwarder();
        let manager = &self.inner.manager;
        match self.inner.rt.block_on(manager.start()) {
            Ok(addr) => {
                eprintln!("[surrogate] proxy started on {addr}");
            }
            Err(e) => {
                eprintln!("[surrogate] auto-start failed: {e}");
            }
        }
    }

    pub fn start_proxy(&self) -> Result<String, String> {
        self.ensure_event_forwarder();
        self.inner
            .rt
            .block_on(self.inner.manager.start())
            .map(|addr| addr.to_string())
            .map_err(|e| e.to_string())
    }

    pub fn stop_proxy(&self) -> Result<(), String> {
        self.inner
            .rt
            .block_on(self.inner.manager.stop())
            .map_err(|e| e.to_string())
    }

    pub fn is_running(&self) -> bool {
        self.inner.rt.block_on(self.inner.manager.is_running())
    }

    pub fn status(&self) -> ProxyStatus {
        self.inner.rt.block_on(self.inner.manager.status())
    }

    pub fn toggle_system_proxy(&self, enable: bool) -> Result<bool, String> {
        if enable {
            let status = self.status();
            let addr = status.listen_addr.ok_or("proxy is not running")?;
            let (host, port_str) = addr
                .rsplit_once(':')
                .ok_or_else(|| format!("invalid listen address: {addr}"))?;
            let port: u16 = port_str
                .parse()
                .map_err(|_| format!("invalid port: {port_str}"))?;
            surrogate_app::system_proxy::enable_all_proxies(host, port)
                .map_err(|e| e.to_string())?;
        } else {
            surrogate_app::system_proxy::disable_all_proxies().map_err(|e| e.to_string())?;
        }
        Ok(enable)
    }

    pub fn is_system_proxy_enabled(&self) -> bool {
        surrogate_app::system_proxy::is_proxy_enabled().unwrap_or(false)
    }

    pub fn get_config_content(&self) -> Result<String, String> {
        let status = self.status();
        std::fs::read_to_string(&status.config_path).map_err(|e| e.to_string())
    }

    pub fn save_config_content(&self, content: &str) -> Result<(), String> {
        let status = self.status();
        let path = PathBuf::from(&status.config_path);
        let doc: surrogate_contract::config::ConfigDocument =
            toml::from_str(content).map_err(|e| format!("invalid TOML: {e}"))?;
        surrogate_contract::config::validate(&doc)
            .map_err(|e| format!("validation error: {e}"))?;
        std::fs::write(&path, content).map_err(|e| format!("write error: {e}"))?;
        Ok(())
    }

    pub fn export_command(&self) -> Option<String> {
        let status = self.status();
        status.listen_addr.map(|addr| {
            format!(
                "export https_proxy=http://{addr} http_proxy=http://{addr} all_proxy=socks5://{addr}"
            )
        })
    }

    pub fn recent_events(&self) -> Vec<Event> {
        let guard = self
            .inner
            .recent_events
            .lock()
            .expect("recent_events lock should not be poisoned");
        guard.iter().rev().take(50).cloned().collect()
    }

    pub fn ui_mode(&self) -> UiMode {
        *self
            .inner
            .ui_mode
            .lock()
            .expect("ui_mode lock should not be poisoned")
    }

    pub fn set_ui_mode(&self, mode: UiMode) {
        *self
            .inner
            .ui_mode
            .lock()
            .expect("ui_mode lock should not be poisoned") = mode;
    }

    pub fn cleanup_and_exit(&self) {
        if self.is_running() {
            let _ = surrogate_app::system_proxy::disable_all_proxies();
            let _ = self.stop_proxy();
        }
    }

    fn ensure_event_forwarder(&self) {
        let mut started = self
            .inner
            .event_forwarder_started
            .lock()
            .expect("event_forwarder_started lock should not be poisoned");
        if *started {
            return;
        }
        *started = true;

        let mut rx = self.inner.manager.subscribe_events();
        let events = Arc::clone(&self.inner.recent_events);
        self.inner.rt.spawn(async move {
            while let Ok(event) = rx.recv().await {
                let mut guard = events
                    .lock()
                    .expect("recent_events lock should not be poisoned");
                guard.push_back(event);
                while guard.len() > 200 {
                    guard.pop_front();
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_mode_default_is_simple() {
        assert_eq!(UiMode::default(), UiMode::Simple);
    }

    #[test]
    fn ui_mode_roundtrip_serde() {
        let mode = UiMode::Expert;
        let json = serde_json::to_string(&mode).expect("serialize");
        let back: UiMode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, UiMode::Expert);
    }
}
