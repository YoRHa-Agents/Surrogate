use crate::dispatcher::AppController;
use crate::theme::{yorha_button, yorha_group_box, BODY, CAPTION, TITLE_LG};
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let outbounds = controller.outbounds();
    let default_ob = controller.default_outbound_id();

    let outbound_rows: Vec<Vec<String>> = outbounds
        .iter()
        .map(|o| {
            let default_marker = if o.id == default_ob {
                "\u{25CF}".to_string()
            } else {
                String::new()
            };
            vec![o.id.clone(), format!("{:?}", o.kind), default_marker]
        })
        .collect();

    let has_outbounds = !outbound_rows.is_empty();

    let mut page = View::vstack()
        .child(View::text("PROFILES").bold().font_size(TITLE_LG))
        .child(View::spacer().height(16.0));

    if has_outbounds {
        page = page.child(
            yorha_group_box("OUTBOUND PROFILES").child(View::table_view(
                vec![
                    "ID".to_string(),
                    "TYPE".to_string(),
                    "DEFAULT".to_string(),
                ],
                outbound_rows,
            )),
        );
    } else {
        page = page.child(
            yorha_group_box("OUTBOUND PROFILES").child(
                View::text("No outbound profiles configured.").font_size(BODY),
            ),
        );
    }

    page = page
        .child(View::spacer().height(12.0))
        .child(
            yorha_group_box("SUBSCRIPTIONS").child(
                View::hstack()
                    .child(View::text_field("Enter subscription URL\u{2026}").width(400.0))
                    .child(View::spacer().width(8.0))
                    .child(yorha_button("ADD").on_click_fn(|| {
                        eprintln!("[surrogate] subscription add clicked (scaffold)");
                    }))
                    .child(View::spacer()),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            yorha_group_box("EGRESS STRATEGY").child(
                View::dropdown(
                    "STRATEGY",
                    vec![
                        "Direct Priority".to_string(),
                        "Balanced".to_string(),
                        "Latency Optimized".to_string(),
                    ],
                )
                .font_size(CAPTION),
            ),
        );

    page
}
