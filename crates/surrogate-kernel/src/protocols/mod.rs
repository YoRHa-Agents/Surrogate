pub mod shadowsocks;
pub mod trojan;
pub mod vless;
pub mod vmess;
pub mod wireguard;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Boxed future returned by [`ProtocolHandler::connect`].
pub type ConnectResult<'a> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    Box<dyn surrogate_contract::transport::TransportStream>,
                    ProtocolError,
                >,
            > + Send
            + 'a,
    >,
>;

/// Protocol handler trait. Each protocol module implements this.
pub trait ProtocolHandler: Send + Sync {
    /// Protocol identifier (e.g., "shadowsocks", "vless").
    fn name(&self) -> &str;

    /// Protocol specification reference.
    fn spec_reference(&self) -> &str;

    /// Clean-room risk level.
    fn clean_room_risk(&self) -> CleanRoomRisk;

    /// Whether this protocol is experimental.
    fn is_experimental(&self) -> bool {
        false
    }

    /// Establish an outbound connection through this protocol.
    fn connect(&self, target: &str, config: &ProtocolConfig) -> ConnectResult<'_>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanRoomRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    pub server: String,
    pub port: u16,
    pub password: Option<String>,
    pub method: Option<String>,
    pub uuid: Option<String>,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("authentication failed: {0}")]
    AuthFailed(String),
    #[error("protocol error: {0}")]
    ProtocolViolation(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Registry of available protocol handlers.
pub struct ProtocolRegistry {
    handlers: Vec<Box<dyn ProtocolHandler>>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn register(&mut self, handler: Box<dyn ProtocolHandler>) {
        self.handlers.push(handler);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ProtocolHandler> {
        self.handlers
            .iter()
            .find(|h| h.name() == name)
            .map(|h| h.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.handlers.iter().map(|h| h.name()).collect()
    }
}

impl Default for ProtocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry() -> ProtocolRegistry {
        let mut reg = ProtocolRegistry::new();
        reg.register(Box::new(shadowsocks::ShadowsocksHandler));
        reg.register(Box::new(vless::VlessHandler));
        reg.register(Box::new(trojan::TrojanHandler));
        reg.register(Box::new(vmess::VmessHandler));
        reg.register(Box::new(wireguard::WireguardHandler));
        reg
    }

    #[test]
    fn registry_register_and_lookup() {
        let reg = make_registry();
        let ss = reg.get("shadowsocks").expect("shadowsocks should exist");
        assert_eq!(ss.name(), "shadowsocks");

        let vl = reg.get("vless").expect("vless should exist");
        assert_eq!(vl.name(), "vless");

        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn registry_list_protocols() {
        let reg = make_registry();
        let names = reg.list();
        assert_eq!(names.len(), 5);
        assert!(names.contains(&"shadowsocks"));
        assert!(names.contains(&"vless"));
        assert!(names.contains(&"trojan"));
        assert!(names.contains(&"vmess"));
        assert!(names.contains(&"wireguard"));
    }

    #[test]
    fn shadowsocks_metadata() {
        let h = shadowsocks::ShadowsocksHandler;
        assert_eq!(h.name(), "shadowsocks");
        assert_eq!(h.spec_reference(), "Shadowsocks AEAD/2022 specification");
        assert_eq!(h.clean_room_risk(), CleanRoomRisk::Low);
        assert!(!h.is_experimental());
    }

    #[test]
    fn vless_metadata() {
        let h = vless::VlessHandler;
        assert_eq!(h.name(), "vless");
        assert_eq!(h.spec_reference(), "VLESS protocol specification (XTLS)");
        assert_eq!(h.clean_room_risk(), CleanRoomRisk::Low);
        assert!(!h.is_experimental());
    }

    #[test]
    fn vmess_is_experimental() {
        let h = vmess::VmessHandler;
        assert_eq!(h.name(), "vmess");
        assert_eq!(h.clean_room_risk(), CleanRoomRisk::High);
        assert!(h.is_experimental());
    }

    #[test]
    fn wireguard_metadata() {
        let h = wireguard::WireguardHandler;
        assert_eq!(h.name(), "wireguard");
        assert_eq!(
            h.spec_reference(),
            "WireGuard whitepaper (Jason Donenfeld, 2017)"
        );
        assert_eq!(h.clean_room_risk(), CleanRoomRisk::Low);
        assert!(!h.is_experimental());
    }
}
