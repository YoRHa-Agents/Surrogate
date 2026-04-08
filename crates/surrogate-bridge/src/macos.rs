use crate::bridge_trait::PlatformBridge;
use std::future::Future;
use std::pin::Pin;
use surrogate_contract::error::BridgeError;

/// macOS bridge backed by NETransparentProxyProvider / NEPacketTunnelProvider.
///
/// On non-macOS platforms every method returns a stub error because the
/// Network Extension APIs are unavailable.
pub struct MacOsBridge;

impl MacOsBridge {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOsBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformBridge for MacOsBridge {
    fn start(&self) -> Pin<Box<dyn Future<Output = Result<(), BridgeError>> + Send + '_>> {
        Box::pin(async {
            Err(BridgeError::PlatformApi(
                "macOS NE APIs not available on this platform".into(),
            ))
        })
    }

    fn stop(&self) -> Pin<Box<dyn Future<Output = Result<(), BridgeError>> + Send + '_>> {
        Box::pin(async {
            Err(BridgeError::PlatformApi(
                "macOS NE APIs not available on this platform".into(),
            ))
        })
    }

    fn platform_name(&self) -> &str {
        "macos"
    }

    fn supports_process_identification(&self) -> bool {
        true
    }
}
