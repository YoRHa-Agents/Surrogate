use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::Path;
use thiserror::Error;

/// Top-level configuration document as parsed from TOML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigDocument {
    /// Socket address string the proxy listens on (e.g. `"127.0.0.1:41080"`).
    pub listen: String,
    /// Identifier of the outbound used when no rule matches.
    pub default_outbound: String,
    /// Declared outbound transports.
    #[serde(default)]
    pub outbounds: Vec<OutboundConfig>,
    /// Ordered routing rules.
    #[serde(default)]
    pub rules: Vec<RouteRuleConfig>,
}

/// Configuration for a single outbound transport.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutboundConfig {
    /// Unique outbound identifier.
    pub id: String,
    /// Transport kind (direct, reject, …).
    #[serde(rename = "type")]
    pub kind: OutboundKind,
}

/// Supported outbound transport types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutboundKind {
    /// Forward traffic directly to the destination.
    Direct,
    /// Drop the connection immediately.
    Reject,
}

/// A single route rule entry in the raw configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteRuleConfig {
    /// Unique rule identifier.
    pub id: String,
    /// Exact hostname match, if set.
    #[serde(default)]
    pub host_equals: Option<String>,
    /// Hostname suffix match, if set.
    #[serde(default)]
    pub host_suffix: Option<String>,
    /// Destination port match, if set.
    #[serde(default)]
    pub port: Option<u16>,
    /// Outbound this rule routes to.
    pub outbound: String,
}

/// Validated and normalized configuration ready for the kernel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedConfig {
    /// Canonical listen address.
    pub listen_addr: String,
    /// Lowercased default outbound id.
    pub default_outbound: String,
    /// Sorted, normalized outbounds.
    pub outbounds: Vec<NormalizedOutbound>,
    /// Priority-assigned normalized rules.
    pub rules: Vec<NormalizedRule>,
}

/// Outbound entry after normalization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedOutbound {
    /// Lowercased outbound identifier.
    pub id: String,
    /// Transport kind.
    pub kind: OutboundKind,
}

/// Route rule entry after normalization with an assigned priority.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedRule {
    /// 1-based priority (lower = higher priority).
    pub priority: u32,
    /// Lowercased rule identifier.
    pub id: String,
    /// Lowercased exact hostname, if set.
    pub host_equals: Option<String>,
    /// Lowercased hostname suffix, if set.
    pub host_suffix: Option<String>,
    /// Destination port, if set.
    pub port: Option<u16>,
    /// Lowercased target outbound identifier.
    pub outbound: String,
}

/// Errors produced during config loading, parsing, and validation.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Could not read the config file from disk.
    #[error("failed to read config at `{path}`: {source}")]
    Read {
        /// Filesystem path that was attempted.
        path: String,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// TOML parsing failed.
    #[error("failed to parse TOML config: {0}")]
    Parse(#[from] toml::de::Error),
    /// JSON serialization of the normalized config failed.
    #[error("failed to serialize normalized config: {0}")]
    Serialize(#[from] serde_json::Error),
    /// The listen address is not a valid `SocketAddr`.
    #[error("listen address `{0}` is not a valid socket address")]
    InvalidListen(String),
    /// No outbounds were declared.
    #[error("at least one outbound is required")]
    MissingOutbounds,
    /// The default outbound field is blank.
    #[error("default outbound id cannot be empty")]
    EmptyDefaultOutbound,
    /// Two outbounds share the same id.
    #[error("duplicate outbound id `{0}`")]
    DuplicateOutboundId(String),
    /// The default outbound id does not match any declared outbound.
    #[error("default outbound `{0}` does not exist")]
    UnknownDefaultOutbound(String),
    /// Two rules share the same id.
    #[error("duplicate rule id `{0}`")]
    DuplicateRuleId(String),
    /// A rule has a blank id.
    #[error("rule id cannot be empty")]
    EmptyRuleId,
    /// A rule has no matchers (host, suffix, or port).
    #[error("rule `{0}` must contain at least one matcher")]
    EmptyRule(String),
    /// A rule references an outbound that does not exist.
    #[error("rule `{rule_id}` references unknown outbound `{outbound_id}`")]
    UnknownRuleOutbound {
        /// The rule whose outbound reference is invalid.
        rule_id: String,
        /// The outbound id that was not found.
        outbound_id: String,
    },
}

/// Load a TOML config file from `path`, parse it, and validate.
pub fn load_and_validate(path: &Path) -> Result<ConfigDocument, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.display().to_string(),
        source,
    })?;
    let document: ConfigDocument = toml::from_str(&content)?;
    validate(&document)?;
    Ok(document)
}

