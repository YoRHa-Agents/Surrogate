use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use surrogate_contract::error::BridgeError;

/// Metadata about a captured traffic flow.
#[derive(Debug, Clone)]
pub struct FlowMetadata {
    pub source_addr: SocketAddr,
    pub dest_addr: SocketAddr,
    pub process_name: Option<String>,
    pub process_id: Option<u32>,
    pub bundle_id: Option<String>,
}

/// Platform-agnostic bridge trait for traffic ingress.
pub trait PlatformBridge: Send + Sync {
    /// Start the bridge, begin capturing traffic.
    fn start(&self) -> Pin<Box<dyn Future<Output = Result<(), BridgeError>> + Send + '_>>;
    /// Stop the bridge gracefully.
    fn stop(&self) -> Pin<Box<dyn Future<Output = Result<(), BridgeError>> + Send + '_>>;
    /// Platform identifier.
    fn platform_name(&self) -> &str;
    /// Whether this bridge can identify source processes.
    fn supports_process_identification(&self) -> bool;
}
