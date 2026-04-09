use crate::dispatcher::{AppController, UiMode};
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let status = controller.status();
    let sys_proxy_enabled = controller.is_system_proxy_enabled();
    let mode = controller.ui_mode();

    let mut page = View::vstack()
        .child(View::text("Settings").bold().font_size(20.0))
        .child(View::spacer().height(16.0))
        .child(
            View::group_box("Proxy").child(
                View::vstack()
                    .child(
                        View::text(&format!("Config: {}", status.config_path)).font_size(12.0),
                    )
                    .child(
                        View::text(&format!(
                            "Listen: {}",
                            status.listen_addr.as_deref().unwrap_or("—")
                        ))
                        .font_size(12.0),
                    )
                    .child(
                        View::text(&format!(
                            "System Proxy: {}",
                            if sys_proxy_enabled { "Enabled" } else { "Disabled" }
                        ))
                        .font_size(12.0),
                    ),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("UI Mode").child(
                View::vstack()
                    .child(View::text(&format!("Current: {:?}", mode)).font_size(12.0))
                    .child(
                        View::hstack()
                            .child(View::button("Simple"))
                            .child(View::button("Advanced"))
                            .child(View::button("Expert")),
                    ),
            ),
        )
        .child(View::spacer().height(12.0));

    if mode == UiMode::Expert {
        page = page
            .child(
                View::group_box("Config Editor (Expert)")
                    .child(View::text("TOML editor placeholder").font_size(12.0)),
            )
            .child(View::spacer().height(12.0));
    }

    if let Some(cmd) = controller.export_command() {
        page = page.child(
            View::group_box("Export Command").child(View::text(&cmd).font_size(11.0)),
        );
    }

    page = page
        .child(View::spacer().height(16.0))
        .child(View::text(&format!("Surrogate v{}", env!("CARGO_PKG_VERSION"))).font_size(11.0));

    page
}
