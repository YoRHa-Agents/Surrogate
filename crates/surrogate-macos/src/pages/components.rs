use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let running = controller.is_running();
    let status = controller.status();
    let event_count = status.total_events;

    let status_label = if running { "● Active" } else { "○ Inactive" };

    View::vstack()
        .child(View::text("Components").bold().font_size(22.0))
        .child(
            View::text("Internal architecture module status and diagnostics")
                .font_size(12.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(16.0))
        .child(
            View::group_box("BaseProxy").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text(status_label).font_size(12.0).bold())
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::table_view(
                            vec![
                                "Protocol".to_string(),
                                "Status".to_string(),
                                "Sessions".to_string(),
                            ],
                            vec![
                                vec![
                                    "HTTP CONNECT".to_string(),
                                    if running { "Enabled" } else { "Standby" }.to_string(),
                                    if running {
                                        event_count.to_string()
                                    } else {
                                        "—".to_string()
                                    },
                                ],
                                vec![
                                    "SOCKS5".to_string(),
                                    if running { "Enabled" } else { "Standby" }.to_string(),
                                    "—".to_string(),
                                ],
                            ],
                        ),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::text(
                            status
                                .listen_addr
                                .as_deref()
                                .map(|a| format!("Listening on {a}"))
                                .unwrap_or_else(|| "Not listening".to_string()),
                        )
                        .font_size(11.0)
                        .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("StreamingLayer").child(
                View::table_view(
                    vec![
                        "Protocol".to_string(),
                        "Support".to_string(),
                        "Notes".to_string(),
                    ],
                    vec![
                        vec![
                            "HTTP/2".to_string(),
                            "Passthrough".to_string(),
                            "CONNECT tunnel preserves h2".to_string(),
                        ],
                        vec![
                            "WebSocket".to_string(),
                            "Passthrough".to_string(),
                            "Upgrade headers forwarded".to_string(),
                        ],
                        vec![
                            "SSE".to_string(),
                            "Passthrough".to_string(),
                            "Chunked transfer preserved".to_string(),
                        ],
                    ],
                ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("ProtocolModules").child(
                View::vstack()
                    .child(
                        View::table_view(
                            vec![
                                "Module".to_string(),
                                "License".to_string(),
                                "Status".to_string(),
                                "Build".to_string(),
                            ],
                            vec![
                                vec![
                                    "Shadowsocks".to_string(),
                                    "MIT".to_string(),
                                    "Planned".to_string(),
                                    "—".to_string(),
                                ],
                                vec![
                                    "Trojan".to_string(),
                                    "MIT".to_string(),
                                    "Planned".to_string(),
                                    "—".to_string(),
                                ],
                                vec![
                                    "VMess".to_string(),
                                    "Clean-room".to_string(),
                                    "Planned (D07)".to_string(),
                                    "—".to_string(),
                                ],
                                vec![
                                    "VLESS".to_string(),
                                    "Clean-room".to_string(),
                                    "Planned (D07)".to_string(),
                                    "—".to_string(),
                                ],
                                vec![
                                    "WireGuard".to_string(),
                                    "MIT".to_string(),
                                    "Future".to_string(),
                                    "—".to_string(),
                                ],
                            ],
                        ),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::text("Protocol modules follow MIT / clean-room licensing strategy (D01)")
                            .font_size(10.0)
                            .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Observability").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("Event pipeline:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(
                                View::text(if running { "● Active" } else { "○ Idle" })
                                    .font_size(12.0)
                                    .bold(),
                            )
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(4.0))
                    .child(
                        View::text(&format!("Total events emitted: {event_count}"))
                            .font_size(11.0),
                    )
                    .child(
                        View::text("Buffer: 200 events in-memory ring")
                            .font_size(10.0)
                            .foreground("secondaryLabelColor"),
                    ),
            ),
        )
}
