use thiserror::Error;

/// Errors originating from the contract / schema layer.
#[derive(Debug, Error)]
pub enum ContractError {
    /// A configuration value failed schema validation.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    /// The persisted schema version does not match the expected version.
    #[error("schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersionMismatch {
        /// Version the code expects.
        expected: String,
        /// Version found on disk or in the database.
        actual: String,
    },
    /// A serialization or deserialization round-trip failed.
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Errors originating from the control plane.
#[derive(Debug, Error)]
pub enum ControlError {
    /// Persistent config store read/write failure.
    #[error("config store error: {0}")]
    ConfigStore(String),
    /// A rule set could not be compiled into a matcher tree.
    #[error("rule compilation failed: {0}")]
    RuleCompilation(String),
    /// A plugin lifecycle or execution error.
    #[error("plugin error: {0}")]
    Plugin(String),
    /// Subscription or config import failure.
    #[error("import engine error: {0}")]
    ImportEngine(String),
    /// Schema migration could not be applied.
    #[error("migration error: {0}")]
    Migration(String),
}

/// Errors originating from the platform bridge layer.
#[derive(Debug, Error)]
pub enum BridgeError {
    /// A platform-specific API call failed.
    #[error("platform API error: {0}")]
    PlatformApi(String),
    /// Traffic injection into the network stack failed.
    #[error("traffic injection failed: {0}")]
    TrafficInjection(String),
    /// Could not identify the originating process for a connection.
    #[error("process identification failed: {0}")]
    ProcessIdentification(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_error_display() {
        let mismatch = ContractError::SchemaVersionMismatch {
            expected: "2.0".to_string(),
            actual: "1.5".to_string(),
        };
        let msg = format!("{mismatch}");
        assert!(msg.contains("2.0"), "should contain expected version");
        assert!(msg.contains("1.5"), "should contain actual version");

        let invalid = ContractError::InvalidConfig("bad value".to_string());
        assert!(format!("{invalid}").contains("bad value"));

        let ser = ContractError::Serialization("broken json".to_string());
        assert!(format!("{ser}").contains("broken json"));
    }

    #[test]
    fn control_error_display() {
        let cases: Vec<(ControlError, &str)> = vec![
            (ControlError::ConfigStore("db down".to_string()), "db down"),
            (
                ControlError::RuleCompilation("bad regex".to_string()),
                "bad regex",
            ),
            (ControlError::Plugin("crashed".to_string()), "crashed"),
            (ControlError::ImportEngine("timeout".to_string()), "timeout"),
            (
                ControlError::Migration("schema v3 missing".to_string()),
                "schema v3 missing",
            ),
        ];
        for (err, expected_substr) in &cases {
            let msg = format!("{err}");
            assert!(
                msg.contains(expected_substr),
                "expected '{expected_substr}' in '{msg}'"
            );
        }
    }

    #[test]
    fn bridge_error_display() {
        let cases: Vec<(BridgeError, &str)> = vec![
            (
                BridgeError::PlatformApi("not supported".to_string()),
                "not supported",
            ),
            (
                BridgeError::TrafficInjection("tun failed".to_string()),
                "tun failed",
            ),
            (
                BridgeError::ProcessIdentification("pid unknown".to_string()),
                "pid unknown",
            ),
        ];
        for (err, expected_substr) in &cases {
            let msg = format!("{err}");
            assert!(
                msg.contains(expected_substr),
                "expected '{expected_substr}' in '{msg}'"
            );
        }
    }
}
