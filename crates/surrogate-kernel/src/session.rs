use crate::MatchResult;
use std::fmt;
use std::net::SocketAddr;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionProtocol {
    HttpConnect,
    Socks5,
}

impl fmt::Display for SessionProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionProtocol::HttpConnect => f.write_str("HTTP CONNECT"),
            SessionProtocol::Socks5 => f.write_str("SOCKS5"),
        }
    }
}

pub struct Session {
    pub id: u64,
    pub protocol: SessionProtocol,
    pub source_addr: SocketAddr,
    pub target_host: String,
    pub target_port: u16,
    pub rule_match: Option<MatchResult>,
    pub started_at: Instant,
}

impl Session {
    pub fn new(
        id: u64,
        protocol: SessionProtocol,
        source_addr: SocketAddr,
        target_host: String,
        target_port: u16,
    ) -> Self {
        Self {
            id,
            protocol,
            source_addr,
            target_host,
            target_port,
            rule_match: None,
            started_at: Instant::now(),
        }
    }

    pub fn set_rule_match(&mut self, rule_match: MatchResult) {
        self.rule_match = Some(rule_match);
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;

    #[test]
    fn session_protocol_display() {
        assert_eq!(format!("{}", SessionProtocol::HttpConnect), "HTTP CONNECT");
        assert_eq!(format!("{}", SessionProtocol::Socks5), "SOCKS5");
    }

    #[test]
    fn session_new_sets_fields() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345);
        let session = Session::new(
            42,
            SessionProtocol::HttpConnect,
            addr,
            "example.com".to_string(),
            443,
        );
        assert_eq!(session.id, 42);
        assert_eq!(session.protocol, SessionProtocol::HttpConnect);
        assert_eq!(session.source_addr, addr);
        assert_eq!(session.target_host, "example.com");
        assert_eq!(session.target_port, 443);
        assert!(session.rule_match.is_none());
    }

    #[test]
    fn session_elapsed_increases() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345);
        let session = Session::new(1, SessionProtocol::Socks5, addr, "test.com".to_string(), 80);
        std::thread::sleep(Duration::from_millis(1));
        assert!(session.elapsed() > Duration::ZERO);
    }

    #[test]
    fn session_set_rule_match() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345);
        let mut session = Session::new(
            1,
            SessionProtocol::HttpConnect,
            addr,
            "example.com".to_string(),
            443,
        );
        assert!(session.rule_match.is_none());
        session.set_rule_match(MatchResult {
            rule_id: "test-rule".to_string(),
            outbound: "direct".to_string(),
        });
        let m = session.rule_match.expect("rule_match should be Some");
        assert_eq!(m.rule_id, "test-rule");
        assert_eq!(m.outbound, "direct");
    }
}
