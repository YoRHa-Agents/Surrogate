use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use surrogate_app::{ProxyManager, ProxyStatus};
use surrogate_contract::events::Event;

#[derive(Debug, Clone)]
pub struct UiState {
    pub running: bool,
    pub listen_addr: Option<String>,
    pub uptime_secs: Option<u64>,
    pub system_proxy: bool,
    pub mode: UiMode,
    pub total_events: u64,
    pub session_count: usize,
    pub error_count: usize,
    pub config_summary: String,
    pub config_path: String,
    pub recent_events: Vec<Event>,
    pub default_outbound: String,
}

pub mod view_tags {
    pub const STATUS_PROXY: isize = 9001;
    pub const STATUS_UPTIME: isize = 9002;
    pub const STATUS_EVENTS: isize = 9003;
    pub const STATUS_ERRORS: isize = 9004;
    pub const STATUS_MODE: isize = 9005;
    pub const STATUS_SYS_PROXY: isize = 9006;
    pub const OVERVIEW_PROXY_STATE: isize = 9010;
    pub const OVERVIEW_TOTAL_EVENTS: isize = 9011;
    pub const OVERVIEW_SESSIONS: isize = 9012;
    pub const OVERVIEW_ERRORS: isize = 9013;
    pub const OVERVIEW_UPTIME: isize = 9014;
    pub const OVERVIEW_DEFAULT_EXIT: isize = 9015;
    pub const OVERVIEW_CONFIG: isize = 9016;
    pub const OBSERVE_EVENT_COUNT: isize = 9020;
}

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

    pub fn config_document(&self) -> Option<surrogate_contract::config::ConfigDocument> {
        let content = self.get_config_content().ok()?;
        toml::from_str(&content).ok()
    }

    pub fn outbounds(&self) -> Vec<surrogate_contract::config::OutboundConfig> {
        self.config_document()
            .map(|d| d.outbounds)
            .unwrap_or_default()
    }

    pub fn rules(&self) -> Vec<surrogate_contract::config::RouteRuleConfig> {
        self.config_document()
            .map(|d| d.rules)
            .unwrap_or_default()
    }

    pub fn default_outbound_id(&self) -> String {
        self.config_document()
            .map(|d| d.default_outbound)
            .unwrap_or_else(|| "direct".to_string())
    }

    pub fn snapshot(&self) -> UiState {
        let status = self.status();
        let (_, sessions, errors) = self.event_counts();
        let events = self.recent_events();
        UiState {
            running: status.running,
            listen_addr: status.listen_addr,
            uptime_secs: status.uptime_secs,
            system_proxy: self.is_system_proxy_enabled(),
            mode: self.ui_mode(),
            total_events: status.total_events,
            session_count: sessions,
            error_count: errors,
            config_summary: status.config_summary,
            config_path: status.config_path,
            recent_events: events,
            default_outbound: self.default_outbound_id(),
        }
    }

    pub fn event_counts(&self) -> (usize, usize, usize) {
        let events = self.recent_events();
        let sessions = events
            .iter()
            .filter(|e| e.kind == surrogate_contract::events::EventKind::SessionStarted)
            .count();
        let errors = events
            .iter()
            .filter(|e| e.kind == surrogate_contract::events::EventKind::Error)
            .count();
        (events.len(), sessions, errors)
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

    #[test]
    fn ui_state_clone() {
        let state = UiState {
            running: false,
            listen_addr: None,
            uptime_secs: None,
            system_proxy: false,
            mode: UiMode::Simple,
            total_events: 0,
            session_count: 0,
            error_count: 0,
            config_summary: String::new(),
            config_path: String::from("/tmp/config.toml"),
            recent_events: vec![],
            default_outbound: String::from("direct"),
        };
        let cloned = state.clone();
        assert_eq!(cloned.running, false);
        assert_eq!(cloned.default_outbound, "direct");
        assert_eq!(cloned.mode, UiMode::Simple);
    }

    #[test]
    fn view_tags_unique() {
        let tags = [
            view_tags::STATUS_PROXY,
            view_tags::STATUS_UPTIME,
            view_tags::STATUS_EVENTS,
            view_tags::STATUS_ERRORS,
            view_tags::STATUS_MODE,
            view_tags::STATUS_SYS_PROXY,
            view_tags::OVERVIEW_PROXY_STATE,
            view_tags::OVERVIEW_TOTAL_EVENTS,
            view_tags::OVERVIEW_SESSIONS,
            view_tags::OVERVIEW_ERRORS,
            view_tags::OVERVIEW_UPTIME,
            view_tags::OVERVIEW_DEFAULT_EXIT,
            view_tags::OVERVIEW_CONFIG,
            view_tags::OBSERVE_EVENT_COUNT,
        ];
        let mut seen = std::collections::HashSet::new();
        for tag in &tags {
            assert!(seen.insert(tag), "duplicate view tag: {tag}");
        }
        assert_eq!(seen.len(), 14);
    }

    #[test]
    fn ui_state_debug_format() {
        let state = UiState {
            running: true,
            listen_addr: Some("127.0.0.1:41080".to_string()),
            uptime_secs: Some(60),
            system_proxy: false,
            mode: UiMode::Advanced,
            total_events: 42,
            session_count: 10,
            error_count: 2,
            config_summary: "2 outbounds, 1 rule".to_string(),
            config_path: "/tmp/c.toml".to_string(),
            recent_events: vec![],
            default_outbound: "direct".to_string(),
        };
        let dbg = format!("{state:?}");
        assert!(dbg.contains("running: true"));
        assert!(dbg.contains("Advanced"));
    }
}
