use std::future::Future;
use std::pin::Pin;

use surrogate_contract::plugin::{PluginAction, PluginCapability, PluginHandle, PluginResult};

macro_rules! stub_plugin {
    ($name:ident, $display:expr, $caps:expr) => {
        pub struct $name;

        impl PluginHandle for $name {
            fn name(&self) -> &str {
                $display
            }

            fn capabilities(&self) -> &[PluginCapability] {
                &$caps
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
                Box::pin(async { Ok(PluginAction::Continue) })
            }
        }
    };
}

stub_plugin!(
    ClaudeCodePlugin,
    "claude-code",
    [
        PluginCapability::ProxyBootstrap,
        PluginCapability::StreamingCompatibility,
        PluginCapability::RemoteExecution,
    ]
);

stub_plugin!(
    CursorPlugin,
    "cursor",
    [
        PluginCapability::ProxyBootstrap,
        PluginCapability::StreamingCompatibility,
    ]
);

stub_plugin!(
    CodexPlugin,
    "codex",
    [
        PluginCapability::ProxyBootstrap,
        PluginCapability::RemoteExecution,
    ]
);

stub_plugin!(
    CopilotPlugin,
    "copilot",
    [
        PluginCapability::ProxyBootstrap,
        PluginCapability::StreamingCompatibility,
    ]
);

stub_plugin!(
    GeminiPlugin,
    "gemini",
    [
        PluginCapability::ProxyBootstrap,
        PluginCapability::RegionRisk,
    ]
);

stub_plugin!(
    RemoteServerModePlugin,
    "remote-server-mode",
    [
        PluginCapability::RemoteExecution,
        PluginCapability::Diagnostic,
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_registry::PluginRegistry;
    use std::sync::Arc;
    use std::task::{Context, Wake, Waker};

    struct NoopWake;
    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    #[test]
    fn all_six_plugins_register() {
        let mut registry = PluginRegistry::new();
        let plugins: Vec<Arc<dyn PluginHandle>> = vec![
            Arc::new(ClaudeCodePlugin),
            Arc::new(CursorPlugin),
            Arc::new(CodexPlugin),
            Arc::new(CopilotPlugin),
            Arc::new(GeminiPlugin),
            Arc::new(RemoteServerModePlugin),
        ];

        for p in &plugins {
            registry.register(p.name().to_string(), Arc::clone(p));
        }

        assert_eq!(registry.list_enabled().len(), 6);

        let expected_names = [
            "claude-code",
            "cursor",
            "codex",
            "copilot",
            "gemini",
            "remote-server-mode",
        ];
        for name in &expected_names {
            assert!(registry.get(name).is_some(), "missing plugin: {name}");
        }
    }

    #[test]
    fn builtin_plugin_on_request_returns_continue() {
        let waker = Waker::from(Arc::new(NoopWake));
        let mut cx = Context::from_waker(&waker);
        let plugin = ClaudeCodePlugin;
        let mut fut = plugin.on_request(b"test data");
        match fut.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(result) => {
                assert_eq!(result.unwrap(), PluginAction::Continue);
            }
            std::task::Poll::Pending => panic!("expected future to be immediately ready"),
        }
    }

    #[test]
    fn builtin_capabilities_are_distinct() {
        let plugins: Vec<Box<dyn PluginHandle>> = vec![
            Box::new(ClaudeCodePlugin),
            Box::new(CursorPlugin),
            Box::new(CodexPlugin),
            Box::new(CopilotPlugin),
            Box::new(GeminiPlugin),
            Box::new(RemoteServerModePlugin),
        ];

        let mut names = std::collections::HashSet::new();
        for p in &plugins {
            assert!(
                !p.capabilities().is_empty(),
                "plugin {} has empty capabilities",
                p.name()
            );
            assert!(
                names.insert(p.name().to_string()),
                "duplicate name: {}",
                p.name()
            );
        }
        assert_eq!(names.len(), 6);
    }
}
