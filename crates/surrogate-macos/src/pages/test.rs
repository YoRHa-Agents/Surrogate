use crate::dispatcher::AppController;
use cocoanut::prelude::*;

fn dim(title: &'static str, items: &[&'static str]) -> View {
    let mut col = View::vstack();
    for line in items {
        col = col.child(View::text(*line).font_size(11.0));
    }
    View::group_box(title).child(col)
}

pub fn build(_controller: &AppController) -> View {
    View::vstack()
        .child(View::text("Test").bold().font_size(20.0))
        .child(View::spacer().height(8.0))
        .child(View::text("Five-dimension panel (placeholders)").font_size(12.0))
        .child(View::spacer().height(8.0))
        .child(dim(
            "Identity",
            &[
                "IP / ASN check — not run",
                "DNS resolution — not run",
                "WebRTC leak — not run",
            ],
        ))
        .child(View::spacer().height(6.0))
        .child(dim(
            "Transport",
            &["TCP reachability", "TLS handshake", "Latency", "Bandwidth — all pending"],
        ))
        .child(View::spacer().height(6.0))
        .child(dim(
            "Streaming",
            &["HTTP/2", "WebSocket", "SSE — placeholders"],
        ))
        .child(View::spacer().height(6.0))
        .child(dim(
            "Tool",
            &["Runs tool-specific checks when templates are active — placeholder"],
        ))
        .child(View::spacer().height(6.0))
        .child(dim(
            "Risk",
            &["Region risk", "Egress quality", "Site reachability — placeholder"],
        ))
}
