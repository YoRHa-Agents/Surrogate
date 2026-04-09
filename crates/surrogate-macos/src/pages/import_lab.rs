use crate::dispatcher::AppController;
use crate::theme::{yorha_button, yorha_group_box, BODY, CAPTION, TITLE_LG};
use cocoanut::prelude::*;

pub fn build(_controller: &AppController) -> View {
    let mut page = View::vstack()
        .child(View::text("IMPORT LAB").bold().font_size(TITLE_LG))
        .child(View::spacer().height(8.0))
        .child(
            View::text("Import and convert configurations from other proxy tools.")
                .font_size(BODY),
        )
        .child(View::spacer().height(12.0));

    page = page.child(
        yorha_group_box("SOURCE FORMAT").child(
            View::vstack()
                .child(
                    View::hstack()
                        .child(View::text("FORMAT:").font_size(BODY))
                        .child(View::spacer().width(8.0))
                        .child(View::segmented_control(vec![
                            "Clash".to_string(),
                            "Surge".to_string(),
                            "V2Ray".to_string(),
                            "SingBox".to_string(),
                            "Custom".to_string(),
                        ]))
                        .child(View::spacer()),
                ),
        ),
    );

    page = page.child(View::spacer().height(12.0));

    page = page.child(
        yorha_group_box("INPUT").child(
            View::vstack()
                .child(
                    View::hstack()
                        .child(View::text("URL:").font_size(BODY))
                        .child(View::spacer().width(8.0))
                        .child(
                            View::text_field("Paste subscription URL…")
                                .width(400.0),
                        )
                        .child(View::spacer()),
                )
                .child(View::spacer().height(8.0))
                .child(
                    View::text("— or paste content below —")
                        .font_size(CAPTION),
                )
                .child(View::spacer().height(8.0))
                .child(
                    View::text_area("Paste configuration content here…")
                        .height(120.0),
                )
                .child(View::spacer().height(8.0))
                .child(
                    View::hstack()
                        .child(
                            yorha_button("PARSE").on_click_fn(|| {
                                eprintln!("[surrogate] import-lab: PARSE clicked (placeholder)");
                            }),
                        )
                        .child(View::spacer()),
                ),
        ),
    );

    page = page.child(View::spacer().height(12.0));

    page = page.child(
        yorha_group_box("IMPORT PREVIEW").child(View::table_view(
            vec![
                "ITEM".to_string(),
                "TYPE".to_string(),
                "STATUS".to_string(),
            ],
            vec![
                vec![
                    "Outbounds".to_string(),
                    "—".to_string(),
                    "Awaiting parse".to_string(),
                ],
                vec![
                    "Rules".to_string(),
                    "—".to_string(),
                    "Awaiting parse".to_string(),
                ],
                vec![
                    "Groups".to_string(),
                    "—".to_string(),
                    "Awaiting parse".to_string(),
                ],
            ],
        )),
    );

    page
}

#[cfg(test)]
mod tests {
    #[test]
    fn import_lab_compiles() {
        assert!(true);
    }
}
