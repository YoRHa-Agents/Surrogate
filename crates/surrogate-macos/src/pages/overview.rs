use crate::dispatcher::{view_tags, AppController};
use crate::theme;
use cocoanut::prelude::*;
use surrogate_contract::events::EventKind;
use surrogate_control::ability_lens::{AbilityLens, AbilityStatus};

pub fn build(controller: &AppController) -> View {
    let snap = controller.snapshot();

    let proxy_state = if snap.running {
        format!(
            "● Running on {}",
            snap.listen_addr.as_deref().unwrap_or("unknown")
        )
    } else {
        "○ Stopped".to_string()
    };

    let uptime_text = snap
        .uptime_secs
        .map(format_uptime)
        .unwrap_or_else(|| "—".to_string());

    let sys_proxy_text = if snap.system_proxy {
        "Enabled"
    } else {
        "Disabled"
    };

    let mode_text = format!("{:?}", snap.mode);

    let abilities = AbilityLens::all_abilities();
    let projections = AbilityLens::project(&abilities);

    let mut ability_rows: Vec<Vec<String>> = Vec::new();
    for s in &projections {
        let name = format!("{:?}", s.ability);
        let status_str = match s.status {
            AbilityStatus::NotStarted => "Not Started",
            AbilityStatus::InProgress => "In Progress",
            AbilityStatus::Functional => "Functional",
            AbilityStatus::Mature => "Mature",
        };
        ability_rows.push(vec![
            name,
            status_str.to_string(),
            format!("{:.0}%", s.maturity_pct),
            s.dependencies_met.len().to_string(),
            s.dependencies_unmet.len().to_string(),
        ]);
    }

    let error_events: Vec<_> = snap
        .recent_events
        .iter()
        .filter(|e| e.kind == EventKind::Error)
        .collect();
    let active_error_count = error_events.len();
    let last_errors: Vec<String> = error_events
        .iter()
        .rev()
        .take(5)
        .map(|e| crate::event_format::event_detail_line(e))
        .collect();

    let ctrl_start = controller.clone();
    let ctrl_stop = controller.clone();

    let mut page = View::vstack()
        .child(View::text("OVERVIEW").bold().font_size(theme::TITLE_LG))
        .child(View::spacer().height(16.0))
        .child(
            theme::yorha_group_box("PLATFORM STATUS").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(
                                View::text(&proxy_state)
                                    .font_size(theme::TITLE_SM)
                                    .bold()
                                    .tag(view_tags::OVERVIEW_PROXY_STATE),
                            )
                            .child(View::spacer())
                            .child(
                                theme::yorha_button("Start").on_click_fn(move || {
                                    let c = ctrl_start.clone();
                                    std::thread::spawn(move || {
                                        let _ = c.start_proxy();
                                    });
                                }),
                            )
                            .child(
                                theme::yorha_button("Stop").on_click_fn(move || {
                                    let c = ctrl_stop.clone();
                                    std::thread::spawn(move || {
                                        let _ = c.stop_proxy();
                                    });
                                }),
                            ),
                    )
                    .child(View::spacer().height(8.0))
                    .child(
                        View::hstack()
                            .child(
                                View::text(&format!("Uptime: {uptime_text}"))
                                    .font_size(theme::BODY)
                                    .tag(view_tags::OVERVIEW_UPTIME),
                            )
                            .child(View::spacer().width(24.0))
                            .child(
                                View::text(&format!(
                                    "Default Exit: {}",
                                    snap.default_outbound
                                ))
                                .font_size(theme::BODY)
                                .tag(view_tags::OVERVIEW_DEFAULT_EXIT),
                            )
                            .child(View::spacer().width(24.0))
                            .child(
                                View::text(&format!("System Proxy: {sys_proxy_text}"))
                                    .font_size(theme::BODY),
                            ),
                    )
                    .child(View::spacer().height(4.0))
                    .child(
                        View::text(&format!("Mode: {mode_text}"))
                            .font_size(theme::BODY)
                            .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            theme::yorha_group_box("TRAFFIC SUMMARY").child(
                View::hstack()
                    .child(theme::yorha_stat_card(
                        "Total Events",
                        &snap.total_events.to_string(),
                        view_tags::OVERVIEW_TOTAL_EVENTS,
                    ))
                    .child(View::spacer().width(16.0))
                    .child(theme::yorha_stat_card(
                        "Sessions",
                        &snap.session_count.to_string(),
                        view_tags::OVERVIEW_SESSIONS,
                    ))
                    .child(View::spacer().width(16.0))
                    .child(theme::yorha_stat_card(
                        "Errors",
                        &snap.error_count.to_string(),
                        view_tags::OVERVIEW_ERRORS,
                    )),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            theme::yorha_group_box("CONFIGURATION").child(
                View::vstack()
                    .child(
                        View::text(&snap.config_summary)
                            .font_size(theme::BODY)
                            .tag(view_tags::OVERVIEW_CONFIG),
                    )
                    .child(View::spacer().height(4.0))
                    .child(
                        View::text(&snap.config_path)
                            .font_size(theme::CAPTION)
                            .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            theme::yorha_group_box("CAPABILITY OVERVIEW").child(View::table_view(
                vec![
                    "Capability".to_string(),
                    "Status".to_string(),
                    "Maturity".to_string(),
                    "Met".to_string(),
                    "Unmet".to_string(),
                ],
                ability_rows,
            )),
        )
        .child(View::spacer().height(12.0));

    let mut alerts_content = View::vstack().child(
        View::text(&format!("{active_error_count} active error(s)"))
            .font_size(theme::BODY)
            .bold(),
    );
    if !last_errors.is_empty() {
        alerts_content = alerts_content.child(View::spacer().height(4.0));
        for msg in &last_errors {
            alerts_content = alerts_content.child(
                View::text(msg)
                    .font_size(theme::CAPTION)
                    .foreground("secondaryLabelColor"),
            );
        }
    }
    page = page.child(theme::yorha_group_box("ACTIVE ALERTS").child(alerts_content));

    page
}

fn format_uptime(secs: u64) -> String {
    if secs < 60 {
        return format!("{secs}s");
    }
    if secs < 3600 {
        return format!("{}m {}s", secs / 60, secs % 60);
    }
    let h = secs / 3600;
    format!("{h}h {}m", (secs % 3600) / 60)
}
