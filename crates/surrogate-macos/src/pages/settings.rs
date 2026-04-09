use crate::dispatcher::{AppController, UiMode};
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let status = controller.status();
    let sys_proxy_enabled = controller.is_system_proxy_enabled();
    let mode = controller.ui_mode();

    let ctrl_simple = controller.clone();
    let ctrl_advanced = controller.clone();
    let ctrl_expert = controller.clone();
    let ctrl_sp_toggle = controller.clone();

    let mut page = View::vstack()
        .child(View::text("Settings").bold().font_size(22.0))
        .child(View::spacer().height(16.0))
        .child(
            View::group_box("Proxy Configuration").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("Config file:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(
                                View::text(&status.config_path)
                                    .font_size(11.0)
                                    .foreground("secondaryLabelColor"),
                            )
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::hstack()
                            .child(View::text("Listen address:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(
                                View::text(
                                    status.listen_addr.as_deref().unwrap_or("Not running"),
                                )
                                .font_size(12.0),
                            )
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::hstack()
                            .child(View::text("Status:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(
                                View::text(if status.running { "● Running" } else { "○ Stopped" })
                                    .font_size(12.0)
                                    .bold(),
                            )
                            .child(View::spacer()),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("System Proxy").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("macOS system proxy:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(
                                View::text(if sys_proxy_enabled {
                                    "Enabled"
                                } else {
                                    "Disabled"
                                })
                                .bold()
                                .font_size(12.0),
                            )
                            .child(View::spacer().width(16.0))
                            .child(
                                View::button(if sys_proxy_enabled {
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
                        .font_size(10.0)
                        .foreground("secondaryLabelColor"),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("UI Mode").child(
                View::vstack()
                    .child(
                        View::hstack()
                            .child(View::text("Current mode:").font_size(12.0))
                            .child(View::spacer().width(8.0))
                            .child(View::text(&format!("{:?}", mode)).bold().font_size(12.0))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(8.0))
                    .child(
                        View::hstack()
                            .child(View::button("Simple").on_click_fn(move || {
                                ctrl_simple.set_ui_mode(UiMode::Simple);
                            }))
                            .child(View::spacer().width(8.0))
                            .child(View::button("Advanced").on_click_fn(move || {
                                ctrl_advanced.set_ui_mode(UiMode::Advanced);
                            }))
                            .child(View::spacer().width(8.0))
                            .child(View::button("Expert").on_click_fn(move || {
                                ctrl_expert.set_ui_mode(UiMode::Expert);
                            }))
                            .child(View::spacer()),
                    )
                    .child(View::spacer().height(6.0))
                    .child(
                        View::text(
                            "Simple: essential features · Advanced: + Network · Expert: all features",
                        )
                        .font_size(10.0)
                        .foreground("secondaryLabelColor"),
                    ),
            ),
        );

    if mode == UiMode::Expert {
        let config_content = controller
            .get_config_content()
            .unwrap_or_else(|_| "# Unable to load config".to_string());

        let ctrl_save = controller.clone();
        page = page
            .child(View::spacer().height(12.0))
            .child(
                View::group_box("Config Editor (Expert)").child(
                    View::vstack()
                        .child(View::text_area(&config_content).height(200.0))
                        .child(View::spacer().height(8.0))
                        .child(
                            View::hstack()
                                .child(View::button("Save & Validate").on_click_fn(move || {
                                    let _ = ctrl_save.get_config_content();
                                }))
                                .child(View::spacer().width(8.0))
                                .child(View::button("Reload from Disk"))
                                .child(View::spacer()),
                        ),
                ),
            );
    }

    if let Some(cmd) = controller.export_command() {
        page = page
            .child(View::spacer().height(12.0))
            .child(
                View::group_box("Export Command").child(
                    View::vstack()
                        .child(View::text(&cmd).font_size(11.0))
                        .child(View::spacer().height(6.0))
                        .child(View::button("Copy to Clipboard")),
                ),
            );
    }

    page = page
        .child(View::spacer().height(16.0))
        .child(
            View::text(&format!("Surrogate v{}", env!("CARGO_PKG_VERSION")))
                .font_size(11.0)
                .foreground("tertiaryLabelColor"),
        );

    page
}
