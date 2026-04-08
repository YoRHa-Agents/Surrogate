use std::future::Future;
use std::pin::Pin;

/// Convenience alias for plugin intercept results.
pub type PluginResult = Result<PluginAction, PluginError>;

/// Action a plugin returns after inspecting a request or response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginAction {
    /// Pass-through — do not modify the payload.
    Continue,
    /// Replace the payload with the provided bytes.
    Modify(Vec<u8>),
    /// Block the request/response with a reason string.
    Block(String),
}

/// Errors that a plugin can produce.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// General execution failure inside the plugin.
    #[error("plugin execution failed: {0}")]
    ExecutionFailed(String),
    /// The plugin did not complete within its budget.
    #[error("plugin timed out after {0}ms")]
    Timeout(u64),
}

/// Capabilities a plugin may declare to the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PluginCapability {
    /// Participate in proxy unit bootstrap.
    ProxyBootstrap,
    /// Adapt streaming protocols.
    StreamingCompatibility,
    /// Execute tasks on remote nodes.
    RemoteExecution,
    /// Evaluate geographic risk scores.
    RegionRisk,
    /// Provide diagnostic / introspection data.
    Diagnostic,
    /// Migrate configuration between schema versions.
    ConfigMigration,
}

/// Async intercept handle that the plugin host invokes for every
/// request/response flowing through the proxy.
pub trait PluginHandle: Send + Sync {
    /// Unique name of the plugin instance.
    fn name(&self) -> &str;
    /// Declared capabilities for capability-gated invocation.
    fn capabilities(&self) -> &[PluginCapability];
    /// Intercept an outbound request payload.
    fn on_request(&self, data: &[u8]) -> Pin<Box<dyn Future<Output = PluginResult> + Send + '_>>;
    /// Intercept an inbound response payload.
    fn on_response(&self, data: &[u8]) -> Pin<Box<dyn Future<Output = PluginResult> + Send + '_>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPlugin;

    impl PluginHandle for MockPlugin {
        fn name(&self) -> &str {
            "mock-plugin"
        }

        fn capabilities(&self) -> &[PluginCapability] {
            &[PluginCapability::Diagnostic, PluginCapability::RegionRisk]
        }

        fn on_request(
            &self,
            _data: &[u8],
        ) -> Pin<Box<dyn Future<Output = PluginResult> + Send + '_>> {
            Box::pin(async { Ok(PluginAction::Continue) })
        }

        fn on_response(
            &self,
            _data: &[u8],
        ) -> Pin<Box<dyn Future<Output = PluginResult> + Send + '_>> {
            Box::pin(async { Ok(PluginAction::Modify(vec![1, 2, 3])) })
        }
    }

    #[tokio::test]
    async fn mock_plugin_handle_compiles() {
        let plugin = MockPlugin;
        assert_eq!(plugin.name(), "mock-plugin");
        assert_eq!(plugin.capabilities().len(), 2);

        let request_result = plugin.on_request(b"hello").await;
        assert_eq!(request_result.unwrap(), PluginAction::Continue);

        let response_result = plugin.on_response(b"world").await;
        assert_eq!(
            response_result.unwrap(),
            PluginAction::Modify(vec![1, 2, 3])
        );
    }

    #[test]
    fn plugin_action_variants() {
        let cont = PluginAction::Continue;
        let modify = PluginAction::Modify(vec![42]);
        let block = PluginAction::Block("denied".to_string());

        assert_ne!(cont, modify);
        assert_ne!(modify, block);
        assert_ne!(cont, block);
    }

    #[test]
    fn plugin_error_display() {
        let exec = PluginError::ExecutionFailed("segfault".to_string());
        assert_eq!(format!("{exec}"), "plugin execution failed: segfault");

        let timeout = PluginError::Timeout(500);
        assert_eq!(format!("{timeout}"), "plugin timed out after 500ms");
    }

    #[test]
    fn plugin_capability_serde_roundtrip() {
        let capabilities = vec![
            PluginCapability::ProxyBootstrap,
            PluginCapability::StreamingCompatibility,
            PluginCapability::RemoteExecution,
            PluginCapability::RegionRisk,
            PluginCapability::Diagnostic,
            PluginCapability::ConfigMigration,
        ];
        let json = serde_json::to_string(&capabilities).expect("serialize");
        let deserialized: Vec<PluginCapability> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(capabilities, deserialized);
    }
}
