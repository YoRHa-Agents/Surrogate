use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(_controller: &AppController) -> View {
    View::vstack()
        .child(View::text("Egress Lab").bold().font_size(20.0))
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Outbound quality").child(
                View::vstack()
                    .child(View::text("Latency / loss / jitter — placeholder").font_size(12.0))
                    .child(View::text("Last probe: —").font_size(11.0)),
            ),
        )
        .child(View::spacer().height(8.0))
        .child(
            View::group_box("Region risk")
                .child(View::text("Egress ASN / geo vs policy — placeholder").font_size(12.0)),
        )
        .child(View::spacer().height(8.0))
        .child(
            View::group_box("Site group connectivity").child(
                View::vstack()
                    .child(
                        View::text("Group \"AI tools\" — reachability placeholder")
                            .font_size(11.0),
                    )
                    .child(
                        View::text("Group \"Dev resources\" — reachability placeholder")
                            .font_size(11.0),
                    ),
            ),
        )
}
