use std::time::{SystemTime, UNIX_EPOCH};
use surrogate_contract::config::{load_and_validate, normalize};
use surrogate_contract::events::{EventCollector, EventKind, Observability};
use surrogate_kernel::Kernel;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn forwards_one_real_request_through_kernel() {
    let target = spawn_target_server().await;
    let config_path = unique_test_path("surrogate_kernel_connect_flow");
    let config = format!(
        r#"
listen = "127.0.0.1:0"
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"

[[outbounds]]
id = "reject"
type = "reject"

[[rules]]
id = "local-http"
host_equals = "127.0.0.1"
port = {port}
outbound = "direct"
"#,
        port = target.port
    );
    std::fs::write(&config_path, config).expect("write config fixture");

    let document = load_and_validate(&config_path).expect("load config");
    let normalized = normalize(&document);
    let (observability, collector) = Observability::collector();
    let kernel = Kernel::new(normalized, observability).expect("build kernel");
    let running = kernel.spawn().await.expect("start kernel");

    let mut client = TcpStream::connect(running.local_addr())
        .await
        .expect("connect to kernel");
    client
        .write_all(
            format!(
                "CONNECT 127.0.0.1:{port} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n",
                port = target.port
            )
            .as_bytes(),
        )
        .await
        .expect("write connect request");

    let connect_response = read_until_headers_complete(&mut client).await;
    let connect_response = String::from_utf8(connect_response).expect("connect response is utf-8");
    assert!(connect_response.contains("200 Connection Established"));

    client
        .write_all(b"GET /health HTTP/1.1\r\nHost: local-test\r\nConnection: close\r\n\r\n")
        .await
        .expect("write upstream request");
    client
        .shutdown()
        .await
        .expect("close client write half after request");

    let mut response = Vec::new();
    client
        .read_to_end(&mut response)
        .await
        .expect("read upstream response");
    let response = String::from_utf8(response).expect("upstream response is utf-8");
    assert!(response.contains("hello kernel"));

    target
        .join_handle
        .await
        .expect("target server task should exit");
    running.shutdown().await.expect("shutdown kernel");
    wait_for_forward_closed(&collector).await;

    let events = collector.events();
    let event_kinds = events
        .iter()
        .map(|event| event.kind.clone())
        .collect::<Vec<_>>();
    assert!(event_kinds.contains(&EventKind::SessionStarted));
    assert!(event_kinds.contains(&EventKind::RuleMatched));
    assert!(event_kinds.contains(&EventKind::ForwardConnected));
    assert!(event_kinds.contains(&EventKind::ForwardClosed));
    assert!(
        events
            .iter()
            .any(|event| event.rule_id.as_deref() == Some("local-http"))
    );

    let _ = std::fs::remove_file(config_path);
}

struct TargetServer {
    port: u16,
    join_handle: JoinHandle<()>,
}

async fn spawn_target_server() -> TargetServer {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind target server");
    let port = listener.local_addr().expect("target server addr").port();
    let join_handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept proxied client");
        let _request = read_until_headers_complete(&mut stream).await;
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\nConnection: close\r\n\r\nhello kernel",
            )
            .await
            .expect("write target response");
    });

    TargetServer { port, join_handle }
}

async fn read_until_headers_complete(stream: &mut TcpStream) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 512];

    loop {
        let read = stream.read(&mut chunk).await.expect("read from stream");
        assert!(read > 0, "stream closed before headers completed");
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            return buffer;
        }
    }
}

fn unique_test_path(prefix: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nanos}.toml"))
}

async fn wait_for_forward_closed(collector: &EventCollector) {
    for _ in 0..20 {
        if collector
            .events()
            .iter()
            .any(|event| event.kind == EventKind::ForwardClosed)
        {
            return;
        }
        sleep(Duration::from_millis(10)).await;
    }

    panic!("forward_closed event was not emitted in time");
}

