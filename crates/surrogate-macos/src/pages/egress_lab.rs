use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let outbounds = controller.outbounds();
    let exit_names: Vec<String> = outbounds.iter().map(|o| o.id.clone()).collect();
    let running = controller.is_running();

    View::vstack()
        .child(View::text("Egress Lab").bold().font_size(22.0))
        .child(
            View::text("Outbound quality analysis, region risk, and site connectivity")
                .font_size(12.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(16.0))
        .child(
            View::hstack()
                .child(View::text("Test exit:").font_size(12.0))
                .child(View::spacer().width(8.0))
                .child(View::dropdown("Select outbound", exit_names.clone()))
                .child(View::spacer().width(16.0))
                .child(if running {
                    View::button("Run Egress Tests")
                } else {
                    View::text("Start proxy to test")
                        .font_size(12.0)
                        .foreground("secondaryLabelColor")
                })
                .child(View::spacer()),
        )
        .child(View::spacer().height(16.0))
        .child(
            View::group_box("Outbound Quality").child(
                View::table_view(
                    vec![
                        "Metric".to_string(),
                        "Value".to_string(),
                        "Rating".to_string(),
                    ],
                    vec![
                        vec![
                            "Latency".to_string(),
                            "— ms".to_string(),
                            "Run test".to_string(),
                        ],
                        vec![
                            "Packet Loss".to_string(),
                            "— %".to_string(),
                            "Run test".to_string(),
                        ],
                        vec![
                            "Jitter".to_string(),
                            "— ms".to_string(),
                            "Run test".to_string(),
                        ],
                        vec![
                            "Throughput".to_string(),
                            "— Mbps".to_string(),
                            "Run test".to_string(),
                        ],
                    ],
                ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Region Risk Assessment").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("Exit ASN / Geo:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(
                                View::text("Run test to identify")
                                    .font_size(12.0)
                                    .foreground("secondaryLabelColor"),
                            )
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::text(
                            "D10: Mixed data source for egress quality — \
                             ICMP, HTTP timing, DNS, and active probes",
                        )
                        .font_size(10.0)
                        .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Site Group Connectivity").child(
                View::table_view(
                    vec![
                        "Group".to_string(),
                        "Sites".to_string(),
                        "Reachable".to_string(),
                        "Status".to_string(),
                    ],
                    vec![
                        vec![
                            "AI Tools".to_string(),
                            "api.anthropic.com, api.openai.com, …".to_string(),
                            "—".to_string(),
                            "Not tested".to_string(),
                        ],
                        vec![
                            "Dev Resources".to_string(),
                            "github.com, npmjs.com, crates.io, …".to_string(),
                            "—".to_string(),
                            "Not tested".to_string(),
                        ],
                        vec![
                            "Cloud Infra".to_string(),
                            "aws.amazon.com, cloud.google.com, …".to_string(),
                            "—".to_string(),
                            "Not tested".to_string(),
                        ],
                    ],
                ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Target Site Groups (Editable)").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text_field("Group name").width(150.0))
                            .child(View::spacer().width(8.0))
                            .child(View::text_field("Sites (comma-separated)").width(300.0))
                            .child(View::spacer().width(8.0))
                            .child(View::button("Add Group"))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::text(
                            "TargetSiteGroup: group critical domains for reachability monitoring",
                        )
                        .font_size(10.0)
                        .foreground("secondaryLabelColor"),
                    ),
            ),
        )
}
