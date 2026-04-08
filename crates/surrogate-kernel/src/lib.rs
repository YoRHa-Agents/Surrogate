pub mod pool;
pub mod protocols;
pub mod session;
pub mod socks5;
pub mod streaming;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use surrogate_contract::config::{NormalizedConfig, OutboundKind};
use surrogate_contract::events::{Event, Observability};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

const CONNECT_OK_RESPONSE: &[u8] = b"HTTP/1.1 200 Connection Established\r\n\r\n";
const CONNECT_REJECT_RESPONSE: &[u8] = b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n";
const MAX_REQUEST_SIZE: usize = 8192;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestContext {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub rule_id: String,
    pub outbound: String,
}

#[derive(Debug, Clone)]
struct CompiledRule {
    id: String,
    host_equals: Option<String>,
    host_suffix: Option<String>,
    port: Option<u16>,
    outbound: String,
}

#[derive(Debug, Clone)]
pub struct RuleRegistry {
    default_outbound: String,
    rules: Vec<CompiledRule>,
    host_exact_index: HashMap<String, Vec<usize>>,
    suffix_rules: Vec<usize>,
    port_only_rules: Vec<usize>,
}

impl RuleRegistry {
    pub fn from_config(config: &NormalizedConfig) -> Result<Self, KernelError> {
        if config.outbounds.is_empty() {
            return Err(KernelError::NoOutboundsConfigured);
        }

        let rules: Vec<CompiledRule> = config
            .rules
            .iter()
            .map(|rule| CompiledRule {
                id: rule.id.clone(),
                host_equals: rule.host_equals.clone(),
                host_suffix: rule.host_suffix.clone(),
                port: rule.port,
                outbound: rule.outbound.clone(),
            })
            .collect();

        let mut host_exact_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut suffix_rules = Vec::new();
        let mut port_only_rules = Vec::new();

        for (idx, rule) in rules.iter().enumerate() {
            if let Some(ref host) = rule.host_equals {
                host_exact_index.entry(host.clone()).or_default().push(idx);
            } else if rule.host_suffix.is_some() {
                suffix_rules.push(idx);
            } else {
                port_only_rules.push(idx);
            }
        }

        Ok(Self {
            default_outbound: config.default_outbound.clone(),
            rules,
            host_exact_index,
            suffix_rules,
            port_only_rules,
        })
    }

    pub fn match_request(&self, request: &RequestContext) -> MatchResult {
        if let Some(indices) = self.host_exact_index.get(&request.host) {
            for &idx in indices {
                if rule_matches(&self.rules[idx], request) {
                    return MatchResult {
                        rule_id: self.rules[idx].id.clone(),
                        outbound: self.rules[idx].outbound.clone(),
                    };
                }
            }
        }

        for &idx in &self.suffix_rules {
            if rule_matches(&self.rules[idx], request) {
                return MatchResult {
                    rule_id: self.rules[idx].id.clone(),
                    outbound: self.rules[idx].outbound.clone(),
                };
            }
        }

        for &idx in &self.port_only_rules {
            if rule_matches(&self.rules[idx], request) {
                return MatchResult {
                    rule_id: self.rules[idx].id.clone(),
                    outbound: self.rules[idx].outbound.clone(),
                };
            }
        }

        MatchResult {
            rule_id: "default".to_string(),
            outbound: self.default_outbound.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct KernelState {
    pub(crate) outbounds: Arc<HashMap<String, OutboundKind>>,
    pub(crate) rules: RuleRegistry,
    pub(crate) observability: Observability,
    pub(crate) next_session_id: Arc<AtomicU64>,
}

pub struct Kernel {
    bind_addr: SocketAddr,
    state: KernelState,
}

impl Kernel {
    pub fn new(
        config: NormalizedConfig,
        observability: Observability,
    ) -> Result<Self, KernelError> {
        let bind_addr = config
            .listen_addr
            .parse::<SocketAddr>()
            .map_err(|_| KernelError::InvalidBindAddress(config.listen_addr.clone()))?;

        let outbounds = config
            .outbounds
            .iter()
            .map(|outbound| (outbound.id.clone(), outbound.kind))
            .collect::<HashMap<_, _>>();
        if !outbounds.contains_key(&config.default_outbound) {
            return Err(KernelError::UnknownOutbound(config.default_outbound));
        }

        let rules = RuleRegistry::from_config(&config)?;

        Ok(Self {
            bind_addr,
            state: KernelState {
                outbounds: Arc::new(outbounds),
                rules,
                observability,
                next_session_id: Arc::new(AtomicU64::new(0)),
            },
        })
    }

    pub async fn spawn(self) -> Result<RunningKernel, KernelError> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        let local_addr = listener.local_addr()?;
        let state = self.state.clone();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        let join_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = &mut shutdown_rx => {
                        break;
                    }
                    accepted = listener.accept() => {
                        let (stream, _) = match accepted {
                            Ok(conn) => conn,
                            Err(error) => {
                                state.observability.emit(Event::error(0, format!("accept error: {error}")));
                                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                                continue;
                            }
                        };
                        let connection_state = state.clone();
                        tokio::spawn(async move {
                            if let Err(error) = handle_client(stream, connection_state.clone()).await {
                                connection_state
                                    .observability
                                    .emit(Event::error(0, format!("connection task failed: {error}")));
                            }
                        });
                    }
                }
            }

            Ok::<(), KernelError>(())
        });

        Ok(RunningKernel {
            local_addr,
            shutdown_tx: Some(shutdown_tx),
            join_handle,
        })
    }
}

