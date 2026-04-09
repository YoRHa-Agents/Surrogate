use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(_controller: &AppController) -> View {
    View::vstack()
        .child(View::text("Import Lab").bold().font_size(22.0))
        .child(
            View::text("Import and convert configurations from other proxy tools")
                .font_size(12.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(16.0))
        .child(
            View::group_box("Source Format").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("Select format:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(View::segmented_control(vec![
                                "Clash / Mihomo".to_string(),
                                "sing-box".to_string(),
                                "V2Ray / Xray".to_string(),
                            ]))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::text("Supports D09: Clash/Mihomo + sing-box + V2Ray/Xray parallel import")
                            .font_size(10.0)
                            .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Input").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("URL:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(View::text_field("Paste subscription URL…").width(400.0))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(8.0))
                    .child(View::text("— or —").font_size(11.0).foreground("tertiaryLabelColor"))
                    .child(View::spacer().height(8.0))
                    .child(
                        View::text_area("Paste configuration content here…").height(120.0),
                    )
                    .child(View::spacer().height(8.0))
                    .child(
                        View::hstack()
                            .child(View::button("Parse & Preview"))
                            .child(View::spacer().width(8.0))
                            .child(View::button("Import to Config"))
                            .child(View::spacer()),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Import Report").child(
                View::vstack()
                    .child(
                        View::table_view(
                            vec![
                                "Item".to_string(),
                                "Status".to_string(),
                                "Notes".to_string(),
                            ],
                            vec![
                                vec![
                                    "Outbounds".to_string(),
                                    "—".to_string(),
                                    "Run import to see results".to_string(),
                                ],
                                vec![
                                    "Rules".to_string(),
                                    "—".to_string(),
                                    "Rule conversion depends on format".to_string(),
                                ],
                                vec![
                                    "Groups".to_string(),
                                    "—".to_string(),
                                    "Strategy groups mapped to Surrogate model".to_string(),
                                ],
                            ],
                        ),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::text("M = Migrated · D = Degraded · I = Ignored")
                            .font_size(10.0)
                            .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Import Engine").child(
                View::text(
                    "ImportEngine converts external configs into Surrogate's ConfigDocument model. \
                     Unsupported features are flagged as Degraded; incompatible entries are Ignored.",
                )
                .font_size(11.0)
                .foreground("secondaryLabelColor"),
            ),
        )
}
