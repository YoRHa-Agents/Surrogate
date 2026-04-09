use crate::dispatcher::AppController;
use cocoanut::prelude::*;

fn plugin_row(name: &str, caps: &str, enabled: bool) -> View {
    let state = if enabled { "on" } else { "off" };
    View::text(&format!("{name}  [{state}]  {caps}")).font_size(11.0)
}

pub fn build(_controller: &AppController) -> View {
    View::vstack()
        .child(View::text("Plugins").bold().font_size(20.0))
        .child(View::spacer().height(12.0))
        .child(View::text("Installed plugins (placeholder)").font_size(12.0))
        .child(View::spacer().height(8.0))
        .child(
            View::group_box("List").child(
                View::vstack()
                    .child(plugin_row(
                        "core-bootstrap",
                        "ProxyBootstrap, Diagnostic",
                        true,
                    ))
                    .child(plugin_row("region-risk", "RegionRisk", false))
                    .child(
                        View::hstack()
                            .child(View::button("Enable"))
                            .child(View::button("Disable")),
                    ),
            ),
        )
}
