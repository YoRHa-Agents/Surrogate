use crate::dispatcher::AppController;
use crate::theme::{yorha_group_box, yorha_section_header, BODY, CAPTION, TITLE_LG};
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let running = controller.is_running();

    let active = if running { "ACTIVE" } else { "INACTIVE" };
    let standby = if running { "ACTIVE" } else { "STANDBY" };

    let mut page = View::vstack()
        .child(View::text("COMPONENTS").bold().font_size(TITLE_LG))
        .child(View::spacer().height(8.0))
        .child(
            View::text("Internal architecture module status.")
                .font_size(BODY),
        )
        .child(View::spacer().height(12.0));

    page = page.child(
        yorha_group_box("BASE PROXY").child(
            View::vstack()
                .child(View::table_view(
                    vec![
                        "PROTOCOL".to_string(),
                        "STATUS".to_string(),
                        "NOTES".to_string(),
                    ],
                    vec![
                        vec![
                            "HTTP CONNECT".to_string(),
                            standby.to_string(),
                            if running {
                                "Listening".to_string()
                            } else {
                                "Proxy stopped".to_string()
                            },
                        ],
                        vec![
                            "SOCKS5".to_string(),
                            standby.to_string(),
                            if running {
                                "Listening".to_string()
                            } else {
                                "Proxy stopped".to_string()
                            },
                        ],
                    ],
                )),
        ),
    );

    page = page.child(View::spacer().height(12.0));

    page = page.child(
        yorha_group_box("STREAMING LAYER").child(
            View::table_view(
                vec![
                    "PROTOCOL".to_string(),
                    "STATUS".to_string(),
                    "NOTES".to_string(),
                ],
                vec![
                    vec![
                        "HTTP/2".to_string(),
                        "IN PROGRESS".to_string(),
                        "CONNECT tunnel preserves h2".to_string(),
                    ],
                    vec![
                        "WebSocket".to_string(),
                        "PLANNED".to_string(),
                        "Upgrade headers forwarded".to_string(),
                    ],
                    vec![
                        "SSE".to_string(),
                        "PLANNED".to_string(),
                        "Chunked transfer preserved".to_string(),
                    ],
                ],
            ),
        ),
    );

    page = page.child(View::spacer().height(12.0));

    page = page.child(
        yorha_group_box("PROTOCOL MODULES").child(
            View::vstack()
                .child(View::table_view(
                    vec![
                        "MODULE".to_string(),
                        "STATUS".to_string(),
                        "NOTES".to_string(),
                    ],
                    vec![
                        vec![
                            "Shadowsocks".to_string(),
                            "SCAFFOLDED".to_string(),
                            "MIT license".to_string(),
                        ],
                        vec![
                            "Trojan".to_string(),
                            "SCAFFOLDED".to_string(),
                            "MIT license".to_string(),
                        ],
                        vec![
                            "WireGuard".to_string(),
                            "SCAFFOLDED".to_string(),
                            "MIT license".to_string(),
                        ],
                        vec![
                            "VMess".to_string(),
                            "SCAFFOLDED".to_string(),
                            "Clean-room (D07)".to_string(),
                        ],
                        vec![
                            "VLESS".to_string(),
                            "SCAFFOLDED".to_string(),
                            "Clean-room (D07)".to_string(),
                        ],
                    ],
                ))
                .child(View::spacer().height(4.0))
                .child(
                    View::text("Protocol modules follow MIT / clean-room licensing (D01)")
                        .font_size(CAPTION),
                ),
        ),
    );

    page
}

#[cfg(test)]
mod tests {
    #[test]
    fn components_page_compiles() {
        assert!(true);
    }
}