/// Validate a parsed config document for structural correctness.
pub fn validate(document: &ConfigDocument) -> Result<(), ConfigError> {
    let listen = document.listen.trim();
    if listen.parse::<SocketAddr>().is_err() {
        return Err(ConfigError::InvalidListen(document.listen.clone()));
    }

    let default_outbound = normalize_id(&document.default_outbound);
    if default_outbound.is_empty() {
        return Err(ConfigError::EmptyDefaultOutbound);
    }

    if document.outbounds.is_empty() {
        return Err(ConfigError::MissingOutbounds);
    }

    let mut outbound_ids = HashSet::new();
    for outbound in &document.outbounds {
        let outbound_id = normalize_id(&outbound.id);
        if !outbound_ids.insert(outbound_id.clone()) {
            return Err(ConfigError::DuplicateOutboundId(outbound.id.clone()));
        }
    }

    if !outbound_ids.contains(&default_outbound) {
        return Err(ConfigError::UnknownDefaultOutbound(
            document.default_outbound.clone(),
        ));
    }

    let mut rule_ids = HashSet::new();
    for rule in &document.rules {
        let rule_id = normalize_id(&rule.id);
        if rule_id.is_empty() {
            return Err(ConfigError::EmptyRuleId);
        }

        if !rule_ids.insert(rule_id.clone()) {
            return Err(ConfigError::DuplicateRuleId(rule.id.clone()));
        }

        if rule.host_equals.is_none() && rule.host_suffix.is_none() && rule.port.is_none() {
            return Err(ConfigError::EmptyRule(rule.id.clone()));
        }

        let outbound_id = normalize_id(&rule.outbound);
        if !outbound_ids.contains(&outbound_id) {
            return Err(ConfigError::UnknownRuleOutbound {
                rule_id: rule.id.clone(),
                outbound_id: rule.outbound.clone(),
            });
        }
    }

    Ok(())
}

/// Produce a [`NormalizedConfig`] from a validated document.
pub fn normalize(document: &ConfigDocument) -> NormalizedConfig {
    let mut outbounds = document
        .outbounds
        .iter()
        .map(|outbound| NormalizedOutbound {
            id: normalize_id(&outbound.id),
            kind: outbound.kind,
        })
        .collect::<Vec<_>>();
    outbounds.sort_by(|left, right| left.id.cmp(&right.id));

    let rules = document
        .rules
        .iter()
        .enumerate()
        .map(|(index, rule)| NormalizedRule {
            priority: index as u32 + 1,
            id: normalize_id(&rule.id),
            host_equals: rule.host_equals.as_ref().map(|value| normalize_host(value)),
            host_suffix: rule.host_suffix.as_ref().map(|value| normalize_host(value)),
            port: rule.port,
            outbound: normalize_id(&rule.outbound),
        })
        .collect();

    NormalizedConfig {
        listen_addr: document.listen.trim().to_string(),
        default_outbound: normalize_id(&document.default_outbound),
        outbounds,
        rules,
    }
}

/// Serialize a normalized config to pretty-printed JSON.
pub fn serialize_normalized(normalized: &NormalizedConfig) -> Result<String, ConfigError> {
    Ok(serde_json::to_string_pretty(normalized)?)
}

