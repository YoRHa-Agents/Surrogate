use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// DomainError
// ---------------------------------------------------------------------------

/// Errors related to domain model invariants.
#[derive(Debug, Error)]
pub enum DomainError {
    /// An illegal state-machine transition was attempted.
    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        /// State the unit was in before the attempted transition.
        from: ProxyUnitState,
        /// State that was requested.
        to: ProxyUnitState,
    },
}

// ---------------------------------------------------------------------------
// ProxyUnitState
// ---------------------------------------------------------------------------

/// State machine states for a `ProxyUnit`.
///
/// Valid transitions:
///   Configuring -> Active, Active -> Degraded, Degraded -> Active,
///   Active -> Inactive, Degraded -> Inactive, Inactive -> Configuring.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProxyUnitState {
    /// The unit is loading configuration and not yet available.
    Configuring,
    /// The unit is fully operational.
    Active,
    /// The unit is partially operational (e.g. high latency).
    Degraded,
    /// The unit has been shut down or disabled.
    Inactive,
}

impl ProxyUnitState {
    fn can_transition_to(self, to: ProxyUnitState) -> bool {
        matches!(
            (self, to),
            (ProxyUnitState::Configuring, ProxyUnitState::Active)
                | (ProxyUnitState::Active, ProxyUnitState::Degraded)
                | (ProxyUnitState::Degraded, ProxyUnitState::Active)
                | (ProxyUnitState::Active, ProxyUnitState::Inactive)
                | (ProxyUnitState::Degraded, ProxyUnitState::Inactive)
                | (ProxyUnitState::Inactive, ProxyUnitState::Configuring)
        )
    }
}

// ---------------------------------------------------------------------------
// ProxyProfile
// ---------------------------------------------------------------------------

/// User-facing proxy configuration: name, protocol parameters, and optional
/// subscription source for upstream node lists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProxyProfile {
    /// Unique profile identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Protocol identifier (e.g. `"shadowsocks"`, `"trojan"`).
    pub protocol: String,
    /// Upstream server address (hostname or IP).
    pub server_address: String,
    /// Upstream server port.
    pub server_port: u16,
    /// Optional URL for subscription-based node list updates.
    pub subscription_url: Option<String>,
    /// Protocol-specific extra parameters (opaque JSON or similar).
    pub extra_params: Option<String>,
}

// ---------------------------------------------------------------------------
// ProxyUnit
// ---------------------------------------------------------------------------

/// Runtime capability unit that wraps a `ProxyProfile` with a state machine
/// and health metadata.  Projects to `EgressProfile` for downstream consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProxyUnit {
    /// Unique unit identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Current state-machine state.
    pub state: ProxyUnitState,
    /// Identifier of the underlying `ProxyProfile`.
    pub profile_id: String,
    /// Last measured latency in milliseconds.
    pub latency_ms: Option<f64>,
    /// Label for identity/egress verification.
    pub identity_label: Option<String>,
    /// Computed risk score (0.0 = safe, 1.0 = maximum risk).
    pub risk_score: Option<f64>,
}

impl ProxyUnit {
    /// Attempt a state transition, returning an error for illegal moves.
    pub fn transition(&mut self, to: ProxyUnitState) -> Result<(), DomainError> {
        if self.state.can_transition_to(to) {
            self.state = to;
            Ok(())
        } else {
            Err(DomainError::InvalidTransition {
                from: self.state,
                to,
            })
        }
    }

    /// Create an `EgressProfile` projection from the current unit state.
    pub fn project_egress(&self) -> EgressProfile {
        EgressProfile {
            proxy_unit_id: self.id.clone(),
            latency_ms: self.latency_ms.unwrap_or(0.0),
            identity_label: self
                .identity_label
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            risk_score: self.risk_score.unwrap_or(0.0),
            active: self.state == ProxyUnitState::Active,
        }
    }
}

// ---------------------------------------------------------------------------
// EgressProfile
// ---------------------------------------------------------------------------

