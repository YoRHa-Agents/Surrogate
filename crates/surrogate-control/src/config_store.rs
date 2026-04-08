use surrogate_contract::config::NormalizedConfig;

/// Versioned configuration store with snapshot and rollback.
pub struct ConfigStore {
    current: Option<VersionedConfig>,
    history: Vec<VersionedConfig>,
    max_history: usize,
}

#[derive(Debug, Clone)]
pub struct VersionedConfig {
    pub version: u64,
    pub config: NormalizedConfig,
    pub created_at: std::time::SystemTime,
}

impl ConfigStore {
    pub fn new(max_history: usize) -> Self {
        Self {
            current: None,
            history: Vec::new(),
            max_history,
        }
    }

    /// Apply a new config, archiving the previous one. Returns the new version number.
    pub fn apply(&mut self, config: NormalizedConfig) -> u64 {
        let version = self.current.as_ref().map_or(1, |c| c.version + 1);

        if let Some(prev) = self.current.take() {
            self.history.push(prev);
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }

        self.current = Some(VersionedConfig {
            version,
            config,
            created_at: std::time::SystemTime::now(),
        });

        version
    }

    pub fn current(&self) -> Option<&VersionedConfig> {
        self.current.as_ref()
    }

    /// Roll back to the most recent historical config. Returns the restored version.
    pub fn rollback(&mut self) -> Result<u64, ConfigStoreError> {
        let prev = self.history.pop().ok_or(ConfigStoreError::NoHistory)?;
        self.current = Some(prev);
        Ok(self.current.as_ref().unwrap().version)
    }

    pub fn history(&self) -> &[VersionedConfig] {
        &self.history
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigStoreError {
    #[error("no previous version to rollback to")]
    NoHistory,
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrogate_contract::config::{NormalizedConfig, NormalizedOutbound, OutboundKind};

    fn make_config(label: &str) -> NormalizedConfig {
        NormalizedConfig {
            listen_addr: "127.0.0.1:41080".to_string(),
            default_outbound: label.to_string(),
            outbounds: vec![NormalizedOutbound {
                id: label.to_string(),
                kind: OutboundKind::Direct,
            }],
            rules: vec![],
        }
    }

    #[test]
    fn apply_and_current() {
        let mut store = ConfigStore::new(10);
        assert!(store.current().is_none());

        let v = store.apply(make_config("direct"));
        assert_eq!(v, 1);
        assert_eq!(store.current().unwrap().version, 1);
        assert_eq!(store.current().unwrap().config.default_outbound, "direct");

        let v2 = store.apply(make_config("reject"));
        assert_eq!(v2, 2);
        assert_eq!(store.current().unwrap().config.default_outbound, "reject");
    }

    #[test]
    fn rollback() {
        let mut store = ConfigStore::new(10);
        store.apply(make_config("v1"));
        store.apply(make_config("v2"));

        let restored = store.rollback().unwrap();
        assert_eq!(restored, 1);
        assert_eq!(store.current().unwrap().config.default_outbound, "v1");

        assert!(store.rollback().is_err());
    }

    #[test]
    fn max_history_trim() {
        let mut store = ConfigStore::new(2);
        store.apply(make_config("a"));
        store.apply(make_config("b"));
        store.apply(make_config("c"));
        store.apply(make_config("d"));

        assert_eq!(store.history().len(), 2);
        assert_eq!(store.history()[0].config.default_outbound, "b");
        assert_eq!(store.history()[1].config.default_outbound, "c");
    }

    #[test]
    fn apply_multiple_then_rollback_chain() {
        let mut store = ConfigStore::new(10);
        store.apply(make_config("v1"));
        store.apply(make_config("v2"));
        store.apply(make_config("v3"));

        let restored = store.rollback().unwrap();
        assert_eq!(restored, 2);
        assert_eq!(store.current().unwrap().config.default_outbound, "v2");

        let restored2 = store.rollback().unwrap();
        assert_eq!(restored2, 1);
        assert_eq!(store.current().unwrap().config.default_outbound, "v1");
    }

    #[test]
    fn rollback_empty_store_fails() {
        let mut store = ConfigStore::new(10);
        assert!(store.rollback().is_err());
    }

    #[test]
    fn apply_after_rollback_increments_correctly() {
        let mut store = ConfigStore::new(10);
        store.apply(make_config("v1"));
        store.apply(make_config("v2"));
        store.rollback().unwrap();
        let v = store.apply(make_config("v3"));
        assert_eq!(v, 2);
        assert_eq!(store.current().unwrap().config.default_outbound, "v3");
    }
}