pub struct RunningKernel {
    local_addr: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
    join_handle: JoinHandle<Result<(), KernelError>>,
}

impl RunningKernel {
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub async fn shutdown(mut self) -> Result<(), KernelError> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        match self.join_handle.await {
            Ok(result) => result,
            Err(error) => Err(KernelError::Join(error.to_string())),
        }
    }
}

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("no outbounds configured")]
    NoOutboundsConfigured,
    #[error("bind address `{0}` is invalid")]
    InvalidBindAddress(String),
    #[error("unknown outbound `{0}`")]
    UnknownOutbound(String),
    #[error("client I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("listener task join failed: {0}")]
    Join(String),
    #[error("client closed before CONNECT request completed")]
    ClientClosed,
    #[error("CONNECT request exceeded {MAX_REQUEST_SIZE} bytes")]
    RequestTooLarge,
    #[error("CONNECT request was not valid UTF-8")]
    InvalidUtf8,
    #[error("CONNECT request line is missing")]
    MissingRequestLine,
    #[error("only CONNECT is supported during P0, got `{0}`")]
    UnsupportedMethod(String),
    #[error("CONNECT target `{0}` is invalid")]
    InvalidConnectTarget(String),
    #[error("failed to connect to upstream `{target}`: {source}")]
    UpstreamConnect {
        target: String,
        source: std::io::Error,
    },
    #[error("SOCKS5 protocol error: {0}")]
    Socks5ProtocolError(String),
    #[error("SOCKS5 unsupported command: 0x{0:02x}")]
    Socks5UnsupportedCommand(u8),
    #[error("connection pool exhausted")]
    PoolExhausted,
}

async fn handle_client(mut client: TcpStream, state: KernelState) -> Result<(), KernelError> {
    let session_id = state.next_session_id.fetch_add(1, Ordering::Relaxed) + 1;

    let result = dispatch_protocol(&mut client, &state, session_id).await;
    if let Err(error) = &result {
        state
            .observability
            .emit(Event::error(session_id, error.to_string()));
    }
    result
}

async fn dispatch_protocol(
    client: &mut TcpStream,
    state: &KernelState,
    session_id: u64,
) -> Result<(), KernelError> {
    let mut peek_buf = [0u8; 1];
    let n = client.peek(&mut peek_buf).await?;
    if n == 0 {
        return Err(KernelError::ClientClosed);
    }

    if peek_buf[0] == 0x05 {
        socks5::handle_socks5(client, state, session_id).await
    } else {
        handle_client_inner(client, state, session_id).await
    }
}

async fn handle_client_inner(
    client: &mut TcpStream,
    state: &KernelState,
    session_id: u64,
) -> Result<(), KernelError> {
    let request = read_connect_request(client).await?;
    state.observability.emit(Event::session_started(
        session_id,
        &request.host,
        request.port,
    ));

    let matched = state.rules.match_request(&request);
    state.observability.emit(Event::rule_matched(
        session_id,
        &request.host,
        request.port,
        &matched.rule_id,
        &matched.outbound,
    ));

    let outbound = state
        .outbounds
        .get(&matched.outbound)
        .copied()
        .ok_or_else(|| KernelError::UnknownOutbound(matched.outbound.clone()))?;

    match outbound {
        OutboundKind::Direct => {
            forward_direct(client, state, session_id, request, &matched.outbound).await
        }
        OutboundKind::Reject => reject_request(client, state, session_id, &matched.rule_id).await,
    }
}