/// Projected view of a `ProxyUnit`'s capabilities: latency, identity, risk,
/// and whether the underlying unit is currently active.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EgressProfile {
    /// Identifier of the source `ProxyUnit`.
    pub proxy_unit_id: String,
    /// Measured latency in milliseconds.
    pub latency_ms: f64,
    /// Identity label (region, ISP, etc.).
    pub identity_label: String,
    /// Risk score (0.0–1.0).
    pub risk_score: f64,
    /// Whether the unit is currently in `Active` state.
    pub active: bool,
}

// ---------------------------------------------------------------------------
// AppBinding
// ---------------------------------------------------------------------------

/// Maps an application (identified by process name or bundle ID) to a
/// specific `ProxyUnit` for traffic routing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppBinding {
    /// Unique binding identifier.
    pub id: String,
    /// Process name or bundle identifier.
    pub app_identifier: String,
    /// Target proxy unit id.
    pub proxy_unit_id: String,
    /// Whether this binding is active.
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// AppGroup
// ---------------------------------------------------------------------------

/// A named group of applications that share a common routing policy.
/// Members inherit the group-level policy unless overridden individually.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppGroup {
    /// Unique group identifier.
    pub id: String,
    /// Human-readable group name.
    pub name: String,
    /// Application identifiers belonging to this group.
    pub members: Vec<String>,
    /// Optional policy id applied to the group.
    pub policy_id: Option<String>,
    /// Whether the group is active.
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// RuleSet
// ---------------------------------------------------------------------------

/// Ordered collection of routing rules evaluated by priority (lower value =
/// higher priority).  Each entry maps a matcher to an outbound action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleSet {
    /// Unique rule-set identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Evaluation priority (lower = higher).
    pub priority: u32,
    /// Ordered rule entries.
    pub rules: Vec<RuleEntry>,
}

/// Single routing rule inside a `RuleSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleEntry {
    /// Unique rule-entry identifier.
    pub id: String,
    /// Matcher expression string.
    pub matcher: String,
    /// Target outbound identifier.
    pub outbound: String,
}

// ---------------------------------------------------------------------------
// TargetSiteGroup
// ---------------------------------------------------------------------------

/// Named group of target sites used for connectivity and reachability checks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TargetSiteGroup {
    /// Unique group identifier.
    pub id: String,
    /// Human-readable group name.
    pub name: String,
    /// List of site URLs or hostnames to check.
    pub sites: Vec<String>,
    /// Interval between successive checks, in seconds.
    pub check_interval_secs: u64,
}

// ---------------------------------------------------------------------------
// CoverageInsight
// ---------------------------------------------------------------------------

/// Rolling analysis comparing actual traffic patterns against configured rule
/// coverage.  Highlights gaps and redundant rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoverageInsight {
    /// Unique insight identifier.
    pub id: String,
    /// Total requests observed in the analysis window.
    pub total_requests: u64,
    /// Requests that matched at least one rule.
    pub matched_requests: u64,
    /// Requests that matched no rule.
    pub unmatched_requests: u64,
    /// Ratio of matched to total requests.
    pub coverage_ratio: f64,
    /// Hostnames most frequently seen in unmatched traffic.
    pub top_unmatched_hosts: Vec<String>,
}

// ---------------------------------------------------------------------------
// ToolTemplateBinding
// ---------------------------------------------------------------------------

/// Maps a developer tool (e.g. Claude, Cursor, browser) to proxy and egress
/// configuration so each tool can use a tailored routing strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolTemplateBinding {
    /// Unique binding identifier.
    pub id: String,
    /// Name of the developer tool.
    pub tool_name: String,
    /// Optional proxy unit id to route through.
    pub proxy_unit_id: Option<String>,
    /// Optional egress label override.
    pub egress_label: Option<String>,
    /// Whether this binding is active.
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// PluginBinding
// ---------------------------------------------------------------------------