fn normalize_id(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_host(value: &str) -> String {
    value.trim().trim_start_matches('.').to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_validates_and_normalizes_config() {
        let path = unique_test_path("surrogate_config_round_trip");
        let config = r#"
listen = "127.0.0.1:41080"
default_outbound = "DIRECT"

[[outbounds]]
id = "direct"
type = "direct"

[[outbounds]]
id = "reject"
type = "reject"

[[rules]]
id = "match-local"
host_equals = "LOCALHOST"
port = 8080
outbound = "direct"
"#;

        std::fs::write(&path, config).expect("write config fixture");

        let document = load_and_validate(&path).expect("load config");
        let normalized = normalize(&document);
        let serialized = serialize_normalized(&normalized).expect("serialize normalized config");

        assert_eq!(normalized.default_outbound, "direct");
        assert_eq!(normalized.rules.len(), 1);
        assert_eq!(
            normalized.rules[0].host_equals.as_deref(),
            Some("localhost")
        );
        assert!(serialized.contains("\"listen_addr\": \"127.0.0.1:41080\""));
        assert!(serialized.contains("\"priority\": 1"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_empty_rules() {
        let document = ConfigDocument {
            listen: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![OutboundConfig {
                id: "direct".to_string(),
                kind: OutboundKind::Direct,
            }],
            rules: vec![RouteRuleConfig {
                id: "missing-matchers".to_string(),
                host_equals: None,
                host_suffix: None,
                port: None,
                outbound: "direct".to_string(),
            }],
        };

        let error = validate(&document).expect_err("empty rules must fail");
        assert!(matches!(error, ConfigError::EmptyRule(_)));
    }

    fn unique_test_path(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}.toml"))
    }

    fn base_document() -> ConfigDocument {
        ConfigDocument {
            listen: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![OutboundConfig {
                id: "direct".to_string(),
                kind: OutboundKind::Direct,
            }],
            rules: vec![],
        }
    }

    #[test]
    fn validate_duplicate_outbound_id() {
        let mut doc = base_document();
        doc.outbounds.push(OutboundConfig {
            id: "direct".to_string(),
            kind: OutboundKind::Reject,
        });
        let err = validate(&doc).expect_err("duplicate outbound should fail");
        assert!(matches!(err, ConfigError::DuplicateOutboundId(_)));
    }

    #[test]
    fn validate_unknown_default_outbound() {
        let doc = ConfigDocument {
            listen: "127.0.0.1:0".to_string(),
            default_outbound: "nonexistent".to_string(),
            outbounds: vec![OutboundConfig {
                id: "direct".to_string(),
                kind: OutboundKind::Direct,
            }],
            rules: vec![],
        };
        let err = validate(&doc).expect_err("unknown default outbound should fail");
        assert!(matches!(err, ConfigError::UnknownDefaultOutbound(_)));
    }

    #[test]
    fn validate_invalid_listen_address() {
        let mut doc = base_document();
        doc.listen = "not_a_socket".to_string();
        let err = validate(&doc).expect_err("invalid listen should fail");
        assert!(matches!(err, ConfigError::InvalidListen(_)));
    }

    #[test]
    fn validate_empty_default_outbound() {
        let doc = ConfigDocument {
            listen: "127.0.0.1:0".to_string(),
            default_outbound: "  ".to_string(),
            outbounds: vec![OutboundConfig {
                id: "direct".to_string(),
                kind: OutboundKind::Direct,
            }],
            rules: vec![],
        };
        let err = validate(&doc).expect_err("empty default outbound should fail");
        assert!(matches!(err, ConfigError::EmptyDefaultOutbound));
    }

    #[test]
    fn validate_missing_outbounds() {
        let doc = ConfigDocument {
            listen: "127.0.0.1:0".to_string(),
            default_outbound: "direct".to_string(),
            outbounds: vec![],
            rules: vec![],
        };
        let err = validate(&doc).expect_err("missing outbounds should fail");
        assert!(matches!(err, ConfigError::MissingOutbounds));
    }

    #[test]
    fn validate_duplicate_rule_id() {
        let mut doc = base_document();
        doc.rules = vec![
            RouteRuleConfig {
                id: "rule1".to_string(),
                host_equals: Some("a.com".to_string()),
                host_suffix: None,
                port: None,
                outbound: "direct".to_string(),
            },
            RouteRuleConfig {
                id: "rule1".to_string(),
                host_equals: Some("b.com".to_string()),
                host_suffix: None,
                port: None,
                outbound: "direct".to_string(),
            },
        ];
        let err = validate(&doc).expect_err("duplicate rule id should fail");
        assert!(matches!(err, ConfigError::DuplicateRuleId(_)));
    }

    #[test]
    fn validate_empty_rule_id() {
        let mut doc = base_document();
        doc.rules = vec![RouteRuleConfig {
            id: "".to_string(),
            host_equals: Some("a.com".to_string()),
            host_suffix: None,
            port: None,
            outbound: "direct".to_string(),
        }];
        let err = validate(&doc).expect_err("empty rule id should fail");
        assert!(matches!(err, ConfigError::EmptyRuleId));
    }

    #[test]
    fn validate_unknown_rule_outbound() {
        let mut doc = base_document();
        doc.rules = vec![RouteRuleConfig {
            id: "rule1".to_string(),
            host_equals: Some("a.com".to_string()),
            host_suffix: None,
            port: None,
            outbound: "nonexistent".to_string(),
        }];
        let err = validate(&doc).expect_err("unknown rule outbound should fail");
        assert!(matches!(err, ConfigError::UnknownRuleOutbound { .. }));
    }

    #[test]
    fn normalize_lowercases_and_sorts() {
        let doc = ConfigDocument {
            listen: "127.0.0.1:8080".to_string(),
            default_outbound: "DIRECT".to_string(),
            outbounds: vec![
                OutboundConfig {
                    id: "Reject".to_string(),
                    kind: OutboundKind::Reject,
                },
                OutboundConfig {
                    id: "DIRECT".to_string(),
                    kind: OutboundKind::Direct,
                },
            ],
            rules: vec![RouteRuleConfig {
                id: "MyRule".to_string(),
                host_equals: Some("EXAMPLE.COM".to_string()),
                host_suffix: Some(".Google.Com".to_string()),
                port: Some(443),
                outbound: "direct".to_string(),
            }],
        };
        let normalized = normalize(&doc);

        assert_eq!(normalized.default_outbound, "direct");
        assert_eq!(normalized.outbounds[0].id, "direct");
        assert_eq!(normalized.outbounds[1].id, "reject");
        assert_eq!(normalized.rules[0].id, "myrule");
        assert_eq!(
            normalized.rules[0].host_equals.as_deref(),
            Some("example.com")
        );
        assert_eq!(
            normalized.rules[0].host_suffix.as_deref(),
            Some("google.com")
        );
        assert_eq!(normalized.rules[0].port, Some(443));
    }
}
