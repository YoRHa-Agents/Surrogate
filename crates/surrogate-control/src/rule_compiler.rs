use surrogate_contract::rules::{CompiledRule, CompiledRuleSet, RuleConflict};

/// Compiles routing rules from config into an optimized, immutable rule set.
pub struct RuleCompiler;

impl RuleCompiler {
    /// Compile rules into a versioned rule set, detecting priority conflicts.
    pub fn compile(rules: Vec<CompiledRule>, default_outbound: String) -> CompilationResult {
        let conflicts = Self::detect_conflicts(&rules);
        let version = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        CompilationResult {
            rule_set: CompiledRuleSet {
                rules,
                default_outbound,
                version,
            },
            conflicts,
        }
    }

    fn detect_conflicts(rules: &[CompiledRule]) -> Vec<RuleConflict> {
        let mut conflicts = Vec::new();
        for (i, a) in rules.iter().enumerate() {
            for b in rules.iter().skip(i + 1) {
                if a.priority == b.priority {
                    conflicts.push(RuleConflict {
                        rule_a_id: a.id.clone(),
                        rule_b_id: b.id.clone(),
                        description: format!("rules have the same priority ({})", a.priority),
                    });
                }
            }
        }
        conflicts
    }
}

pub struct CompilationResult {
    pub rule_set: CompiledRuleSet,
    pub conflicts: Vec<RuleConflict>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrogate_contract::rules::RulePredicate;

    fn make_rule(id: &str, priority: u32) -> CompiledRule {
        CompiledRule {
            id: id.to_string(),
            priority,
            predicate: RulePredicate::HostEquals("example.com".to_string()),
            outbound: "direct".to_string(),
            enabled: true,
        }
    }

    #[test]
    fn compile_no_conflicts() {
        let rules = vec![make_rule("r1", 1), make_rule("r2", 2)];
        let result = RuleCompiler::compile(rules, "direct".to_string());

        assert!(result.conflicts.is_empty());
        assert_eq!(result.rule_set.rules.len(), 2);
        assert_eq!(result.rule_set.default_outbound, "direct");
        assert!(result.rule_set.version > 0);
    }

    #[test]
    fn compile_detects_priority_conflict() {
        let rules = vec![make_rule("r1", 5), make_rule("r2", 5), make_rule("r3", 10)];
        let result = RuleCompiler::compile(rules, "direct".to_string());

        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].rule_a_id, "r1");
        assert_eq!(result.conflicts[0].rule_b_id, "r2");
        assert!(result.conflicts[0].description.contains("5"));
    }

    #[test]
    fn compile_three_way_same_priority_conflict() {
        let rules = vec![make_rule("r1", 5), make_rule("r2", 5), make_rule("r3", 5)];
        let result = RuleCompiler::compile(rules, "direct".to_string());

        assert_eq!(result.conflicts.len(), 3);
        let pairs: Vec<(&str, &str)> = result
            .conflicts
            .iter()
            .map(|c| (c.rule_a_id.as_str(), c.rule_b_id.as_str()))
            .collect();
        assert!(pairs.contains(&("r1", "r2")));
        assert!(pairs.contains(&("r1", "r3")));
        assert!(pairs.contains(&("r2", "r3")));
    }

    #[test]
    fn compile_empty_rules() {
        let result = RuleCompiler::compile(vec![], "fallback".to_string());
        assert!(result.conflicts.is_empty());
        assert!(result.rule_set.rules.is_empty());
        assert_eq!(result.rule_set.default_outbound, "fallback");
    }

    #[test]
    fn compile_disabled_rules_still_in_set() {
        let mut rule = make_rule("r1", 1);
        rule.enabled = false;
        let result = RuleCompiler::compile(vec![rule], "direct".to_string());
        assert_eq!(result.rule_set.rules.len(), 1);
        assert!(!result.rule_set.rules[0].enabled);
    }

    #[test]
    fn compile_version_is_nonzero() {
        let result = RuleCompiler::compile(vec![make_rule("r1", 1)], "direct".to_string());
        assert!(result.rule_set.version > 0);
    }
}
