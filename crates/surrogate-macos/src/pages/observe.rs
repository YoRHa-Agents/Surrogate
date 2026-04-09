use crate::dispatcher::AppController;
use crate::event_format::event_detail_line;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let events = controller.recent_events();

    let mut list = View::vstack();
    if events.is_empty() {
        list = list.child(View::text("No events yet — traffic will appear here.").font_size(11.0));
    } else {
        for e in events.iter().take(32) {
            list = list.child(View::text(&event_detail_line(e)).font_size(10.0));
        }
    }

    View::vstack()
        .child(View::text("Observe").bold().font_size(20.0))
        .child(View::spacer().height(8.0))
        .child(
            View::text(&format!(
                "Recent events (newest first, max 50): {}",
                events.len()
            ))
            .font_size(12.0),
        )
        .child(View::spacer().height(8.0))
        .child(View::group_box("Event stream").child(list))
}