async fn forward_direct(
    client: &mut TcpStream,
    state: &KernelState,
    session_id: u64,
    request: RequestContext,
    outbound: &str,
) -> Result<(), KernelError> {
    let target = format!("{}:{}", request.host, request.port);
    let mut upstream =
        TcpStream::connect(&target)
            .await
            .map_err(|source| KernelError::UpstreamConnect {
                target: target.clone(),
                source,
            })?;

    state.observability.emit(Event::forward_connected(
        session_id,
        &request.host,
        request.port,
        outbound,
    ));
    client.write_all(CONNECT_OK_RESPONSE).await?;

    let (bytes_from_client, bytes_from_upstream) =
        tokio::io::copy_bidirectional(client, &mut upstream).await?;
    state.observability.emit(Event::forward_closed(
        session_id,
        bytes_from_client,
        bytes_from_upstream,
    ));

    Ok(())
}

async fn reject_request(
    client: &mut TcpStream,
    state: &KernelState,
    session_id: u64,
    rule_id: &str,
) -> Result<(), KernelError> {
    client.write_all(CONNECT_REJECT_RESPONSE).await?;
    state.observability.emit(Event::error(
        session_id,
        format!("request rejected by rule `{rule_id}`"),
    ));
    Ok(())
}

async fn read_connect_request(client: &mut TcpStream) -> Result<RequestContext, KernelError> {
    let mut buffer = vec![0_u8; MAX_REQUEST_SIZE];
    let mut received = 0_usize;

    loop {
        if received == buffer.len() {
            return Err(KernelError::RequestTooLarge);
        }

        let read = client.read(&mut buffer[received..]).await?;
        if read == 0 {
            return Err(KernelError::ClientClosed);
        }
        received += read;

        if buffer[..received]
            .windows(4)
            .any(|window| window == b"\r\n\r\n")
        {
            let request =
                std::str::from_utf8(&buffer[..received]).map_err(|_| KernelError::InvalidUtf8)?;
            return parse_connect_request(request);
        }
    }
}

fn parse_connect_request(request: &str) -> Result<RequestContext, KernelError> {
    let request_line = request
        .lines()
        .next()
        .ok_or(KernelError::MissingRequestLine)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or(KernelError::MissingRequestLine)?;
    if !method.eq_ignore_ascii_case("CONNECT") {
        return Err(KernelError::UnsupportedMethod(method.to_string()));
    }

    let target = parts.next().ok_or(KernelError::MissingRequestLine)?;
    let _version = parts.next().ok_or(KernelError::MissingRequestLine)?;
    let (host, port) = parse_target(target)?;

    Ok(RequestContext { host, port })
}

fn parse_target(target: &str) -> Result<(String, u16), KernelError> {
    let (raw_host, raw_port) = target
        .rsplit_once(':')
        .ok_or_else(|| KernelError::InvalidConnectTarget(target.to_string()))?;
    let port = raw_port
        .parse::<u16>()
        .map_err(|_| KernelError::InvalidConnectTarget(target.to_string()))?;
    let host = raw_host
        .trim()
        .trim_matches('[')
        .trim_matches(']')
        .to_ascii_lowercase();
    if host.is_empty() {
        return Err(KernelError::InvalidConnectTarget(target.to_string()));
    }

    Ok((host, port))
}

