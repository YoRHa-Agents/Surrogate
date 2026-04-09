use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(_controller: &AppController) -> View {
    View::vstack()
        .child(View::text("Apps").bold().font_size(20.0))
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("AppGroup").child(
                View::vstack()
                    .child(View::text("Default group (placeholder)").font_size(12.0))
                    .child(View::text("Drag apps into groups — not wired yet.").font_size(11.0)),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Search / filter")
                .child(View::text("Search by name or bundle ID — placeholder").font_size(12.0)),
        )
        .child(View::spacer().height(12.0))
        .child(View::text("Applications").bold().font_size(13.0))
        .child(
            View::group_box("Bindings (sample)").child(
                View::vstack()
                    .child(
                        View::text("com.apple.Safari          → DIRECT      active")
                            .font_size(11.0),
                    )
                    .child(
                        View::text("com.todesktop.Cursor        → ProxyUnit-A  idle")
                            .font_size(11.0),
                    )
                    .child(
                        View::text("com.openai.chat             → REJECT       idle")
                            .font_size(11.0),
                    ),
            ),
        )
}
