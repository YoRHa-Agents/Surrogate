use crate::dispatcher::AppController;
use crate::event_format::event_detail_line;
use cocoanut::prelude::*;
use surrogate_contract::events::EventKind;

pub fn build(controller: &AppController) -> View {
    let events = controller.recent_events();
    let (total, sessions, errors) = controller.event_counts();

    let category_rows: Vec<Vec<String>> = vec![
        vec![
            "SessionStarted".to_string(),
            sessions.to_string(),
            "New proxy sessions opened".to_string(),
        ],
        vec![
            "RuleMatched".to_string(),
            events
                .iter()
                .filter(|e| e.kind == EventKind::RuleMatched)
                .count()
                .to_string(),
            "Routing rules matched".to_string(),
        ],
        vec![
            "ForwardConnected".to_string(),
            events
                .iter()
                .filter(|e| e.kind == EventKind::ForwardConnected)
                .count()
                .to_string(),
            "Upstream connections established".to_string(),
        ],
        vec![
            "ForwardClosed".to_string(),
            events
                .iter()
                .filter(|e| e.kind == EventKind::ForwardClosed)
                .count()
                .to_string(),
            "Upstream connections closed".to_string(),
        ],
        vec![
            "Error".to_string(),
            errors.to_string(),
            "Processing errors".to_string(),
        ],
    ];

    let mut event_list = View::vstack();
    if events.is_empty() {
        event_list = event_list.child(
            View::text("No events yet — proxy traffic will appear here")
                .font_size(11.0)
                .foreground("secondaryLabelColor"),
        );
    } else {
        for e in events.iter().take(50) {
            let line = event_detail_line(e);
            let kind_prefix = match e.kind {
                EventKind::Error => "✗ ",
                EventKind::SessionStarted => "→ ",
                EventKind::ForwardConnected => "↔ ",
                EventKind::ForwardClosed => "← ",
                EventKind::RuleMatched => "◆ ",
            };
            event_list = event_list.child(
                View::text(&format!("{kind_prefix}{line}")).font_size(10.0),
            );
        }
    }

    View::vstack()
        .child(View::text("Observe").bold().font_size(22.0))
        .child(
            View::text("Real-time traffic and event monitoring")
                .font_size(12.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Event Summary").child(
                View::hstack()
                    .child(stat_pill("Total", total))
                    .child(View::spacer().width(12.0))
                    .child(stat_pill("Sessions", sessions))
                    .child(View::spacer().width(12.0))
                    .child(stat_pill("Errors", errors))
                    .child(View::spacer()),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Event Categories").child(
                View::table_view(
                    vec![
                        "Kind".to_string(),
                        "Count".to_string(),
                        "Description".to_string(),
                    ],
                    category_rows,
                ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Event Stream (newest first)").child(
                View::scroll_view().child(event_list).height(300.0),
            ),
        )
}

fn stat_pill(label: &str, value: usize) -> View {
    View::vstack()
        .child(View::text(&value.to_string()).bold().font_size(20.0))
        .child(
            View::text(label)
                .font_size(10.0)
                .foreground("secondaryLabelColor"),
        )
        .padding(6.0)
}
