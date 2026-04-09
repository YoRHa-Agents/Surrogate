use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let outbounds = controller.outbounds();
    let default_ob = controller.default_outbound_id();
    let status = controller.status();

    let outbound_rows: Vec<Vec<String>> = outbounds
        .iter()
        .map(|o| {
            let is_default = if o.id == default_ob { "★" } else { "" };
            vec![
                o.id.clone(),
                format!("{:?}", o.kind),
                is_default.to_string(),
                "—".to_string(),
            ]
        })
        .collect();

    let nodes_empty = outbound_rows.is_empty();

    let mut page = View::vstack()
        .child(View::text("Profiles").bold().font_size(22.0))
        .child(
            View::text("Outbound transports, subscriptions, and proxy unit configuration")
                .font_size(12.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(16.0))
        .child(
            View::group_box("Subscriptions").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text_field("Enter subscription URL…").width(400.0))
                            .child(View::spacer().width(8.0))
                            .child(View::button("Add"))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(8.0))
                    .child(
                        View::text("No active subscriptions — add a URL to import nodes")
                            .font_size(11.0)
                            .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0));

    if nodes_empty {
        page = page.child(
            View::group_box("Nodes").child(
                View::text("Start proxy to load outbound configuration")
                    .font_size(12.0)
                    .foreground("secondaryLabelColor"),
            ),
        );
    } else {
        page = page.child(
            View::group_box("Nodes (Outbounds)").child(
                View::table_view(
                    vec![
                        "ID".to_string(),
                        "Type".to_string(),
                        "Default".to_string(),
                        "Latency".to_string(),
                    ],
                    outbound_rows,
                ),
            ),
        );
    }

    page = page
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Proxy Unit").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("Default outbound:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(View::text(&default_ob).bold().font_size(12.0))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(8.0))
                    .child(
                        View::hstack()
                            .child(View::text("Strategy:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(View::segmented_control(vec![
                                "Best Latency".to_string(),
                                "Round Robin".to_string(),
                                "Manual".to_string(),
                            ]))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(8.0))
                    .child(
                        View::text(&format!("Config: {}", status.config_path))
                            .font_size(10.0)
                            .foreground("tertiaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Protocol Summary").child(
                View::vstack()
                    .child(View::text("HTTP CONNECT — primary tunnel protocol").font_size(12.0))
                    .child(View::text("SOCKS5 — secondary tunnel protocol").font_size(12.0))
                    .child(
                        View::text("Additional protocols available via plugin system")
                            .font_size(11.0)
                            .foreground("secondaryLabelColor"),
                    ),
            ),
        );

    page
}
