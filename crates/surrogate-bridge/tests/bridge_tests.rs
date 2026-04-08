use std::net::SocketAddr;

use surrogate_bridge::bridge_trait::PlatformBridge;
use surrogate_bridge::linux::{LinuxExplicitProxyBridge, extract_client_metadata};
use surrogate_bridge::macos::MacOsBridge;

#[test]
fn macos_bridge_reports_correct_platform_name() {
    let bridge = MacOsBridge::new();
    assert_eq!(bridge.platform_name(), "macos");
}

#[test]
fn macos_bridge_supports_process_id() {
    let bridge = MacOsBridge::new();
    assert!(bridge.supports_process_identification());
}

#[test]
fn linux_bridge_reports_correct_platform_name() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let bridge = LinuxExplicitProxyBridge::new(addr);
    assert_eq!(bridge.platform_name(), "linux");
}

#[test]
fn linux_bridge_does_not_support_process_id() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let bridge = LinuxExplicitProxyBridge::new(addr);
    assert!(!bridge.supports_process_identification());
}

#[tokio::test]
async fn linux_bridge_starts_and_stops() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let bridge = LinuxExplicitProxyBridge::new(addr);

    bridge.start().await.expect("bridge should start");

    let bound = bridge.bound_addr().expect("should have bound address");
    assert_ne!(bound.port(), 0, "OS should assign a real port");

    let _conn = tokio::net::TcpStream::connect(bound)
        .await
        .expect("should connect to bridge listener");

    bridge.stop().await.expect("bridge should stop");
}

#[tokio::test]
async fn linux_flow_metadata_extraction() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let bridge = LinuxExplicitProxyBridge::new(addr);

    bridge.start().await.expect("bridge should start");

    let bound = bridge.bound_addr().expect("should have bound address");
    let stream = tokio::net::TcpStream::connect(bound)
        .await
        .expect("should connect to bridge");

    let local_addr = stream.local_addr().unwrap();
    let peer_addr = stream.peer_addr().unwrap();

    let metadata = extract_client_metadata(&stream);

    assert_eq!(metadata.source_addr, peer_addr);
    assert_eq!(metadata.dest_addr, local_addr);
    assert!(metadata.process_name.is_none());
    assert!(metadata.process_id.is_none());
    assert!(metadata.bundle_id.is_none());

    bridge.stop().await.expect("bridge should stop");
}

#[tokio::test]
async fn macos_bridge_start_returns_error_on_non_macos() {
    let bridge = MacOsBridge::new();
    let result = bridge.start().await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("macOS"), "got: {msg}");
}

#[tokio::test]
async fn macos_bridge_stop_returns_error_on_non_macos() {
    let bridge = MacOsBridge::new();
    let result = bridge.stop().await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("macOS"), "got: {msg}");
}

#[test]
fn linux_bridge_bound_addr_none_before_start() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let bridge = LinuxExplicitProxyBridge::new(addr);
    assert!(bridge.bound_addr().is_none());
}

#[test]
fn linux_bridge_trait_object_polymorphism() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let bridge: Box<dyn PlatformBridge> = Box::new(LinuxExplicitProxyBridge::new(addr));
    assert_eq!(bridge.platform_name(), "linux");
    assert!(!bridge.supports_process_identification());
}