/// Associates a loaded plugin with its declared capabilities and trust level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginBinding {
    /// Unique binding identifier.
    pub id: String,
    /// Name of the plugin.
    pub plugin_name: String,
    /// List of declared capability strings.
    pub capabilities: Vec<String>,
    /// Trust classification (e.g. `"sandboxed"`, `"trusted"`).
    pub trust_level: String,
    /// Whether this binding is active.
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// RuntimePolicy
// ---------------------------------------------------------------------------

/// Runtime policy that can be applied at global, group, or app level.
/// `PolicyResolver` merges multiple layers using field-level priority override.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RuntimePolicy {
    /// Master enable/disable switch.
    pub enabled: Option<bool>,
    /// Routing mode (e.g. `"transparent"`, `"strict"`).
    pub mode: Option<String>,
    /// Logging verbosity.
    pub log_level: Option<String>,
    /// Maximum concurrent connections allowed.
    pub max_connections: Option<u64>,
    /// Action taken when no rule matches (e.g. `"direct"`, `"reject"`).
    pub fallback_action: Option<String>,
}

// ---------------------------------------------------------------------------
// TestSuiteConfig
// ---------------------------------------------------------------------------

/// Configuration for the 5-dimension test framework: connectivity, latency,
/// stability, identity, and coverage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestSuiteConfig {
    /// Unique configuration identifier.
    pub id: String,
    /// Hostnames or URLs to test connectivity against.
    pub connectivity_targets: Vec<String>,
    /// Maximum acceptable latency in milliseconds.
    pub latency_threshold_ms: f64,
    /// Time window in seconds for stability measurement.
    pub stability_window_secs: u64,
    /// Whether identity/egress verification is enabled.
    pub identity_check_enabled: bool,
    /// Minimum acceptable rule-coverage ratio (0.0–1.0).
    pub coverage_min_ratio: f64,
}

// ---------------------------------------------------------------------------
// ObservabilityRecord
// ---------------------------------------------------------------------------

/// Aggregated structured observation event capturing traffic, health, and
/// performance data for a time window.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservabilityRecord {
    /// Unique record identifier.
    pub id: String,
    /// Epoch milliseconds when the observation was recorded.
    pub timestamp_epoch_ms: u64,
    /// Classification of the event (e.g. `"traffic"`, `"health"`).
    pub event_type: String,
    /// Subsystem that produced the record.
    pub source: String,
    /// Opaque payload (often JSON).
    pub payload: String,
    /// Severity level (e.g. `"info"`, `"warn"`, `"error"`).
    pub severity: String,
}

// ---------------------------------------------------------------------------
// AbilityLens
// ---------------------------------------------------------------------------

/// Stateless projection of proxy ability maturity across the 5 test
/// dimensions: connectivity, latency, stability, identity, coverage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AbilityLens {
    /// Proxy unit being assessed.
    pub proxy_unit_id: String,
    /// Connectivity score (0.0–100.0).
    pub connectivity_score: f64,
    /// Latency score (0.0–100.0).
    pub latency_score: f64,
    /// Stability score (0.0–100.0).
    pub stability_score: f64,
    /// Identity verification score (0.0–100.0).
    pub identity_score: f64,
    /// Rule-coverage score (0.0–100.0).
    pub coverage_score: f64,
}

// ---------------------------------------------------------------------------
// PolicyResolver
// ---------------------------------------------------------------------------

/// Merges runtime policies across four priority levels.
///
/// Priority (highest -> lowest): Global > AppGroup > App-level > Default.
/// For each `Option` field the highest-priority `Some` value wins.
pub struct PolicyResolver;

impl PolicyResolver {
    /// Resolve a final `RuntimePolicy` by overlaying higher-priority layers
    /// onto the default.
    pub fn resolve(
        global: Option<&RuntimePolicy>,
        group: Option<&RuntimePolicy>,
        app: Option<&RuntimePolicy>,
        default: &RuntimePolicy,
    ) -> RuntimePolicy {
        let mut result = default.clone();

        if let Some(p) = app {
            Self::overlay(&mut result, p);
        }
        if let Some(p) = group {
            Self::overlay(&mut result, p);
        }
        if let Some(p) = global {
            Self::overlay(&mut result, p);
        }

        result
    }

