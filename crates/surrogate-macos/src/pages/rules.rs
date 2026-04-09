use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let rules = controller.rules();
    let default_ob = controller.default_outbound_id();

    let rule_rows: Vec<Vec<String>> = rules
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let priority = format!("{}", (i + 1) * 10);
            let condition = if let Some(h) = &r.host_equals {
                format!("HOST = {h}")
            } else if let Some(s) = &r.host_suffix {
                format!("SUFFIX = {s}")
            } else if let Some(p) = r.port {
                format!("PORT = {p}")
            } else {
                "—".to_string()
            };
            vec![priority, r.id.clone(), condition, r.outbound.clone()]
        })
        .collect();

    let has_rules = !rule_rows.is_empty();

    let mut page = View::vstack()
        .child(View::text("Rules").bold().font_size(22.0))
        .child(
            View::text("Routing rules determine how traffic reaches its exit point")
                .font_size(12.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(16.0));

    if has_rules {
        page = page.child(
            View::group_box("Rule List (priority order)").child(
                View::table_view(
                    vec![
                        "Priority".to_string(),
                        "Rule ID".to_string(),
                        "Condition".to_string(),
                        "Outbound".to_string(),
                    ],
                    rule_rows,
                ),
            ),
        );
    } else {
        page = page.child(
            View::group_box("Rule List").child(
                View::vstack()
                    .child(
                        View::text("No rules configured — all traffic uses the default outbound")
                            .font_size(12.0),
                    )
                    .child(View::spacer().height(8.0))
                    .child(
                        View::text(
                            "Add rules in config.toml to route specific hosts or ports",
                        )
                        .font_size(11.0)
                        .foreground("secondaryLabelColor"),
                    ),
            ),
        );
    }

    page = page
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Default Fallback").child(
                View::hstack()
                    .child(View::text("MATCH  *  →").font_size(13.0))
                    .child(View::spacer().width(8.0))
                    .child(View::text(&default_ob).bold().font_size(13.0))
                    .child(View::spacer()),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Conflict Analysis").child(
                View::vstack()
                    .child(
                        View::text(&format!(
                            "Total rules: {}",
                            rules.len()
                        ))
                        .font_size(12.0),
                    )
                    .child(check_overlaps(&rules)),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Add Rule").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("Type:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(View::segmented_control(vec![
                                "Domain".to_string(),
                                "Suffix".to_string(),
                                "Port".to_string(),
                            ]))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(8.0))
                    .child(
                        View::hstack()
                            .child(View::text_field("Value (e.g. example.com)").width(300.0))
                            .child(View::spacer().width(8.0))
                            .child(View::button("Add Rule"))
                            .child(View::spacer()),
                    ),
            ),
        );

    page
}

fn check_overlaps(rules: &[surrogate_contract::config::RouteRuleConfig]) -> View {
    let mut warnings = Vec::new();
    for (i, a) in rules.iter().enumerate() {
        for b in rules.iter().skip(i + 1) {
            if let (Some(sa), Some(sb)) = (&a.host_suffix, &b.host_suffix) {
                if sa.ends_with(sb) || sb.ends_with(sa) {
                    warnings.push(format!(
                        "Possible overlap: {} ({}) vs {} ({})",
                        a.id, sa, b.id, sb
                    ));
                }
            }
        }
    }

    if warnings.is_empty() {
        View::text("No conflicts detected")
            .font_size(11.0)
            .foreground("secondaryLabelColor")
    } else {
        let mut col = View::vstack();
        for w in warnings {
            col = col.child(View::text(&format!("⚠ {w}")).font_size(11.0));
        }
        col
    }
}
