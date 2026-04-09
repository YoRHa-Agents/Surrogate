use crate::dispatcher::AppController;
use crate::theme::{yorha_group_box, BODY, CAPTION, TITLE_LG, TITLE_SM};
use cocoanut::prelude::*;
use surrogate_contract::rules::{CompiledRule, RulePredicate};
use surrogate_control::rule_compiler::RuleCompiler;

pub fn build(controller: &AppController) -> View {
    let rules = controller.rules();
    let default_ob = controller.default_outbound_id();

    let rule_rows: Vec<Vec<String>> = rules
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let priority = format!("{}", i + 1);
            let condition = format_condition(r);
            vec![priority, r.id.clone(), condition, r.outbound.clone()]
        })
        .collect();

    let has_rules = !rule_rows.is_empty();

    let compiled_rules: Vec<CompiledRule> = rules
        .iter()
        .enumerate()
        .map(|(i, r)| CompiledRule {
            id: r.id.clone(),
            priority: (i + 1) as u32,
            predicate: build_predicate(r),
            outbound: r.outbound.clone(),
            enabled: true,
        })
        .collect();

    let compilation = RuleCompiler::compile(compiled_rules, default_ob.clone());
    let conflicts = &compilation.conflicts;

    let mut page = View::vstack()
        .child(View::text("RULES").bold().font_size(TITLE_LG))
        .child(View::spacer().height(16.0));

    if has_rules {
        page = page.child(
            yorha_group_box("ROUTING RULES").child(View::table_view(
                vec![
                    "PRIORITY".to_string(),
                    "ID".to_string(),
                    "CONDITION".to_string(),
                    "OUTBOUND".to_string(),
                ],
                rule_rows,
            )),
        );
    } else {
        page = page.child(
            yorha_group_box("ROUTING RULES").child(
                View::text("No routing rules configured.").font_size(BODY),
            ),
        );
    }

    page = page.child(View::spacer().height(12.0));

    let conflict_count = conflicts.len();
    let mut conflict_section = View::vstack().child(
        View::text(&format!("Conflicts detected: {conflict_count}"))
            .font_size(BODY)
            .bold(),
    );

    if !conflicts.is_empty() {
        conflict_section = conflict_section.child(View::spacer().height(4.0));
        for c in conflicts {
            conflict_section = conflict_section.child(
                View::text(&format!(
                    "WARNING: Conflict: {} vs {} (same priority)",
                    c.rule_a_id, c.rule_b_id
                ))
                .font_size(CAPTION),
            );
        }
    }

    page = page
        .child(yorha_group_box("CONFLICT DETECTION").child(conflict_section))
        .child(View::spacer().height(12.0))
        .child(
            yorha_group_box("DEFAULT ROUTE").child(
                View::hstack()
                    .child(
                        View::text("Fallback destination:")
                            .font_size(TITLE_SM)
                            .bold(),
                    )
                    .child(View::spacer().width(8.0))
                    .child(View::text(&default_ob).font_size(TITLE_SM))
                    .child(View::spacer()),
            ),
        );

    page
}

fn format_condition(rule: &surrogate_contract::config::RouteRuleConfig) -> String {
    let mut parts = Vec::new();
    if let Some(h) = &rule.host_equals {
        parts.push(format!("HOST = {h}"));
    }
    if let Some(s) = &rule.host_suffix {
        parts.push(format!("SUFFIX = {s}"));
    }
    if let Some(p) = rule.port {
        parts.push(format!("PORT = {p}"));
    }
    if parts.is_empty() {
        "\u{2014}".to_string()
    } else {
        parts.join(", ")
    }
}

fn build_predicate(rule: &surrogate_contract::config::RouteRuleConfig) -> RulePredicate {
    let mut predicates = Vec::new();
    if let Some(h) = &rule.host_equals {
        predicates.push(RulePredicate::HostEquals(h.clone()));
    }
    if let Some(s) = &rule.host_suffix {
        predicates.push(RulePredicate::HostSuffix(s.clone()));
    }
    if let Some(p) = rule.port {
        predicates.push(RulePredicate::PortEquals(p));
    }
    match predicates.len() {
        0 => RulePredicate::HostEquals(String::new()),
        1 => predicates.remove(0),
        _ => RulePredicate::And(predicates),
    }
}
