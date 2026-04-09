use crate::dispatcher::{AppController, UiMode};
use crate::theme;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let snap = controller.snapshot();

    let listen_display = snap
        .listen_addr
        .as_deref()
        .unwrap_or("Not running")
        .to_string();

    let ctrl_simple = controller.clone();
    let ctrl_advanced = controller.clone();
    let ctrl_expert = controller.clone();
    let ctrl_sp_toggle = controller.clone();

    let mut page = View::vstack()
        .child(View::text("SETTINGS").bold().font_size(theme::TITLE_LG))
        .child(View::spacer().height(16.0))
        .child(
            theme::yorha_group_box("PROXY CONFIGURATION").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("Listen address:").font_size(theme::BODY))
                            .child(View::spacer().width(8.0))
                            .child(View::text(&listen_display).font_size(theme::BODY))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::hstack()
                            .child(View::text("Default outbound:").font_size(theme::BODY))
                            .child(View::spacer().width(8.0))
                            .child(
                                View::text(&snap.default_outbound).font_size(theme::BODY),
                            )
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::hstack()
                            .child(View::text("Config file:").font_size(theme::BODY))
                            .child(View::spacer().width(8.0))
                            .child(
                                View::text(&snap.config_path)
                                    .font_size(theme::CAPTION)
                                    .foreground("secondaryLabelColor"),
                            )
                            .child(View::spacer()),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            theme::yorha_group_box("UI MODE").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("Current mode:").font_size(theme::BODY))
                            .child(View::spacer().width(8.0))
                            .child(
                                View::text(&format!("{:?}", snap.mode))
                                    .bold()
                                    .font_size(theme::BODY),
                            )
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(8.0))
                    .child(
                        View::hstack()
                            .child(theme::yorha_button("Simple").on_click_fn(move || {
                                ctrl_simple.set_ui_mode(UiMode::Simple);
                            }))
                            .child(View::spacer().width(8.0))
                            .child(
                                theme::yorha_button("Advanced").on_click_fn(move || {
                                    ctrl_advanced.set_ui_mode(UiMode::Advanced);
                                }),
                            )
                            .child(View::spacer().width(8.0))
                            .child(theme::yorha_button("Expert").on_click_fn(move || {
                                ctrl_expert.set_ui_mode(UiMode::Expert);
                            }))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::text(
                            "Simple: essential features · Advanced: + Network · Expert: all features",
                        )
                        .font_size(theme::CAPTION)
                        .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            theme::yorha_group_box("SYSTEM PROXY").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(
                                View::text("macOS system proxy:").font_size(theme::BODY),
                            )
                            .child(View::spacer().width(8.0))
                            .child(
                                View::text(if snap.system_proxy {
                                    "Enabled"
                                } else {
                                    "Disabled"
                                })
                                .bold()
                                .font_size(theme::BODY),
                            )
                            .child(View::spacer().width(16.0))
                            .child(
                                theme::yorha_button(if snap.system_proxy {
                                    "Disable"
                                } else {
                                    "Enable"
                                })
                                .on_click_fn(move || {
                                    let enabled = ctrl_sp_toggle.is_system_proxy_enabled();
                                    let _ = ctrl_sp_toggle.toggle_system_proxy(!enabled);
                                }),
                            )
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(4.0))
                    .child(
                        View::text(
                            "Controls HTTP/HTTPS/SOCKS proxy settings for all network services",
                        )
                        .font_size(theme::CAPTION)
                        .foreground("secondaryLabelColor"),
                    ),
            ),
        );

    if let Some(cmd) = controller.export_command() {
        page = page
            .child(View::spacer().height(12.0))
            .child(
                theme::yorha_group_box("EXPORT").child(
                    View::vstack()
                        .child(View::text(&cmd).font_size(theme::CAPTION))
                        .child(View::spacer().height(6.0))
                        .child(theme::yorha_button("Copy to Clipboard")),
                ),
            );
    }

    if snap.mode == UiMode::Expert {
        let config_content = controller
            .get_config_content()
            .unwrap_or_else(|_| "# Unable to load config".to_string());

        let ctrl_save = controller.clone();
        page = page
            .child(View::spacer().height(12.0))
            .child(
                theme::yorha_group_box("CONFIGURATION EDITOR").child(
                    View::vstack()
                        .child(View::text_area(&config_content).height(200.0))
                        .child(View::spacer().height(8.0))
                        .child(
                            View::hstack()
                                .child(
                                    theme::yorha_button("Save & Validate").on_click_fn(
                                        move || {
                                            if let Ok(content) = ctrl_save.get_config_content() {
                                                let _ =
                                                    ctrl_save.save_config_content(&content);
                                            }
                                        },
                                    ),
                                )
                                .child(View::spacer().width(8.0))
                                .child(theme::yorha_button("Reload from Disk"))
                                .child(View::spacer()),
                        ),
                ),
            );
    }

    page = page
        .child(View::spacer().height(16.0))
        .child(
            View::text(&format!("Surrogate v{}", env!("CARGO_PKG_VERSION")))
                .font_size(theme::CAPTION)
                .foreground("tertiaryLabelColor"),
        );

    page
}
