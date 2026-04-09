use crate::dispatcher::AppController;
use crate::theme;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let outbounds = controller.outbounds();
    let rules = controller.rules();
    let default_ob = controller.default_outbound_id();
    let snap = controller.snapshot();

    let outbound_ids: Vec<String> = outbounds.iter().map(|o| o.id.clone()).collect();

    let app_rows: Vec<Vec<String>> = rules
        .iter()
        .filter(|r| r.host_equals.is_some() || r.host_suffix.is_some())
        .map(|r| {
            let pattern = r
                .host_equals
                .as_deref()
                .or(r.host_suffix.as_deref())
                .unwrap_or("—");
            let condition = if r.host_equals.is_some() {
                "host_equals"
            } else {
                "host_suffix"
            };
            vec![
                pattern.to_string(),
                r.outbound.clone(),
                format!("{} ({})", r.id, condition),
                if r.outbound == "reject" {
                    "BLOCKED".to_string()
                } else {
                    "ACTIVE".to_string()
                },
            ]
        })
        .collect();

    let matched_rules: Vec<Vec<String>> = rules
        .iter()
        .map(|r| {
            let cond = if let Some(h) = &r.host_equals {
                h.clone()
            } else if let Some(s) = &r.host_suffix {
                s.clone()
            } else if let Some(p) = r.port {
                format!("port:{p}")
            } else {
                "—".to_string()
            };
            vec![r.id.clone(), cond, r.outbound.clone()]
        })
        .collect();

    let mut filter_options = vec!["ALL".to_string()];
    for id in &outbound_ids {
        filter_options.push(id.to_uppercase());
    }

    let mut page = View::vstack()
        .child(View::text("APPS").bold().font_size(theme::TITLE_LG))
        .child(
            View::text("APPLICATION-LEVEL PROXY ROUTING AND COVERAGE ANALYSIS")
                .font_size(theme::CAPTION)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(16.0))
        .child(
            View::hstack()
                .child(View::search_field("FILTER APPLICATIONS…").width(300.0))
                .child(View::spacer().width(12.0))
                .child(View::dropdown("OUTBOUND FILTER", filter_options))
                .child(View::spacer()),
        )
        .child(View::spacer().height(12.0))
        .child(
            theme::yorha_group_box("APPLICATION BINDINGS").child(
                View::table_view(
                    vec![
                        "APPLICATION".to_string(),
                        "OUTBOUND".to_string(),
                        "RULE".to_string(),
                        "STATUS".to_string(),
                    ],
                    app_rows,
                )
                .height(200.0),
            ),
        )
        .child(View::spacer().height(12.0));

    if !matched_rules.is_empty() {
        page = page.child(
            theme::yorha_group_box("MATCHED RULES").child(
                View::table_view(
                    vec![
                        "ID".to_string(),
                        "CONDITION".to_string(),
                        "OUTBOUND".to_string(),
                    ],
                    matched_rules,
                )
                .height(160.0),
            ),
        );
        page = page.child(View::spacer().height(12.0));
    }

    page = page.child(
        theme::yorha_group_box("TRAFFIC").child(
            View::hstack()
                .child(
                    View::vstack()
                        .child(
                            View::text(&snap.session_count.to_string())
                                .bold()
                                .font_size(theme::TITLE_LG),
                        )
                        .child(View::text("SESSIONS").font_size(theme::CAPTION))
                        .padding(8.0),
                )
                .child(View::spacer().width(24.0))
                .child(
                    View::vstack()
                        .child(
                            View::text(&snap.total_events.to_string())
                                .bold()
                                .font_size(theme::TITLE_LG),
                        )
                        .child(View::text("EVENTS").font_size(theme::CAPTION))
                        .padding(8.0),
                )
                .child(View::spacer().width(24.0))
                .child(
                    View::vstack()
                        .child(
                            View::text(&snap.error_count.to_string())
                                .bold()
                                .font_size(theme::TITLE_LG),
                        )
                        .child(View::text("ERRORS").font_size(theme::CAPTION))
                        .padding(8.0),
                )
                .child(View::spacer().width(24.0))
                .child(
                    View::vstack()
                        .child(
                            View::text(&default_ob.to_uppercase())
                                .bold()
                                .font_size(theme::TITLE_MD),
                        )
                        .child(View::text("DEFAULT EXIT").font_size(theme::CAPTION))
                        .padding(8.0),
                )
                .child(View::spacer()),
        ),
    );

    page
}