async fn wait_for_event(collector: &EventCollector, kind: EventKind) {
    for _ in 0..40 {
        if collector.events().iter().any(|event| event.kind == kind) {
            return;
        }
        sleep(Duration::from_millis(10)).await;
    }
    panic!("{kind:?} event was not emitted in time");
}

#[tokio::test]
async fn http_connect_reject_returns_403() {
    let config_path = unique_test_path("surrogate_kernel_reject_flow");
    let config = r#"
listen = "127.0.0.1:0"
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"

[[outbounds]]
id = "reject"
type = "reject"

[[rules]]
id = "block-target"
host_equals = "blocked.example.com"
port = 443
outbound = "reject"
"#;
    std::fs::write(&config_path, config).expect("write config fixture");

    let document = load_and_validate(&config_path).expect("load config");
    let normalized = normalize(&document);
    let (observability, collector) = Observability::collector();
    let kernel = Kernel::new(normalized, observability).expect("build kernel");
    let running = kernel.spawn().await.expect("start kernel");

    let mut client = TcpStream::connect(running.local_addr())
        .await
        .expect("connect to kernel");
    client
        .write_all(
            b"CONNECT blocked.example.com:443 HTTP/1.1\r\nHost: blocked.example.com:443\r\n\r\n",
        )
        .await
        .expect("write connect request");

    let response = read_until_headers_complete(&mut client).await;
    let response = String::from_utf8(response).expect("response is utf-8");
    assert!(
        response.contains("403 Forbidden"),
        "expected 403 response, got: {response}"
    );

    running.shutdown().await.expect("shutdown kernel");
    wait_for_event(&collector, EventKind::Error).await;

    let events = collector.events();
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::Error && e.message.contains("rejected")),
        "expected error event with 'rejected' in message, got: {events:?}"
    );

    let _ = std::fs::remove_file(config_path);
}

#[tokio::test]
async fn socks5_direct_forward_roundtrip() {
    let target = spawn_target_server().await;
    let config_path = unique_test_path("surrogate_kernel_socks5_direct");
    let config = r#"
listen = "127.0.0.1:0"
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"
"#
    .to_string();
    std::fs::write(&config_path, config).expect("write config fixture");

    let document = load_and_validate(&config_path).expect("load config");
    let normalized = normalize(&document);
    let (observability, collector) = Observability::collector();
    let kernel = Kernel::new(normalized, observability).expect("build kernel");
    let running = kernel.spawn().await.expect("start kernel");

    let mut client = TcpStream::connect(running.local_addr())
        .await
        .expect("connect to kernel");

    client
        .write_all(&[0x05, 0x01, 0x00])
        .await
        .expect("write SOCKS5 greeting");
    let mut auth_resp = [0u8; 2];
    client
        .read_exact(&mut auth_resp)
        .await
        .expect("read auth response");
    assert_eq!(auth_resp, [0x05, 0x00]);

    let target_port = target.port;
    let mut connect_req = vec![0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1];
    connect_req.extend_from_slice(&target_port.to_be_bytes());
    client
        .write_all(&connect_req)
        .await
        .expect("write SOCKS5 connect request");

    let mut reply = [0u8; 10];
    client
        .read_exact(&mut reply)
        .await
        .expect("read SOCKS5 connect reply");
    assert_eq!(reply[0], 0x05);
    assert_eq!(reply[1], 0x00, "expected REP_SUCCESS");

    client
        .write_all(b"GET /health HTTP/1.1\r\nHost: local-test\r\nConnection: close\r\n\r\n")
        .await
        .expect("write upstream request through tunnel");
    client
        .shutdown()
        .await
        .expect("close client write half after request");

    let mut response = Vec::new();
    client
        .read_to_end(&mut response)
        .await
        .expect("read upstream response");
    let response = String::from_utf8(response).expect("upstream response is utf-8");
    assert!(
        response.contains("hello kernel"),
        "expected 'hello kernel' in response: {response}"
    );

    target
        .join_handle
        .await
        .expect("target server task should exit");
    running.shutdown().await.expect("shutdown kernel");
    wait_for_forward_closed(&collector).await;

    let events = collector.events();
    let event_kinds: Vec<_> = events.iter().map(|e| e.kind.clone()).collect();
    assert!(event_kinds.contains(&EventKind::SessionStarted));
    assert!(event_kinds.contains(&EventKind::RuleMatched));
    assert!(event_kinds.contains(&EventKind::ForwardConnected));
    assert!(event_kinds.contains(&EventKind::ForwardClosed));

    let _ = std::fs::remove_file(config_path);
}