    /// Apply `higher`'s `Some` fields on top of `base`.
    fn overlay(base: &mut RuntimePolicy, higher: &RuntimePolicy) {
        if higher.enabled.is_some() {
            base.enabled = higher.enabled;
        }
        if higher.mode.is_some() {
            base.mode.clone_from(&higher.mode);
        }
        if higher.log_level.is_some() {
            base.log_level.clone_from(&higher.log_level);
        }
        if higher.max_connections.is_some() {
            base.max_connections = higher.max_connections;
        }
        if higher.fallback_action.is_some() {
            base.fallback_action.clone_from(&higher.fallback_action);
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- ProxyUnit state machine ------------------------------------------

    fn make_unit(state: ProxyUnitState) -> ProxyUnit {
        ProxyUnit {
            id: "pu-1".into(),
            name: "test-unit".into(),
            state,
            profile_id: "pp-1".into(),
            latency_ms: Some(42.0),
            identity_label: Some("us-west".into()),
            risk_score: Some(0.1),
        }
    }

    #[test]
    fn transition_configuring_to_active() {
        let mut unit = make_unit(ProxyUnitState::Configuring);
        assert!(unit.transition(ProxyUnitState::Active).is_ok());
        assert_eq!(unit.state, ProxyUnitState::Active);
    }

    #[test]
    fn transition_active_to_degraded() {
        let mut unit = make_unit(ProxyUnitState::Active);
        assert!(unit.transition(ProxyUnitState::Degraded).is_ok());
        assert_eq!(unit.state, ProxyUnitState::Degraded);
    }

    #[test]
    fn transition_degraded_to_active() {
        let mut unit = make_unit(ProxyUnitState::Degraded);
        assert!(unit.transition(ProxyUnitState::Active).is_ok());
        assert_eq!(unit.state, ProxyUnitState::Active);
    }

    #[test]
    fn transition_active_to_inactive() {
        let mut unit = make_unit(ProxyUnitState::Active);
        assert!(unit.transition(ProxyUnitState::Inactive).is_ok());
        assert_eq!(unit.state, ProxyUnitState::Inactive);
    }

    #[test]
    fn transition_degraded_to_inactive() {
        let mut unit = make_unit(ProxyUnitState::Degraded);
        assert!(unit.transition(ProxyUnitState::Inactive).is_ok());
        assert_eq!(unit.state, ProxyUnitState::Inactive);
    }

    #[test]
    fn transition_inactive_to_configuring() {
        let mut unit = make_unit(ProxyUnitState::Inactive);
        assert!(unit.transition(ProxyUnitState::Configuring).is_ok());
        assert_eq!(unit.state, ProxyUnitState::Configuring);
    }

    #[test]
    fn transition_configuring_to_inactive_rejected() {
        let mut unit = make_unit(ProxyUnitState::Configuring);
        let err = unit.transition(ProxyUnitState::Inactive).unwrap_err();
        assert!(matches!(
            err,
            DomainError::InvalidTransition {
                from: ProxyUnitState::Configuring,
                to: ProxyUnitState::Inactive,
            }
        ));
        assert_eq!(unit.state, ProxyUnitState::Configuring);
    }

    #[test]
    fn transition_inactive_to_active_rejected() {
        let mut unit = make_unit(ProxyUnitState::Inactive);
        assert!(unit.transition(ProxyUnitState::Active).is_err());
    }

    #[test]
    fn transition_active_to_configuring_rejected() {
        let mut unit = make_unit(ProxyUnitState::Active);
        assert!(unit.transition(ProxyUnitState::Configuring).is_err());
    }

    #[test]
    fn transition_same_state_rejected() {
        let mut unit = make_unit(ProxyUnitState::Active);
        assert!(unit.transition(ProxyUnitState::Active).is_err());
    }

    // -- project_egress ---------------------------------------------------

    #[test]
    fn project_egress_active_unit() {
        let unit = make_unit(ProxyUnitState::Active);
        let egress = unit.project_egress();
        assert_eq!(egress.proxy_unit_id, "pu-1");
        assert!((egress.latency_ms - 42.0).abs() < f64::EPSILON);
        assert_eq!(egress.identity_label, "us-west");
        assert!(egress.active);
    }

    #[test]
    fn project_egress_defaults_for_missing_fields() {
        let unit = ProxyUnit {
            id: "pu-2".into(),
            name: "bare".into(),
            state: ProxyUnitState::Degraded,
            profile_id: "pp-2".into(),
            latency_ms: None,
            identity_label: None,
            risk_score: None,
        };
        let egress = unit.project_egress();
        assert!((egress.latency_ms - 0.0).abs() < f64::EPSILON);
        assert_eq!(egress.identity_label, "unknown");
        assert!(!egress.active);
    }

    // -- PolicyResolver ---------------------------------------------------

    fn policy(
        enabled: Option<bool>,
        mode: Option<&str>,
        log_level: Option<&str>,
        max_conn: Option<u64>,
        fallback: Option<&str>,
    ) -> RuntimePolicy {
        RuntimePolicy {
            enabled,
            mode: mode.map(String::from),
            log_level: log_level.map(String::from),
            max_connections: max_conn,
            fallback_action: fallback.map(String::from),
        }
    }

    #[test]
    fn resolve_uses_default_when_no_overrides() {
        let default = policy(
            Some(true),
            Some("transparent"),
            Some("info"),
            Some(100),
            Some("direct"),
        );
        let result = PolicyResolver::resolve(None, None, None, &default);
        assert_eq!(result, default);
    }

    #[test]
    fn resolve_app_overrides_default() {
        let default = policy(
            Some(true),
            Some("transparent"),
            Some("info"),
            Some(100),
            None,
        );
        let app = policy(None, Some("strict"), None, None, Some("reject"));
        let result = PolicyResolver::resolve(None, None, Some(&app), &default);

        assert_eq!(result.enabled, Some(true));
        assert_eq!(result.mode.as_deref(), Some("strict"));
        assert_eq!(result.log_level.as_deref(), Some("info"));
        assert_eq!(result.max_connections, Some(100));
        assert_eq!(result.fallback_action.as_deref(), Some("reject"));
    }

    #[test]
    fn resolve_group_overrides_app() {
        let default = policy(Some(true), None, None, None, None);
        let app = policy(None, Some("strict"), None, Some(50), None);
        let group = policy(None, Some("permissive"), None, None, None);
        let result = PolicyResolver::resolve(None, Some(&group), Some(&app), &default);

        assert_eq!(result.mode.as_deref(), Some("permissive"));
        assert_eq!(result.max_connections, Some(50));
    }

    #[test]
    fn resolve_global_overrides_everything() {
        let default = policy(
            Some(true),
            Some("transparent"),
            Some("info"),
            Some(100),
            Some("direct"),
        );
        let app = policy(None, Some("strict"), None, Some(50), None);
        let group = policy(None, Some("permissive"), None, None, Some("retry"));
        let global = policy(Some(false), None, Some("error"), None, None);
        let result = PolicyResolver::resolve(Some(&global), Some(&group), Some(&app), &default);

        assert_eq!(result.enabled, Some(false));
        assert_eq!(result.mode.as_deref(), Some("permissive"));
        assert_eq!(result.log_level.as_deref(), Some("error"));
        assert_eq!(result.max_connections, Some(50));
        assert_eq!(result.fallback_action.as_deref(), Some("retry"));
    }

    #[test]
    fn resolve_all_none_fields_keeps_default() {
        let default = policy(
            Some(true),
            Some("transparent"),
            Some("info"),
            Some(100),
            Some("direct"),
        );
        let empty = RuntimePolicy::default();
        let result = PolicyResolver::resolve(Some(&empty), Some(&empty), Some(&empty), &default);
        assert_eq!(result, default);
    }

    // -- DomainError display ---------------------------------------------

    #[test]
    fn domain_error_display() {
        let err = DomainError::InvalidTransition {
            from: ProxyUnitState::Configuring,
            to: ProxyUnitState::Inactive,
        };
        let msg = format!("{err}");
        assert!(msg.contains("Configuring"));
        assert!(msg.contains("Inactive"));
    }

    #[test]
    fn proxy_unit_all_valid_transitions_covered() {
        let valid_transitions = [
            (ProxyUnitState::Configuring, ProxyUnitState::Active),
            (ProxyUnitState::Active, ProxyUnitState::Degraded),
            (ProxyUnitState::Degraded, ProxyUnitState::Active),
            (ProxyUnitState::Active, ProxyUnitState::Inactive),
            (ProxyUnitState::Degraded, ProxyUnitState::Inactive),
            (ProxyUnitState::Inactive, ProxyUnitState::Configuring),
        ];
        for (from, to) in &valid_transitions {
            let mut unit = make_unit(*from);
            assert!(
                unit.transition(*to).is_ok(),
                "transition from {from:?} to {to:?} should succeed"
            );
            assert_eq!(unit.state, *to);
        }
    }

    #[test]
    fn proxy_unit_all_invalid_transitions_rejected() {
        let all_states = [
            ProxyUnitState::Configuring,
            ProxyUnitState::Active,
            ProxyUnitState::Degraded,
            ProxyUnitState::Inactive,
        ];
        let is_valid = |from: ProxyUnitState, to: ProxyUnitState| -> bool {
            matches!(
                (from, to),
                (ProxyUnitState::Configuring, ProxyUnitState::Active)
                    | (ProxyUnitState::Active, ProxyUnitState::Degraded)
                    | (ProxyUnitState::Degraded, ProxyUnitState::Active)
                    | (ProxyUnitState::Active, ProxyUnitState::Inactive)
                    | (ProxyUnitState::Degraded, ProxyUnitState::Inactive)
                    | (ProxyUnitState::Inactive, ProxyUnitState::Configuring)
            )
        };
        for from in &all_states {
            for to in &all_states {
                if is_valid(*from, *to) {
                    continue;
                }
                let mut unit = make_unit(*from);
                assert!(
                    unit.transition(*to).is_err(),
                    "transition from {from:?} to {to:?} should be rejected"
                );
            }
        }
    }

    #[test]
    fn egress_profile_serde_roundtrip() {
        let profile = EgressProfile {
            proxy_unit_id: "pu-1".to_string(),
            latency_ms: 42.0,
            identity_label: "us-west".to_string(),
            risk_score: 0.1,
            active: true,
        };
        let json = serde_json::to_string(&profile).expect("serialize");
        let deserialized: EgressProfile = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(profile, deserialized);
    }

    #[test]
    fn policy_resolver_all_none_returns_default_values() {
        let default = RuntimePolicy {
            enabled: Some(true),
            mode: Some("transparent".to_string()),
            log_level: Some("info".to_string()),
            max_connections: Some(500),
            fallback_action: Some("direct".to_string()),
        };
        let result = PolicyResolver::resolve(None, None, None, &default);
        assert_eq!(result.enabled, Some(true));
        assert_eq!(result.mode.as_deref(), Some("transparent"));
        assert_eq!(result.log_level.as_deref(), Some("info"));
        assert_eq!(result.max_connections, Some(500));
        assert_eq!(result.fallback_action.as_deref(), Some("direct"));
    }

    #[test]
    fn app_group_basic_construction() {
        let group = AppGroup {
            id: "ag-1".to_string(),
            name: "dev-tools".to_string(),
            members: vec!["cursor".to_string(), "chrome".to_string()],
            policy_id: Some("policy-1".to_string()),
            enabled: true,
        };
        assert_eq!(group.id, "ag-1");
        assert_eq!(group.name, "dev-tools");
        assert_eq!(group.members.len(), 2);
        assert!(group.members.contains(&"cursor".to_string()));
        assert!(group.members.contains(&"chrome".to_string()));
        assert_eq!(group.policy_id.as_deref(), Some("policy-1"));
        assert!(group.enabled);
    }
}
