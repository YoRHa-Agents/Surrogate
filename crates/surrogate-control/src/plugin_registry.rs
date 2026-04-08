use std::collections::HashMap;
use std::sync::Arc;

use surrogate_contract::plugin::{PluginCapability, PluginHandle};

use crate::error::ControlPlaneError;

pub struct RegisteredPlugin {
    pub name: String,
    pub handler: Arc<dyn PluginHandle>,
    pub enabled: bool,
    pub capabilities: Vec<PluginCapability>,
}

pub struct PluginRegistry {
    plugins: HashMap<String, RegisteredPlugin>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, handler: Arc<dyn PluginHandle>) {
        let capabilities = handler.capabilities().to_vec();
        self.plugins.insert(
            name.clone(),
            RegisteredPlugin {
                name,
                handler,
                enabled: true,
                capabilities,
            },
        );
    }

    pub fn enable(&mut self, name: &str) -> Result<(), ControlPlaneError> {
        let entry = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| ControlPlaneError::PluginNotFound(name.to_string()))?;
        entry.enabled = true;
        Ok(())
    }

    pub fn disable(&mut self, name: &str) -> Result<(), ControlPlaneError> {
        let entry = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| ControlPlaneError::PluginNotFound(name.to_string()))?;
        entry.enabled = false;
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&RegisteredPlugin> {
        self.plugins.get(name)
    }

    pub fn list_enabled(&self) -> Vec<&RegisteredPlugin> {
        self.plugins.values().filter(|p| p.enabled).collect()
    }

    pub fn find_by_capability(&self, cap: PluginCapability) -> Vec<&RegisteredPlugin> {
        self.plugins
            .values()
            .filter(|p| p.enabled && p.capabilities.contains(&cap))
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use surrogate_contract::plugin::{PluginAction, PluginResult};

    struct DummyPlugin {
        n: String,
        caps: Vec<PluginCapability>,
    }

    impl PluginHandle for DummyPlugin {
        fn name(&self) -> &str {
            &self.n
        }
        fn capabilities(&self) -> &[PluginCapability] {
            &self.caps
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

    fn make_dummy(name: &str, caps: Vec<PluginCapability>) -> Arc<dyn PluginHandle> {
        Arc::new(DummyPlugin {
            n: name.to_string(),
            caps,
        })
    }

    #[test]
    fn register_and_lookup() {
        let mut reg = PluginRegistry::new();
        reg.register(
            "alpha".into(),
            make_dummy("alpha", vec![PluginCapability::Diagnostic]),
        );

        let p = reg.get("alpha").unwrap();
        assert_eq!(p.name, "alpha");
        assert!(p.enabled);
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn enable_disable() {
        let mut reg = PluginRegistry::new();
        reg.register(
            "beta".into(),
            make_dummy("beta", vec![PluginCapability::RegionRisk]),
        );

        reg.disable("beta").unwrap();
        assert!(!reg.get("beta").unwrap().enabled);

        reg.enable("beta").unwrap();
        assert!(reg.get("beta").unwrap().enabled);

        assert!(reg.disable("missing").is_err());
        assert!(reg.enable("missing").is_err());
    }

    #[test]
    fn find_by_capability() {
        let mut reg = PluginRegistry::new();
        reg.register(
            "a".into(),
            make_dummy(
                "a",
                vec![
                    PluginCapability::ProxyBootstrap,
                    PluginCapability::Diagnostic,
                ],
            ),
        );
        reg.register(
            "b".into(),
            make_dummy("b", vec![PluginCapability::ProxyBootstrap]),
        );
        reg.register(
            "c".into(),
            make_dummy("c", vec![PluginCapability::Diagnostic]),
        );

        let bootstrap = reg.find_by_capability(PluginCapability::ProxyBootstrap);
        assert_eq!(bootstrap.len(), 2);

        let diag = reg.find_by_capability(PluginCapability::Diagnostic);
        assert_eq!(diag.len(), 2);

        let migration = reg.find_by_capability(PluginCapability::ConfigMigration);
        assert!(migration.is_empty());
    }

    #[test]
    fn list_enabled_filters() {
        let mut reg = PluginRegistry::new();
        reg.register(
            "x".into(),
            make_dummy("x", vec![PluginCapability::ProxyBootstrap]),
        );
        reg.register(
            "y".into(),
            make_dummy("y", vec![PluginCapability::ProxyBootstrap]),
        );
        reg.register(
            "z".into(),
            make_dummy("z", vec![PluginCapability::ProxyBootstrap]),
        );

        assert_eq!(reg.list_enabled().len(), 3);

        reg.disable("y").unwrap();
        assert_eq!(reg.list_enabled().len(), 2);
        assert!(
            reg.list_enabled()
                .iter()
                .all(|p| p.name == "x" || p.name == "z")
        );

        // disabled plugin excluded from find_by_capability too
        assert_eq!(
            reg.find_by_capability(PluginCapability::ProxyBootstrap)
                .len(),
            2
        );
    }

    #[test]
    fn register_duplicate_name_overwrites() {
        let mut reg = PluginRegistry::new();
        reg.register(
            "dup".into(),
            make_dummy("dup", vec![PluginCapability::Diagnostic]),
        );
        reg.register(
            "dup".into(),
            make_dummy("dup", vec![PluginCapability::RegionRisk]),
        );

        let p = reg.get("dup").unwrap();
        assert_eq!(p.capabilities, vec![PluginCapability::RegionRisk]);
        assert_eq!(reg.list_enabled().len(), 1);
    }

    #[test]
    fn find_by_capability_returns_all_matching() {
        let mut reg = PluginRegistry::new();
        reg.register(
            "p1".into(),
            make_dummy(
                "p1",
                vec![PluginCapability::Diagnostic, PluginCapability::RegionRisk],
            ),
        );
        reg.register(
            "p2".into(),
            make_dummy("p2", vec![PluginCapability::Diagnostic]),
        );
        reg.register(
            "p3".into(),
            make_dummy("p3", vec![PluginCapability::ProxyBootstrap]),
        );

        let diag = reg.find_by_capability(PluginCapability::Diagnostic);
        assert_eq!(diag.len(), 2);
        let names: Vec<&str> = diag.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"p1"));
        assert!(names.contains(&"p2"));
    }
}
