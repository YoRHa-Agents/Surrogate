use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let outbounds = controller.outbounds();
    let default_ob = controller.default_outbound_id();
    let rules = controller.rules();
    let (_, sessions, _) = controller.event_counts();

    let outbound_names: Vec<String> = outbounds.iter().map(|o| o.id.clone()).collect();

    let app_rows = vec![
        vec![
            "Safari".to_string(),
            "com.apple.Safari".to_string(),
            "DIRECT".to_string(),
            "active".to_string(),
        ],
        vec![
            "Cursor".to_string(),
            "com.todesktop.230313mzl4w4u92".to_string(),
            outbound_names.first().cloned().unwrap_or_else(|| "direct".to_string()),
            "active".to_string(),
        ],
        vec![
            "Claude Code".to_string(),
            "cli:claude".to_string(),
            default_ob.clone(),
            "idle".to_string(),
        ],
        vec![
            "Terminal".to_string(),
            "com.apple.Terminal".to_string(),
            "DIRECT".to_string(),
            "active".to_string(),
        ],
        vec![
            "ChatGPT".to_string(),
            "com.openai.chat".to_string(),
            "REJECT".to_string(),
            "blocked".to_string(),
        ],
    ];

    let matched_rules: Vec<Vec<String>> = rules
        .iter()
        .take(5)
        .map(|r| {
            let cond = r
                .host_equals
                .as_deref()
                .or(r.host_suffix.as_deref())
                .unwrap_or("—");
            vec![r.id.clone(), cond.to_string(), r.outbound.clone()]
        })
        .collect();

    let mut page = View::vstack()
        .child(View::text("Apps").bold().font_size(22.0))
        .child(
            View::text("Application-level proxy routing and coverage analysis")
                .font_size(12.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(16.0))
        .child(
            View::hstack()
                .child(View::search_field("Filter applications…").width(300.0))
                .child(View::spacer().width(12.0))
                .child(View::dropdown(
                    "Group",
                    vec![
                        "All".to_string(),
                        "Active".to_string(),
                        "Blocked".to_string(),
                        "Idle".to_string(),
                    ],
                ))
                .child(View::spacer()),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Application Bindings").child(
                View::table_view(
                    vec![
                        "Application".to_string(),
                        "Bundle / CLI ID".to_string(),
                        "Outbound".to_string(),
                        "Status".to_string(),
                    ],
                    app_rows,
                )
                .height(200.0),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("AppGroup Management").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("Default Group").bold().font_size(12.0))
                            .child(View::spacer())
                            .child(View::text(&format!("Default exit: {default_ob}")).font_size(11.0)),
                    )
                    .child(View::spacer().height(8.0))
                    .child(
                        View::text(&format!("Active sessions: {sessions}")).font_size(12.0),
                    )
                    .child(View::spacer().height(4.0))
                    .child(
                        View::text(&format!("Configured outbounds: {}", outbound_names.join(", ")))
                            .font_size(11.0)
                            .foreground("secondaryLabelColor"),
                    ),
            ),
        );

    if !matched_rules.is_empty() {
        page = page
            .child(View::spacer().height(12.0))
            .child(
                View::group_box("Recently Matched Rules").child(
                    View::table_view(
                        vec![
                            "Rule ID".to_string(),
                            "Condition".to_string(),
                            "Outbound".to_string(),
                        ],
                        matched_rules,
                    ),
                ),
            );
    }

    page
}
