use crate::dispatcher::AppController;
use cocoanut::prelude::*;

fn tool_card(name: &str, status: &str, risk: &str) -> View {
    View::group_box(name).child(
        View::vstack()
            .child(View::text(&format!("Status: {status}")).font_size(12.0))
            .child(View::text(&format!("Risk: {risk}")).font_size(11.0)),
    )
}

pub fn build(_controller: &AppController) -> View {
    View::vstack()
        .child(View::text("Tools").bold().font_size(20.0))
        .child(View::spacer().height(12.0))
        .child(View::text("Tool templates (placeholders)").font_size(12.0))
        .child(View::spacer().height(8.0))
        .child(tool_card(
            "Claude Code",
            "not configured",
            "region assessment pending",
        ))
        .child(View::spacer().height(8.0))
        .child(tool_card("Cursor", "configured", "low — placeholder"))
        .child(View::spacer().height(8.0))
        .child(tool_card("Codex", "not configured", "unknown"))
        .child(View::spacer().height(8.0))
        .child(tool_card("Copilot", "exception", "elevated — placeholder"))
        .child(View::spacer().height(8.0))
        .child(tool_card("Gemini", "not configured", "pending review"))
}
