use crate::{KernelError, KernelState, RequestContext};
use std::net::{Ipv4Addr, Ipv6Addr};
use surrogate_contract::config::OutboundKind;
use surrogate_contract::events::Event;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const SOCKS5_VERSION: u8 = 0x05;
const AUTH_NO_AUTH: u8 = 0x00;
const AUTH_NO_ACCEPTABLE: u8 = 0xFF;

const CMD_CONNECT: u8 = 0x01;
const ATYP_IPV4: u8 = 0x01;
const ATYP_DOMAIN: u8 = 0x03;
const ATYP_IPV6: u8 = 0x04;

const REP_SUCCESS: u8 = 0x00;
const REP_GENERAL_FAILURE: u8 = 0x01;
const REP_HOST_UNREACHABLE: u8 = 0x04;
const REP_CMD_NOT_SUPPORTED: u8 = 0x07;
const REP_ATYP_NOT_SUPPORTED: u8 = 0x08;

#[derive(Debug, PartialEq, Eq)]
pub struct Greeting {
    pub version: u8,
    pub methods: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Address {
    Ipv4(Ipv4Addr),
    Domain(String),
    Ipv6(Ipv6Addr),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConnectRequest {
    pub addr: Address,
    pub port: u16,
}

impl ConnectRequest {
    pub fn to_request_context(&self) -> RequestContext {
        let host = match &self.addr {
            Address::Ipv4(addr) => addr.to_string(),
            Address::Domain(domain) => domain.to_ascii_lowercase(),
            Address::Ipv6(addr) => addr.to_string(),
        };
        RequestContext {
            host,
            port: self.port,
        }
    }
}

pub fn parse_greeting(data: &[u8]) -> Result<Greeting, KernelError> {
    if data.len() < 2 {
        return Err(KernelError::Socks5ProtocolError(
            "greeting too short".into(),
        ));
    }
    let version = data[0];
    if version != SOCKS5_VERSION {
        return Err(KernelError::Socks5ProtocolError(format!(
            "expected SOCKS5 version 5, got {version}"
        )));
    }
    let nmethods = data[1] as usize;
    if data.len() < 2 + nmethods {
        return Err(KernelError::Socks5ProtocolError(
            "greeting truncated".into(),
        ));
    }
    let methods = data[2..2 + nmethods].to_vec();
    Ok(Greeting { version, methods })
}

pub fn parse_connect_request(data: &[u8]) -> Result<ConnectRequest, KernelError> {
    if data.len() < 4 {
        return Err(KernelError::Socks5ProtocolError("request too short".into()));
    }
    if data[0] != SOCKS5_VERSION {
        return Err(KernelError::Socks5ProtocolError(format!(
            "expected version 5, got {}",
            data[0]
        )));
    }
    if data[1] != CMD_CONNECT {
        return Err(KernelError::Socks5UnsupportedCommand(data[1]));
    }
    let atyp = data[3];
    match atyp {
        ATYP_IPV4 => {
            if data.len() < 10 {
                return Err(KernelError::Socks5ProtocolError(
                    "IPv4 request truncated".into(),
                ));
            }
            let addr = Ipv4Addr::new(data[4], data[5], data[6], data[7]);
            let port = u16::from_be_bytes([data[8], data[9]]);
            Ok(ConnectRequest {
                addr: Address::Ipv4(addr),
                port,
            })
        }
        ATYP_DOMAIN => {
            if data.len() < 5 {
                return Err(KernelError::Socks5ProtocolError(
                    "domain request truncated".into(),
                ));
            }
            let domain_len = data[4] as usize;
            if data.len() < 5 + domain_len + 2 {
                return Err(KernelError::Socks5ProtocolError(
                    "domain request truncated".into(),
                ));
            }
            let domain = String::from_utf8(data[5..5 + domain_len].to_vec()).map_err(|_| {
                KernelError::Socks5ProtocolError("domain is not valid UTF-8".into())
            })?;
            let port_offset = 5 + domain_len;
            let port = u16::from_be_bytes([data[port_offset], data[port_offset + 1]]);
            Ok(ConnectRequest {
                addr: Address::Domain(domain),
                port,
            })
        }
        ATYP_IPV6 => {
            if data.len() < 22 {
                return Err(KernelError::Socks5ProtocolError(
                    "IPv6 request truncated".into(),
                ));
            }
            let mut octets = [0u8; 16];
            octets.copy_from_slice(&data[4..20]);
            let addr = Ipv6Addr::from(octets);
            let port = u16::from_be_bytes([data[20], data[21]]);
            Ok(ConnectRequest {
                addr: Address::Ipv6(addr),
                port,
            })
        }
        _ => Err(KernelError::Socks5ProtocolError(format!(
            "unsupported address type 0x{atyp:02x}"
        ))),
    }
}

fn build_reply(rep: u8) -> [u8; 10] {
    [SOCKS5_VERSION, rep, 0x00, ATYP_IPV4, 0, 0, 0, 0, 0, 0]
}

pub(crate) async fn handle_socks5(
    client: &mut TcpStream,
    state: &KernelState,
    session_id: u64,
) -> Result<(), KernelError> {
    let mut header = [0u8; 2];
    client.read_exact(&mut header).await?;
    let nmethods = header[1] as usize;
    let mut methods_buf = vec![0u8; nmethods];
    if nmethods > 0 {
        client.read_exact(&mut methods_buf).await?;
    }

    let mut greeting_data = Vec::with_capacity(2 + nmethods);
    greeting_data.extend_from_slice(&header);
    greeting_data.extend_from_slice(&methods_buf);
    let greeting = parse_greeting(&greeting_data)?;

    if !greeting.methods.contains(&AUTH_NO_AUTH) {
        client
            .write_all(&[SOCKS5_VERSION, AUTH_NO_ACCEPTABLE])
            .await?;
        return Err(KernelError::Socks5ProtocolError(
            "no acceptable auth method".into(),
        ));
    }
    client.write_all(&[SOCKS5_VERSION, AUTH_NO_AUTH]).await?;

    let mut req_header = [0u8; 4];
    client.read_exact(&mut req_header).await?;

    if req_header[0] != SOCKS5_VERSION {
        client.write_all(&build_reply(REP_GENERAL_FAILURE)).await?;
        return Err(KernelError::Socks5ProtocolError(format!(
            "expected version 5 in request, got {}",
            req_header[0]
        )));
    }
    if req_header[1] != CMD_CONNECT {
        client
            .write_all(&build_reply(REP_CMD_NOT_SUPPORTED))
            .await?;
        return Err(KernelError::Socks5UnsupportedCommand(req_header[1]));
    }

    let atyp = req_header[3];
    let addr_data = match atyp {
        ATYP_IPV4 => {
            let mut buf = [0u8; 6];
            client.read_exact(&mut buf).await?;
            buf.to_vec()
        }
        ATYP_DOMAIN => {
            let mut len_buf = [0u8; 1];
            client.read_exact(&mut len_buf).await?;
            let domain_len = len_buf[0] as usize;
            let mut buf = vec![0u8; domain_len + 2];
            client.read_exact(&mut buf).await?;
            let mut result = vec![len_buf[0]];
            result.extend_from_slice(&buf);
            result
        }
        ATYP_IPV6 => {
            let mut buf = [0u8; 18];
            client.read_exact(&mut buf).await?;
            buf.to_vec()
        }
        _ => {
            client
                .write_all(&build_reply(REP_ATYP_NOT_SUPPORTED))
                .await?;
            return Err(KernelError::Socks5ProtocolError(format!(
                "unsupported address type 0x{atyp:02x}"
            )));
        }
    };

    let mut request_data = Vec::with_capacity(4 + addr_data.len());
    request_data.extend_from_slice(&req_header);
    request_data.extend_from_slice(&addr_data);

    let connect_req = match parse_connect_request(&request_data) {
        Ok(req) => req,
        Err(e) => {
            client.write_all(&build_reply(REP_GENERAL_FAILURE)).await?;
            return Err(e);
        }
    };

    let request = connect_req.to_request_context();

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
            let target = format!("{}:{}", request.host, request.port);
            match TcpStream::connect(&target).await {
                Ok(mut upstream) => {
                    state.observability.emit(Event::forward_connected(
                        session_id,
                        &request.host,
                        request.port,
                        &matched.outbound,
                    ));
                    client.write_all(&build_reply(REP_SUCCESS)).await?;

                    let (bytes_from_client, bytes_from_upstream) =
                        tokio::io::copy_bidirectional(client, &mut upstream).await?;
                    state.observability.emit(Event::forward_closed(
                        session_id,
                        bytes_from_client,
                        bytes_from_upstream,
                    ));
                    Ok(())
                }
                Err(source) => {
                    client.write_all(&build_reply(REP_HOST_UNREACHABLE)).await?;
                    Err(KernelError::UpstreamConnect { target, source })
                }
            }
        }
        OutboundKind::Reject => {
            client.write_all(&build_reply(REP_GENERAL_FAILURE)).await?;
            state.observability.emit(Event::error(
                session_id,
                format!("request rejected by rule `{}`", matched.rule_id),
            ));
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socks5_greeting_parse() {
        let data = [0x05, 0x01, 0x00];
        let greeting = parse_greeting(&data).unwrap();
        assert_eq!(greeting.version, 5);
        assert_eq!(greeting.methods, vec![0x00]);

        let data = [0x05, 0x03, 0x00, 0x01, 0x02];
        let greeting = parse_greeting(&data).unwrap();
        assert_eq!(greeting.methods, vec![0x00, 0x01, 0x02]);

        let data = [0x04, 0x01, 0x00];
        assert!(parse_greeting(&data).is_err());

        let data = [0x05, 0x02, 0x00];
        assert!(parse_greeting(&data).is_err());

        let data = [0x05];
        assert!(parse_greeting(&data).is_err());
    }

    #[test]
    fn socks5_connect_request_parse() {
        let data = [0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1, 0x00, 0x50];
        let req = parse_connect_request(&data).unwrap();
        assert_eq!(req.addr, Address::Ipv4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(req.port, 80);

        let domain = b"example.com";
        let mut data = vec![0x05, 0x01, 0x00, 0x03, domain.len() as u8];
        data.extend_from_slice(domain);
        data.extend_from_slice(&443u16.to_be_bytes());
        let req = parse_connect_request(&data).unwrap();
        assert_eq!(req.addr, Address::Domain("example.com".to_string()));
        assert_eq!(req.port, 443);

        let mut data = vec![0x05, 0x01, 0x00, 0x04];
        let ipv6 = Ipv6Addr::LOCALHOST;
        data.extend_from_slice(&ipv6.octets());
        data.extend_from_slice(&8080u16.to_be_bytes());
        let req = parse_connect_request(&data).unwrap();
        assert_eq!(req.addr, Address::Ipv6(Ipv6Addr::LOCALHOST));
        assert_eq!(req.port, 8080);

        let data = [0x05, 0x02, 0x00, 0x01, 127, 0, 0, 1, 0x00, 0x50];
        let err = parse_connect_request(&data).unwrap_err();
        assert!(matches!(err, KernelError::Socks5UnsupportedCommand(0x02)));

        let data = [0x05, 0x01, 0x00, 0x01, 127, 0];
        assert!(parse_connect_request(&data).is_err());
    }

    #[test]
    fn socks5_domain_case_lowered_in_request_context() {
        let domain = b"EXAMPLE.COM";
        let mut data = vec![0x05, 0x01, 0x00, 0x03, domain.len() as u8];
        data.extend_from_slice(domain);
        data.extend_from_slice(&443u16.to_be_bytes());
        let req = parse_connect_request(&data).unwrap();
        let ctx = req.to_request_context();
        assert_eq!(ctx.host, "example.com");
        assert_eq!(ctx.port, 443);
    }

    #[test]
    fn socks5_ipv6_to_request_context() {
        let ipv6 = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        let mut data = vec![0x05, 0x01, 0x00, 0x04];
        data.extend_from_slice(&ipv6.octets());
        data.extend_from_slice(&443u16.to_be_bytes());
        let req = parse_connect_request(&data).unwrap();
        let ctx = req.to_request_context();
        assert_eq!(ctx.host, "::1");
        assert_eq!(ctx.port, 443);
    }

    #[test]
    fn socks5_unsupported_address_type() {
        let data = [0x05, 0x01, 0x00, 0x99];
        let err = parse_connect_request(&data).unwrap_err();
        assert!(matches!(err, KernelError::Socks5ProtocolError(ref msg) if msg.contains("0x99")));
    }

    #[test]
    fn socks5_empty_greeting_methods() {
        let data = [0x05, 0x00];
        let greeting = parse_greeting(&data).unwrap();
        assert_eq!(greeting.version, 5);
        assert!(greeting.methods.is_empty());
    }

    #[test]
    fn socks5_zero_length_domain() {
        let mut data = vec![0x05, 0x01, 0x00, 0x03, 0x00];
        data.extend_from_slice(&443u16.to_be_bytes());
        let req = parse_connect_request(&data).unwrap();
        assert_eq!(req.addr, Address::Domain(String::new()));
        assert_eq!(req.port, 443);
    }
}