#[tokio::test]
async fn socks5_reject_returns_failure_reply() {
    let config_path = unique_test_path("surrogate_kernel_socks5_reject");
    let config = r#"
listen = "127.0.0.1:0"
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"

[[outbounds]]
id = "reject"
type = "reject"

[[rules]]
id = "block-target"
host_equals = "127.0.0.1"
port = 9999
outbound = "reject"
"#;
    std::fs::write(&config_path, config).expect("write config fixture");

    let document = load_and_validate(&config_path).expect("load config");
    let normalized = normalize(&document);
    let (observability, collector) = Observability::collector();
    let kernel = Kernel::new(normalized, observability).expect("build kernel");
    let running = kernel.spawn().await.expect("start kernel");

    let mut client = TcpStream::connect(running.local_addr())
        .await
        .expect("connect to kernel");

    client
        .write_all(&[0x05, 0x01, 0x00])
        .await
        .expect("write SOCKS5 greeting");
    let mut auth_resp = [0u8; 2];
    client
        .read_exact(&mut auth_resp)
        .await
        .expect("read auth response");
    assert_eq!(auth_resp, [0x05, 0x00]);

    // port 9999 = 0x270F
    let connect_req = [0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1, 0x27, 0x0F];
    client
        .write_all(&connect_req)
        .await
        .expect("write SOCKS5 connect request");

    let mut reply = [0u8; 10];
    client
        .read_exact(&mut reply)
        .await
        .expect("read SOCKS5 connect reply");
    assert_eq!(reply[0], 0x05);
    assert_eq!(reply[1], 0x01, "expected REP_GENERAL_FAILURE");

    running.shutdown().await.expect("shutdown kernel");
    wait_for_event(&collector, EventKind::Error).await;

    let events = collector.events();
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::Error && e.message.contains("rejected")),
        "expected error event with 'rejected' in message, got: {events:?}"
    );

    let _ = std::fs::remove_file(config_path);
}

#[tokio::test]
async fn non_connect_method_returns_405() {
    let config_path = unique_test_path("surrogate_kernel_405_test");
    let config = r#"
listen = "127.0.0.1:0"
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"
"#;
    std::fs::write(&config_path, config).expect("write config fixture");

    let document = load_and_validate(&config_path).expect("load config");
    let normalized = normalize(&document);
    let (observability, collector) = Observability::collector();
    let kernel = Kernel::new(normalized, observability).expect("build kernel");
    let running = kernel.spawn().await.expect("start kernel");

    let mut client = TcpStream::connect(running.local_addr())
        .await
        .expect("connect to kernel");
    client
        .write_all(b"GET http://example.com/ HTTP/1.1\r\nHost: example.com\r\n\r\n")
        .await
        .expect("write GET request");

    let response = read_until_headers_complete(&mut client).await;
    let response = String::from_utf8(response).expect("response is utf-8");
    assert!(
        response.contains("405 Method Not Allowed"),
        "expected 405 response, got: {response}"
    );
    assert!(
        response.contains("Allow: CONNECT"),
        "expected Allow header, got: {response}"
    );

    running.shutdown().await.expect("shutdown kernel");
    wait_for_event(&collector, EventKind::Error).await;

    let events = collector.events();
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::Error && e.message.contains("CONNECT")),
        "expected error event about CONNECT, got: {events:?}"
    );

    let _ = std::fs::remove_file(config_path);
}