fn rule_matches(rule: &CompiledRule, request: &RequestContext) -> bool {
    if let Some(host_equals) = &rule.host_equals
        && request.host != *host_equals
    {
        return false;
    }

    if let Some(host_suffix) = &rule.host_suffix
        && request.host != *host_suffix
        && !request.host.ends_with(&format!(".{host_suffix}"))
    {
        return false;
    }

    if let Some(port) = rule.port
        && request.port != port
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrogate_contract::config::{NormalizedOutbound, NormalizedRule};
    use surrogate_contract::events::Observability;

    #[test]
    fn matches_specific_rule_before_default() {
        let config = NormalizedConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![
                NormalizedOutbound {
                    id: "direct".to_string(),
                    kind: OutboundKind::Direct,
                },
                NormalizedOutbound {
                    id: "reject".to_string(),
                    kind: OutboundKind::Reject,
                },
            ],
            rules: vec![NormalizedRule {
                priority: 1,
                id: "block-example".to_string(),
                host_equals: Some("example.com".to_string()),
                host_suffix: None,
                port: Some(443),
                outbound: "reject".to_string(),
            }],
        };

        let registry = RuleRegistry::from_config(&config).expect("compile rules");
        let matched = registry.match_request(&RequestContext {
            host: "example.com".to_string(),
            port: 443,
        });
        let default = registry.match_request(&RequestContext {
            host: "localhost".to_string(),
            port: 8080,
        });

        assert_eq!(matched.rule_id, "block-example");
        assert_eq!(matched.outbound, "reject");
        assert_eq!(default.rule_id, "default");
        assert_eq!(default.outbound, "direct");
    }

    #[tokio::test]
    async fn rejects_invalid_methods() {
        let request = "GET localhost:443 HTTP/1.1\r\nHost: localhost:443\r\n\r\n";
        let error = parse_connect_request(request).expect_err("non-CONNECT requests must fail");
        assert!(matches!(error, KernelError::UnsupportedMethod(_)));
    }

    #[tokio::test]
    async fn kernel_creation_requires_default_outbound() {
        let config = NormalizedConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            default_outbound: "missing".to_string(),
            outbounds: vec![NormalizedOutbound {
                id: "direct".to_string(),
                kind: OutboundKind::Direct,
            }],
            rules: Vec::new(),
        };

        let error = match Kernel::new(config, Observability::stdout()) {
            Ok(_) => panic!("kernel creation should fail when default outbound is missing"),
            Err(error) => error,
        };
        assert!(matches!(error, KernelError::UnknownOutbound(_)));
    }

    #[test]
    fn suffix_rule_matches_subdomains() {
        let config = NormalizedConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![
                NormalizedOutbound {
                    id: "direct".to_string(),
                    kind: OutboundKind::Direct,
                },
                NormalizedOutbound {
                    id: "reject".to_string(),
                    kind: OutboundKind::Reject,
                },
            ],
            rules: vec![NormalizedRule {
                priority: 1,
                id: "suffix-match".to_string(),
                host_equals: None,
                host_suffix: Some("example.com".to_string()),
                port: None,
                outbound: "reject".to_string(),
            }],
        };

        let registry = RuleRegistry::from_config(&config).expect("compile rules");
        let matched = registry.match_request(&RequestContext {
            host: "sub.example.com".to_string(),
            port: 443,
        });
        assert_eq!(matched.rule_id, "suffix-match");
        assert_eq!(matched.outbound, "reject");
    }

    #[test]
    fn suffix_rule_does_not_match_different_domain() {
        let config = NormalizedConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![
                NormalizedOutbound {
                    id: "direct".to_string(),
                    kind: OutboundKind::Direct,
                },
                NormalizedOutbound {
                    id: "reject".to_string(),
                    kind: OutboundKind::Reject,
                },
            ],
            rules: vec![NormalizedRule {
                priority: 1,
                id: "suffix-match".to_string(),
                host_equals: None,
                host_suffix: Some("example.com".to_string()),
                port: None,
                outbound: "reject".to_string(),
            }],
        };

        let registry = RuleRegistry::from_config(&config).expect("compile rules");
        let matched = registry.match_request(&RequestContext {
            host: "notexample.com".to_string(),
            port: 443,
        });
        assert_eq!(matched.rule_id, "default");
        assert_eq!(matched.outbound, "direct");
    }

    #[test]
    fn port_only_rule_matches_any_host() {
        let config = NormalizedConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![
                NormalizedOutbound {
                    id: "direct".to_string(),
                    kind: OutboundKind::Direct,
                },
                NormalizedOutbound {
                    id: "reject".to_string(),
                    kind: OutboundKind::Reject,
                },
            ],
            rules: vec![NormalizedRule {
                priority: 1,
                id: "port-only".to_string(),
                host_equals: None,
                host_suffix: None,
                port: Some(8080),
                outbound: "reject".to_string(),
            }],
        };

        let registry = RuleRegistry::from_config(&config).expect("compile rules");
        let matched = registry.match_request(&RequestContext {
            host: "anything.example.org".to_string(),
            port: 8080,
        });
        assert_eq!(matched.rule_id, "port-only");
        assert_eq!(matched.outbound, "reject");
    }

    #[test]
    fn multiple_rules_first_match_wins() {
        let config = NormalizedConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![
                NormalizedOutbound {
                    id: "direct".to_string(),
                    kind: OutboundKind::Direct,
                },
                NormalizedOutbound {
                    id: "reject".to_string(),
                    kind: OutboundKind::Reject,
                },
            ],
            rules: vec![
                NormalizedRule {
                    priority: 1,
                    id: "first-rule".to_string(),
                    host_equals: Some("dup.example.com".to_string()),
                    host_suffix: None,
                    port: None,
                    outbound: "direct".to_string(),
                },
                NormalizedRule {
                    priority: 2,
                    id: "second-rule".to_string(),
                    host_equals: Some("dup.example.com".to_string()),
                    host_suffix: None,
                    port: None,
                    outbound: "reject".to_string(),
                },
            ],
        };

        let registry = RuleRegistry::from_config(&config).expect("compile rules");
        let matched = registry.match_request(&RequestContext {
            host: "dup.example.com".to_string(),
            port: 80,
        });
        assert_eq!(matched.rule_id, "first-rule");
        assert_eq!(matched.outbound, "direct");
    }

    #[test]
    fn host_exact_takes_precedence_over_suffix() {
        let config = NormalizedConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![
                NormalizedOutbound {
                    id: "direct".to_string(),
                    kind: OutboundKind::Direct,
                },
                NormalizedOutbound {
                    id: "reject".to_string(),
                    kind: OutboundKind::Reject,
                },
            ],
            rules: vec![
                NormalizedRule {
                    priority: 1,
                    id: "suffix-rule".to_string(),
                    host_equals: None,
                    host_suffix: Some("example.com".to_string()),
                    port: None,
                    outbound: "direct".to_string(),
                },
                NormalizedRule {
                    priority: 2,
                    id: "exact-rule".to_string(),
                    host_equals: Some("example.com".to_string()),
                    host_suffix: None,
                    port: None,
                    outbound: "reject".to_string(),
                },
            ],
        };

        let registry = RuleRegistry::from_config(&config).expect("compile rules");
        let matched = registry.match_request(&RequestContext {
            host: "example.com".to_string(),
            port: 443,
        });
        assert_eq!(matched.rule_id, "exact-rule");
        assert_eq!(matched.outbound, "reject");
    }

    #[test]
    fn empty_rules_uses_default_outbound() {
        let config = NormalizedConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![NormalizedOutbound {
                id: "direct".to_string(),
                kind: OutboundKind::Direct,
            }],
            rules: Vec::new(),
        };

        let registry = RuleRegistry::from_config(&config).expect("compile rules");
        let matched = registry.match_request(&RequestContext {
            host: "anything.com".to_string(),
            port: 8080,
        });
        assert_eq!(matched.rule_id, "default");
        assert_eq!(matched.outbound, "direct");
    }

    #[test]
    fn parse_connect_ipv6_target() {
        let request = "CONNECT [::1]:443 HTTP/1.1\r\nHost: [::1]:443\r\n\r\n";
        let ctx = parse_connect_request(request).expect("parse IPv6 target");
        assert_eq!(ctx.host, "::1");
        assert_eq!(ctx.port, 443);
    }

    #[test]
    fn parse_connect_missing_port() {
        let request = "CONNECT example.com HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let error = parse_connect_request(request).expect_err("missing port must fail");
        assert!(matches!(error, KernelError::InvalidConnectTarget(_)));
    }

    #[test]
    fn parse_connect_empty_host() {
        let request = "CONNECT :443 HTTP/1.1\r\nHost: :443\r\n\r\n";
        let error = parse_connect_request(request).expect_err("empty host must fail");
        assert!(matches!(error, KernelError::InvalidConnectTarget(_)));
    }

    #[test]
    fn parse_connect_case_insensitive_method() {
        let request = "connect example.com:443 HTTP/1.1\r\nHost: example.com:443\r\n\r\n";
        let ctx = parse_connect_request(request).expect("lowercase method should work");
        assert_eq!(ctx.host, "example.com");
        assert_eq!(ctx.port, 443);
    }

    #[test]
    fn kernel_rejects_empty_outbounds() {
        let config = NormalizedConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: Vec::new(),
            rules: Vec::new(),
        };
        let error =
            RuleRegistry::from_config(&config).expect_err("should fail with empty outbounds");
        assert!(matches!(error, KernelError::NoOutboundsConfigured));
    }

    #[test]
    fn kernel_rejects_invalid_bind_address() {
        let config = NormalizedConfig {
            listen_addr: "not_valid".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![NormalizedOutbound {
                id: "direct".to_string(),
                kind: OutboundKind::Direct,
            }],
            rules: Vec::new(),
        };
        match Kernel::new(config, Observability::stdout()) {
            Ok(_) => panic!("should reject invalid bind address"),
            Err(error) => assert!(matches!(error, KernelError::InvalidBindAddress(_))),
        }
    }
}
