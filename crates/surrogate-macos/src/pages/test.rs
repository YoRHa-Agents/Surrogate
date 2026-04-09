use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let running = controller.is_running();
    let outbounds = controller.outbounds();
    let exit_names: Vec<String> = outbounds.iter().map(|o| o.id.clone()).collect();

    View::vstack()
        .child(View::text("Test").bold().font_size(22.0))
        .child(
            View::text("Five-dimension diagnostic workbench")
                .font_size(12.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::hstack()
                .child(View::text("Target exit:").font_size(12.0))
                .child(View::spacer().width(8.0))
                .child(View::dropdown("Select outbound", exit_names.clone()))
                .child(View::spacer().width(16.0))
                .child(if running {
                    View::button("Run All Tests")
                } else {
                    View::text("Start proxy to enable tests")
                        .font_size(12.0)
                        .foreground("secondaryLabelColor")
                })
                .child(View::spacer()),
        )
        .child(View::spacer().height(16.0))
        .child(build_dimension(
            "Identity",
            &[
                ("IP / ASN Check", "Verify exit IP and autonomous system"),
                ("DNS Resolution", "Check for DNS leaks via exit"),
                ("WebRTC Leak", "Detect WebRTC IP exposure"),
            ],
        ))
        .child(View::spacer().height(8.0))
        .child(build_dimension(
            "Transport",
            &[
                ("TCP Reachability", "Connect to target hosts via proxy"),
                ("TLS Handshake", "Verify TLS negotiation through tunnel"),
                ("Latency", "Round-trip time measurement"),
                ("Bandwidth", "Throughput estimation"),
            ],
        ))
        .child(View::spacer().height(8.0))
        .child(build_dimension(
            "Streaming",
            &[
                ("HTTP/2", "Multiplexed stream test"),
                ("WebSocket", "Bidirectional stream test"),
                ("SSE", "Server-sent events passthrough"),
            ],
        ))
        .child(View::spacer().height(8.0))
        .child(build_dimension(
            "Tool Compatibility",
            &[
                ("Claude Code", "Anthropic API reachability"),
                ("Cursor", "IDE backend connectivity"),
                ("Copilot", "GitHub Copilot endpoint test"),
            ],
        ))
        .child(View::spacer().height(8.0))
        .child(build_dimension(
            "Risk Assessment",
            &[
                ("Region Risk", "Exit region vs. service policy match"),
                ("Egress Quality", "Packet loss, jitter, stability"),
                ("Site Reachability", "Critical domain group check"),
            ],
        ))
}

fn build_dimension(title: &str, checks: &[(&str, &str)]) -> View {
    let mut items = View::vstack();
    for (name, description) in checks {
        items = items.child(
            View::hstack()
                .child(View::text("○").font_size(11.0).width(16.0))
                .child(View::text(*name).font_size(12.0).bold().width(180.0))
                .child(
                    View::text(*description)
                        .font_size(11.0)
                        .foreground("secondaryLabelColor"),
                )
                .child(View::spacer())
                .child(View::button("Run").width(60.0)),
        );
    }
    View::group_box(title).child(items)
}
