use crate::dispatcher::AppController;
use crate::event_format::parse_outbound_ids_from_summary;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let status = controller.status();
    let summary = status.config_summary.as_str();
    let default_line = summary
        .lines()
        .find(|l| l.starts_with("Default Outbound:"))
        .unwrap_or("Default Outbound: (unknown)");
    let ids = parse_outbound_ids_from_summary(summary);

    let mut nodes = View::vstack()
        .child(View::text("Nodes from config (outbounds)").bold().font_size(12.0));

    if ids.is_empty() {
        nodes = nodes.child(View::text("— start proxy to load outbounds —").font_size(11.0));
    } else {
        for id in &ids {
            nodes = nodes.child(
                View::text(&format!("{id}  [protocol: from config]  badge: placeholder"))
                    .font_size(11.0),
            );
        }
    }

    View::vstack()
        .child(View::text("Profiles").bold().font_size(20.0))
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Subscriptions")
                .child(View::text("No subscription URLs loaded (placeholder)").font_size(12.0)),
        )
        .child(View::spacer().height(8.0))
        .child(View::group_box("Nodes").child(nodes))
        .child(View::spacer().height(8.0))
        .child(
            View::group_box("ProxyUnit").child(
                View::vstack()
                    .child(View::text(default_line).font_size(12.0))
                    .child(View::text("Strategy / health UI — placeholder").font_size(11.0)),
            ),
        )
}
