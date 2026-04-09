use crate::dispatcher::AppController;
use crate::theme::{yorha_button, yorha_group_box, BODY, CAPTION, TITLE_LG};
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let outbounds = controller.outbounds();
    let mut exit_names: Vec<String> = vec!["default".to_string()];
    for ob in &outbounds {
        if ob.id != "default" {
            exit_names.push(ob.id.clone());
        }
    }

    View::vstack()
        .child(View::text("TEST").bold().font_size(TITLE_LG))
        .child(View::spacer().height(4.0))
        .child(
            View::text("Five-dimension diagnostic workbench")
                .font_size(CAPTION)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::hstack()
                .child(View::text("TARGET OUTBOUND:").font_size(BODY).bold())
                .child(View::spacer().width(8.0))
                .child(View::dropdown("default", exit_names))
                .child(View::spacer().width(16.0))
                .child(yorha_button("Run All Tests").on_click_fn(|| {
                    std::thread::spawn(|| {
                        eprintln!("[surrogate-test] RUN ALL TESTS triggered (placeholder)");
                    });
                }))
                .child(View::spacer()),
        )
        .child(View::spacer().height(16.0))
        .child(build_dimension_box(
            "IDENTITY",
            "Verify exit identity and detect information leaks",
            &[
                (
                    "DNS RESOLUTION",
                    "Resolve domains through proxy to detect DNS leaks",
                ),
                (
                    "IP LEAK TEST",
                    "Confirm exit IP matches expected outbound address",
                ),
            ],
        ))
        .child(View::spacer().height(8.0))
        .child(build_dimension_box(
            "TRANSPORT",
            "Validate tunnel protocols and connection establishment",
            &[
                (
                    "TLS HANDSHAKE",
                    "Verify TLS negotiation through the tunnel",
                ),
                (
                    "HTTP CONNECT",
                    "Test HTTP CONNECT proxy tunnel method",
                ),
                (
                    "SOCKS5",
                    "Validate SOCKS5 handshake and data relay",
                ),
            ],
        ))
        .child(View::spacer().height(8.0))
        .child(build_dimension_box(
            "STREAMING",
            "Test multiplexed and bidirectional stream support",
            &[
                (
                    "HTTP/2 SUPPORT",
                    "Verify HTTP/2 multiplexed streams through proxy",
                ),
                (
                    "WEBSOCKET UPGRADE",
                    "Test WebSocket upgrade and bidirectional relay",
                ),
            ],
        ))
        .child(View::spacer().height(8.0))
        .child(build_dimension_box(
            "TOOL",
            "Verify reachability of AI development tool endpoints",
            &[
                (
                    "CLAUDE API ENDPOINT",
                    "Test connectivity to Anthropic API",
                ),
                (
                    "CURSOR BACKEND",
                    "Verify Cursor IDE backend reachability",
                ),
                (
                    "NPM REGISTRY",
                    "Check npm registry access through proxy",
                ),
            ],
        ))
        .child(View::spacer().height(8.0))
        .child(build_dimension_box(
            "RISK",
            "Assess exit region policy and certificate trust",
            &[
                (
                    "REGION DETECTION",
                    "Identify exit region and check service policy",
                ),
                (
                    "CERTIFICATE VALIDATION",
                    "Verify TLS certificate chain integrity",
                ),
            ],
        ))
}

fn build_dimension_box(title: &str, description: &str, checks: &[(&str, &str)]) -> View {
    let dim_label = title.to_string();

    let mut content = View::vstack()
        .child(
            View::hstack()
                .child(
                    View::text(description)
                        .font_size(CAPTION)
                        .foreground("secondaryLabelColor"),
                )
                .child(View::spacer())
                .child(yorha_button("Run").on_click_fn(move || {
                    let name = dim_label.clone();
                    std::thread::spawn(move || {
                        eprintln!("[surrogate-test] RUN {name} triggered (placeholder)");
                    });
                })),
        )
        .child(View::spacer().height(4.0))
        .child(
            View::text("PENDING")
                .font_size(CAPTION)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(8.0));

    for (name, desc) in checks {
        content = content.child(
            View::hstack()
                .child(View::text("○").font_size(CAPTION).width(16.0))
                .child(View::text(*name).font_size(BODY).bold().width(200.0))
                .child(
                    View::text(*desc)
                        .font_size(CAPTION)
                        .foreground("secondaryLabelColor"),
                )
                .child(View::spacer()),
        );
    }

    yorha_group_box(title).child(content)
}
