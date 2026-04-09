use crate::dispatcher::view_tags::OBSERVE_EVENT_COUNT;
use crate::dispatcher::AppController;
use crate::event_format::event_detail_line;
use crate::theme::{alloc_tag, yorha_group_box, yorha_stat_card, BODY, CAPTION, TITLE_LG};
use cocoanut::prelude::*;
use surrogate_contract::events::EventKind;

pub fn build(controller: &AppController) -> View {
    let events = controller.recent_events();

    let sessions = events
        .iter()
        .filter(|e| e.kind == EventKind::SessionStarted)
        .count();
    let rules_matched = events
        .iter()
        .filter(|e| e.kind == EventKind::RuleMatched)
        .count();
    let forwards = events
        .iter()
        .filter(|e| e.kind == EventKind::ForwardConnected)
        .count();
    let closed = events
        .iter()
        .filter(|e| e.kind == EventKind::ForwardClosed)
        .count();
    let errors = events
        .iter()
        .filter(|e| e.kind == EventKind::Error)
        .count();

    let mut page = View::vstack().child(View::text("OBSERVE").bold().font_size(TITLE_LG));

    if events.is_empty() {
        page = page.child(View::spacer().height(12.0)).child(
            View::text("No events recorded. Start the proxy to begin observing.")
                .font_size(BODY)
                .foreground("secondaryLabelColor"),
        );
        return page;
    }

    page = page.child(View::spacer().height(12.0)).child(
        yorha_group_box("Event Summary").child(
            View::hstack()
                .child(yorha_stat_card(
                    "Sessions",
                    &sessions.to_string(),
                    alloc_tag(),
                ))
                .child(View::spacer().width(8.0))
                .child(yorha_stat_card(
                    "Rules Matched",
                    &rules_matched.to_string(),
                    alloc_tag(),
                ))
                .child(View::spacer().width(8.0))
                .child(yorha_stat_card(
                    "Forwards",
                    &forwards.to_string(),
                    alloc_tag(),
                ))
                .child(View::spacer().width(8.0))
                .child(yorha_stat_card(
                    "Closed",
                    &closed.to_string(),
                    alloc_tag(),
                ))
                .child(View::spacer().width(8.0))
                .child(yorha_stat_card(
                    "Errors",
                    &errors.to_string(),
                    alloc_tag(),
                ))
                .child(View::spacer()),
        ),
    );

    let display_count = events.len().min(30);
    let mut event_list = View::vstack();
    for e in events.iter().take(30) {
        let line = event_detail_line(e);
        let kind_prefix = match e.kind {
            EventKind::Error => "✗ ",
            EventKind::SessionStarted => "→ ",
            EventKind::ForwardConnected => "↔ ",
            EventKind::ForwardClosed => "← ",
            EventKind::RuleMatched => "◆ ",
        };
        event_list = event_list.child(
            View::text(&format!("{kind_prefix}{line}")).font_size(CAPTION),
        );
    }

    page = page.child(View::spacer().height(12.0)).child(
        yorha_group_box("Event Stream").child(
            View::vstack()
                .child(
                    View::text(&format!(
                        "SHOWING {} OF {} EVENTS",
                        display_count,
                        events.len()
                    ))
                    .font_size(CAPTION)
                    .bold()
                    .tag(OBSERVE_EVENT_COUNT),
                )
                .child(View::spacer().height(4.0))
                .child(View::scroll_view().child(event_list).height(300.0)),
        ),
    );

    let category_rows: Vec<Vec<String>> = vec![
        vec!["SESSIONS".to_string(), sessions.to_string()],
        vec!["RULES MATCHED".to_string(), rules_matched.to_string()],
        vec!["FORWARDS".to_string(), forwards.to_string()],
        vec!["CLOSED".to_string(), closed.to_string()],
        vec!["ERRORS".to_string(), errors.to_string()],
    ];

    page = page.child(View::spacer().height(12.0)).child(
        yorha_group_box("Event Categories").child(View::table_view(
            vec!["CATEGORY".to_string(), "COUNT".to_string()],
            category_rows,
        )),
    );

    page
}
