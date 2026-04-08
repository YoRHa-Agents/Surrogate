use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::ControlPlaneError;

// ---------------------------------------------------------------------------
// Neutral IR
// ---------------------------------------------------------------------------

/// Format-neutral intermediate representation for proxy configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeutralIR {
    pub nodes: Vec<NodeEntry>,
    pub routing_rules: Vec<RoutingEntry>,
    pub proxy_groups: Vec<ProxyGroupEntry>,
    pub metadata: ImportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEntry {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub server: String,
    pub port: u16,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingEntry {
    pub id: String,
    pub matcher: String,
    pub outbound: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroupEntry {
    pub id: String,
    pub name: String,
    pub group_type: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMetadata {
    pub source_format: String,
    pub source_version: Option<String>,
}

// ---------------------------------------------------------------------------
// ImportReport
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct ImportReport {
    pub source: String,
    pub nodes_imported: usize,
    pub nodes_degraded: usize,
    pub nodes_ignored: usize,
    pub warnings: Vec<String>,
    pub health_score: f64,
}

// ---------------------------------------------------------------------------
// ImportSource trait
// ---------------------------------------------------------------------------

pub trait ImportSource: Send + Sync {
    fn name(&self) -> &str;
    fn parse(&self, content: &str) -> Result<NeutralIR, ControlPlaneError>;
}

// ---------------------------------------------------------------------------
// Stub parsers
// ---------------------------------------------------------------------------

pub struct ClashParser;

impl ImportSource for ClashParser {
    fn name(&self) -> &str {
        "clash"
    }

    fn parse(&self, content: &str) -> Result<NeutralIR, ControlPlaneError> {
        let ir: NeutralIR = serde_json::from_str(content)
            .map_err(|e| ControlPlaneError::ImportError(format!("clash parse: {e}")))?;
        Ok(ir)
    }
}

pub struct SingBoxParser;

impl ImportSource for SingBoxParser {
    fn name(&self) -> &str {
        "sing-box"
    }

    fn parse(&self, content: &str) -> Result<NeutralIR, ControlPlaneError> {
        let ir: NeutralIR = serde_json::from_str(content)
            .map_err(|e| ControlPlaneError::ImportError(format!("sing-box parse: {e}")))?;
        Ok(ir)
    }
}

pub struct V2RayParser;

impl ImportSource for V2RayParser {
    fn name(&self) -> &str {
        "v2ray"
    }

    fn parse(&self, content: &str) -> Result<NeutralIR, ControlPlaneError> {
        let ir: NeutralIR = serde_json::from_str(content)
            .map_err(|e| ControlPlaneError::ImportError(format!("v2ray parse: {e}")))?;
        Ok(ir)
    }
}

// ---------------------------------------------------------------------------
// ImportEngine
// ---------------------------------------------------------------------------

pub struct ImportEngine {
    sources: Vec<Box<dyn ImportSource>>,
}

impl ImportEngine {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn register_source(&mut self, source: Box<dyn ImportSource>) {
        self.sources.push(source);
    }

    pub fn import(
        &self,
        source_name: &str,
        content: &str,
    ) -> Result<(NeutralIR, ImportReport), ControlPlaneError> {
        let source = self
            .sources
            .iter()
            .find(|s| s.name() == source_name)
            .ok_or_else(|| {
                ControlPlaneError::ImportError(format!("unknown source: {source_name}"))
            })?;

        let ir = source.parse(content)?;

        let report = ImportReport {
            source: source_name.to_string(),
            nodes_imported: ir.nodes.len(),
            nodes_degraded: 0,
            nodes_ignored: 0,
            warnings: Vec::new(),
            health_score: if ir.nodes.is_empty() { 0.0 } else { 1.0 },
        };

        Ok((ir, report))
    }
}

impl Default for ImportEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ir_json() -> String {
        serde_json::json!({
            "nodes": [{
                "id": "n1",
                "name": "Tokyo-SS",
                "protocol": "shadowsocks",
                "server": "1.2.3.4",
                "port": 8388,
                "extra": {"method": "aes-256-gcm"}
            }],
            "routing_rules": [{
                "id": "r1",
                "matcher": "DOMAIN-SUFFIX,google.com",
                "outbound": "proxy"
            }],
            "proxy_groups": [{
                "id": "g1",
                "name": "auto-select",
                "group_type": "url-test",
                "members": ["n1"]
            }],
            "metadata": {
                "source_format": "clash",
                "source_version": "1.0"
            }
        })
        .to_string()
    }

    fn make_engine() -> ImportEngine {
        let mut engine = ImportEngine::new();
        engine.register_source(Box::new(ClashParser));
        engine.register_source(Box::new(SingBoxParser));
        engine.register_source(Box::new(V2RayParser));
        engine
    }

    #[test]
    fn register_and_import_clash() {
        let engine = make_engine();
        let json = sample_ir_json();
        let (ir, _report) = engine.import("clash", &json).unwrap();

        assert_eq!(ir.nodes.len(), 1);
        assert_eq!(ir.nodes[0].name, "Tokyo-SS");
        assert_eq!(ir.routing_rules.len(), 1);
        assert_eq!(ir.proxy_groups.len(), 1);
        assert_eq!(ir.metadata.source_format, "clash");
    }

    #[test]
    fn import_report_generated() {
        let engine = make_engine();
        let json = sample_ir_json();
        let (_ir, report) = engine.import("sing-box", &json).unwrap();

        assert_eq!(report.source, "sing-box");
        assert_eq!(report.nodes_imported, 1);
        assert_eq!(report.nodes_degraded, 0);
        assert_eq!(report.nodes_ignored, 0);
        assert!(report.warnings.is_empty());
        assert!((report.health_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn unknown_source_errors() {
        let engine = make_engine();
        let result = engine.import("unknown-format", "{}");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown source"), "got: {msg}");
    }

    #[test]
    fn import_invalid_json_returns_error() {
        let engine = make_engine();
        let result = engine.import("clash", "not valid json at all{{{");
        assert!(result.is_err());
        match result.unwrap_err() {
            ControlPlaneError::ImportError(msg) => {
                assert!(msg.contains("clash parse"), "got: {msg}");
            }
            other => panic!("expected ImportError, got: {other:?}"),
        }
    }

    #[test]
    fn import_empty_nodes_zero_health() {
        let json = serde_json::json!({
            "nodes": [],
            "routing_rules": [],
            "proxy_groups": [],
            "metadata": {
                "source_format": "clash",
                "source_version": null
            }
        })
        .to_string();

        let engine = make_engine();
        let (_ir, report) = engine.import("clash", &json).unwrap();
        assert!((report.health_score - 0.0).abs() < f64::EPSILON);
        assert_eq!(report.nodes_imported, 0);
    }

    #[test]
    fn import_all_three_parsers_succeed() {
        let engine = make_engine();
        let json = sample_ir_json();
        for name in &["clash", "sing-box", "v2ray"] {
            let (ir, report) = engine.import(name, &json).unwrap();
            assert_eq!(ir.nodes.len(), 1);
            assert_eq!(report.source, *name);
        }
    }

    #[test]
    fn import_engine_no_sources_registered() {
        let engine = ImportEngine::new();
        let result = engine.import("clash", "{}");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("unknown source"), "got: {msg}");
    }
}
