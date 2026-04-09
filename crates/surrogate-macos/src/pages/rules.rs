use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(_controller: &AppController) -> View {
    View::vstack()
        .child(View::text("Rules").bold().font_size(20.0))
        .child(View::spacer().height(12.0))
        .child(View::text("Priority order (placeholder)").font_size(12.0))
        .child(View::spacer().height(8.0))
        .child(
            View::group_box("Rule list").child(
                View::vstack()
                    .child(
                        View::text(
                            "10  DOMAIN-SUFFIX  example.com  → ProxyUnit-A  ⚠ possible overlap",
                        )
                        .font_size(11.0),
                    )
                    .child(
                        View::text("20  DOMAIN         api.example.com  → DIRECT").font_size(11.0),
                    )
                    .child(
                        View::text("30  IP-CIDR        10.0.0.0/8  → DIRECT").font_size(11.0),
                    ),
            ),
        )
        .child(View::spacer().height(8.0))
        .child(
            View::group_box("Default fallback").child(
                View::text("MATCH  *  → default outbound (highlighted in full UI)").font_size(12.0),
            ),
        )
}
