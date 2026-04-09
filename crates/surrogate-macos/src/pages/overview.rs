use crate::dispatcher::AppController;
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let status = controller.status();
    let running = status.running;

    let status_text = if running {
        format!(
            "Proxy Running — {}",
            status.listen_addr.as_deref().unwrap_or("unknown")
        )
    } else {
        "Proxy Stopped".to_string()
    };

    let default_outbound = &status.config_summary;

    let mut page = View::vstack()
        .child(View::text("Overview").bold().font_size(20.0))
        .child(View::spacer().height(16.0))
        .child(View::text(&status_text).font_size(14.0))
        .child(View::spacer().height(12.0))
        .child(
            View::group_box("Configuration")
                .child(View::text(default_outbound).font_size(12.0)),
        )
        .child(View::spacer().height(12.0));

    let mode = controller.ui_mode();
    page = page.child(View::text(&format!("UI Mode: {:?}", mode)).font_size(12.0));
    page = page.child(View::spacer().height(12.0));

    if let Some(uptime) = status.uptime_secs {
        let uptime_str = format_uptime(uptime);
        page = page.child(View::text(&format!("Uptime: {uptime_str}")).font_size(12.0));
    }

    page = page.child(
        View::text(&format!("Total Events: {}", status.total_events)).font_size(12.0),
    );

    page
}

fn format_uptime(secs: u64) -> String {
    if secs < 60 {
        return format!("{secs}s");
    }
    if secs < 3600 {
        return format!("{}m {}s", secs / 60, secs % 60);
    }
    let h = secs / 3600;
    format!("{h}h {}m", (secs % 3600) / 60)
}
