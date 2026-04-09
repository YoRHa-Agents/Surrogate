use crate::dispatcher::AppController;
use cocoanut::prelude::*;
use surrogate_control::ability_lens::{AbilityLens, AbilityStatus};

pub fn build(controller: &AppController) -> View {
    let status = controller.status();
    let mode = controller.ui_mode();
    let sys_proxy = controller.is_system_proxy_enabled();
    let (total_events, sessions, errors) = controller.event_counts();
    let default_ob = controller.default_outbound_id();

    let proxy_state = if status.running {
        format!(
            "● Running on {}",
            status.listen_addr.as_deref().unwrap_or("unknown")
        )
    } else {
        "○ Stopped".to_string()
    };

    let uptime_text = status
        .uptime_secs
        .map(|s| format_uptime(s))
        .unwrap_or_else(|| "—".to_string());

    let abilities = AbilityLens::all_abilities();
    let snapshots = AbilityLens::project(&abilities);

    let mut ability_rows: Vec<Vec<String>> = Vec::new();
    for snap in &snapshots {
        let name = format!("{:?}", snap.ability);
        let status_str = match snap.status {
            AbilityStatus::NotStarted => "Not Started",
            AbilityStatus::InProgress => "In Progress",
            AbilityStatus::Functional => "Functional",
            AbilityStatus::Mature => "Mature",
        };
        let maturity = format!("{:.0}%", snap.maturity_pct);
        let deps_ok = snap.dependencies_met.len().to_string();
        let deps_missing = snap.dependencies_unmet.len().to_string();
        ability_rows.push(vec![
            name,
            status_str.to_string(),
            maturity,
            deps_ok,
            deps_missing,
        ]);
    }

    let ctrl_start = controller.clone();
    let ctrl_stop = controller.clone();

    View::vstack()
        .child(View::text("Overview").bold().font_size(22.0))
        .child(View::spacer().height(16.0))
        .child(
            View::group_box("Platform Status").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text(&proxy_state).font_size(14.0).bold())
                            .child(View::spacer())
                            .child(
                                View::button("Start").on_click_fn(move || {
                                    let c = ctrl_start.clone();
                                    std::thread::spawn(move || {
                                        let _ = c.start_proxy();
                                    });
                                }),
                            )
                            .child(
                                View::button("Stop").on_click_fn(move || {
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
                            .child(View::text(&format!("Uptime: {uptime_text}")).font_size(12.0))
                            .child(View::spacer().width(24.0))
                            .child(
                                View::text(&format!("Default Exit: {default_ob}")).font_size(12.0),
                            )
                            .child(View::spacer().width(24.0))
                            .child(
                                View::text(&format!(
                                    "System Proxy: {}",
                                    if sys_proxy { "Enabled" } else { "Disabled" }
                                ))
                                .font_size(12.0),
                            ),
                    )
                    .child(View::spacer().height(4.0))
                    .child(
                        View::text(&format!("Mode: {:?}", mode))
                            .font_size(12.0)
                            .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Traffic Summary").child(
                View::hstack()
                    .child(stat_card("Total Events", &total_events.to_string()))
                    .child(View::spacer().width(16.0))
                    .child(stat_card("Sessions", &sessions.to_string()))
                    .child(View::spacer().width(16.0))
                    .child(stat_card("Errors", &errors.to_string())),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Configuration").child(
                View::text(&status.config_summary).font_size(12.0),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Capability Overview").child(
                View::table_view(
                    vec![
                        "Capability".to_string(),
                        "Status".to_string(),
                        "Maturity".to_string(),
                        "Met".to_string(),
                        "Unmet".to_string(),
                    ],
                    ability_rows,
                ),
            ),
        )
}

fn stat_card(title: &str, value: &str) -> View {
    View::vstack()
        .child(View::text(value).bold().font_size(24.0))
        .child(
            View::text(title)
                .font_size(11.0)
                .foreground("secondaryLabelColor"),
        )
        .padding(8.0)
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
