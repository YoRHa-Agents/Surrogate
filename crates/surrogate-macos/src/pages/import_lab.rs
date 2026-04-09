use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(_controller: &AppController) -> View {
    View::vstack()
        .child(View::text("Import Lab").bold().font_size(20.0))
        .child(View::spacer().height(12.0))
        .child(View::text("Source format").font_size(12.0))
        .child(
            View::hstack()
                .child(View::button("Clash / Mihomo"))
                .child(View::button("sing-box"))
                .child(View::button("V2Ray / Xray")),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Import").child(
                View::vstack()
                    .child(
                        View::text("Drop file, paste URL, or paste config — placeholder")
                            .font_size(11.0),
                    )
                    .child(View::button("Parse preview")),
            ),
        )
        .child(View::spacer().height(8.0))
        .child(
            View::group_box("Report").child(
                View::text("Success (M) / Degraded (D) / Ignored (I) — not run").font_size(11.0),
            ),
        )
}
