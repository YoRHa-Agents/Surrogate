use serde::{Deserialize, Serialize};

/// Boolean predicate tree used to match traffic attributes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RulePredicate {
    /// Exact match on the target hostname.
    HostEquals(String),
    /// Suffix match (e.g. `.example.com`).
    HostSuffix(String),
    /// Regex match against the hostname.
    HostRegex(String),
    /// Exact match on the destination port.
    PortEquals(u16),
    /// Inclusive port range `[lo, hi]`.
    PortRange(u16, u16),
    /// CIDR-notation IP match.
    IpCidr(String),
    /// Two-letter ISO country code via GeoIP lookup.
    GeoIp(String),
    /// Match by originating process name.
    ProcessName(String),
    /// All child predicates must match.
    And(Vec<RulePredicate>),
    /// At least one child predicate must match.
    Or(Vec<RulePredicate>),
    /// Negate the inner predicate.
    Not(Box<RulePredicate>),
}

/// A single compiled routing rule ready for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompiledRule {
    /// Unique rule identifier.
    pub id: String,
    /// Lower value = higher priority.
    pub priority: u32,
    /// Predicate tree that must evaluate to `true` for this rule to fire.
    pub predicate: RulePredicate,
    /// Target outbound to route matching traffic through.
    pub outbound: String,
    /// Whether the rule is currently active.
    pub enabled: bool,
}

/// Versioned set of compiled rules with a fallback outbound.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledRuleSet {
    /// Ordered list of compiled rules.
    pub rules: Vec<CompiledRule>,
    /// Outbound used when no rule matches.
    pub default_outbound: String,
    /// Monotonically increasing version for change detection.
    pub version: u64,
}

/// Describes an overlap or contradiction between two rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConflict {
    /// First conflicting rule id.
    pub rule_a_id: String,
    /// Second conflicting rule id.
    pub rule_b_id: String,
    /// Human-readable description of the conflict.
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_rule_serde_roundtrip() {
        let rule = CompiledRule {
            id: "rule-1".to_string(),
            priority: 10,
            predicate: RulePredicate::HostEquals("example.com".to_string()),
            outbound: "direct".to_string(),
            enabled: true,
        };
        let json = serde_json::to_string(&rule).expect("serialize");
        let deserialized: CompiledRule = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(rule, deserialized);
    }

    #[test]
    fn rule_set_serde_roundtrip() {
        let rule_set = CompiledRuleSet {
            rules: vec![CompiledRule {
                id: "r1".to_string(),
                priority: 1,
                predicate: RulePredicate::PortEquals(443),
                outbound: "direct".to_string(),
                enabled: true,
            }],
            default_outbound: "reject".to_string(),
            version: 42,
        };
        let json = serde_json::to_string(&rule_set).expect("serialize");
        let deserialized: CompiledRuleSet = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.default_outbound, "reject");
        assert_eq!(deserialized.version, 42);
        assert_eq!(deserialized.rules.len(), 1);
        assert_eq!(deserialized.rules[0], rule_set.rules[0]);
    }

    #[test]
    fn nested_predicate_serde_roundtrip() {
        let predicate = RulePredicate::And(vec![
            RulePredicate::Or(vec![
                RulePredicate::HostEquals("a.com".to_string()),
                RulePredicate::HostSuffix(".b.com".to_string()),
            ]),
            RulePredicate::Not(Box::new(RulePredicate::PortRange(1000, 2000))),
            RulePredicate::IpCidr("10.0.0.0/8".to_string()),
            RulePredicate::GeoIp("US".to_string()),
            RulePredicate::ProcessName("curl".to_string()),
            RulePredicate::HostRegex(r".*\.example\.com".to_string()),
        ]);
        let json = serde_json::to_string(&predicate).expect("serialize");
        let deserialized: RulePredicate = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(predicate, deserialized);
    }

    #[test]
    fn rule_conflict_serde_roundtrip() {
        let conflict = RuleConflict {
            rule_a_id: "rule-1".to_string(),
            rule_b_id: "rule-2".to_string(),
            description: "overlapping host patterns".to_string(),
        };
        let json = serde_json::to_string(&conflict).expect("serialize");
        let deserialized: RuleConflict = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.rule_a_id, conflict.rule_a_id);
        assert_eq!(deserialized.rule_b_id, conflict.rule_b_id);
        assert_eq!(deserialized.description, conflict.description);
    }
}
