#[derive(Debug, thiserror::Error)]
pub enum ControlPlaneError {
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    #[error("import error: {0}")]
    ImportError(String),
    #[error("config error: {0}")]
    ConfigError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_plane_error_display() {
        let plugin_err = ControlPlaneError::PluginNotFound("test-plugin".to_string());
        assert_eq!(plugin_err.to_string(), "plugin not found: test-plugin");

        let import_err = ControlPlaneError::ImportError("bad format".to_string());
        assert_eq!(import_err.to_string(), "import error: bad format");

        let config_err = ControlPlaneError::ConfigError("invalid key".to_string());
        assert_eq!(config_err.to_string(), "config error: invalid key");
    }
}
