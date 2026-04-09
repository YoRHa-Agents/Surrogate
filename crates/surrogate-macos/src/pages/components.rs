use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(_controller: &AppController) -> View {
    View::vstack()
        .child(View::text("Components").bold().font_size(20.0))
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("BaseProxy").child(
                View::vstack()
                    .child(View::text("HTTP CONNECT — enabled (placeholder)").font_size(12.0))
                    .child(View::text("SOCKS5 — enabled (placeholder)").font_size(12.0))
                    .child(View::text("Active connections: —").font_size(11.0)),
            ),
        )
        .child(View::spacer().height(8.0))
        .child(
            View::group_box("StreamingLayer").child(
                View::vstack()
                    .child(View::text("HTTP/2 — status placeholder").font_size(12.0))
                    .child(
                        View::text("WebSocket — streams / faults placeholder").font_size(12.0),
                    ),
            ),
        )
        .child(View::spacer().height(8.0))
        .child(
            View::group_box("ProtocolModules").child(
                View::vstack()
                    .child(
                        View::text("SS / Trojan / VMess / VLESS / WG — build flags TBD")
                            .font_size(11.0),
                    )
                    .child(
                        View::text("Interop / clean-room columns — placeholder").font_size(11.0),
                    ),
            ),
        )
}
