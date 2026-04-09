use crate::dispatcher::AppController;
use cocoanut::prelude::*;

struct ToolSpec {
    name: &'static str,
    category: &'static str,
    status: &'static str,
    risk: &'static str,
    recommended_exit: &'static str,
    notes: &'static str,
}

const TOOL_SPECS: &[ToolSpec] = &[
    ToolSpec {
        name: "Claude Code",
        category: "AI Agent",
        status: "Requires proxy",
        risk: "Region-sensitive",
        recommended_exit: "US/EU endpoint",
        notes: "Anthropic API — region restrictions apply",
    },
    ToolSpec {
        name: "Cursor",
        category: "AI IDE",
        status: "Compatible",
        risk: "Low",
        recommended_exit: "Any stable exit",
        notes: "Uses HTTPS for API calls; proxy-aware",
    },
    ToolSpec {
        name: "Codex",
        category: "AI Agent",
        status: "Requires proxy",
        risk: "Region-sensitive",
        recommended_exit: "US endpoint",
        notes: "OpenAI API — may require specific regions",
    },
    ToolSpec {
        name: "Copilot",
        category: "AI Assistant",
        status: "Compatible",
        risk: "Elevated",
        recommended_exit: "GitHub-region exit",
        notes: "GitHub Copilot — needs stable low-latency connection",
    },
    ToolSpec {
        name: "Gemini",
        category: "AI Agent",
        status: "Requires proxy",
        risk: "Region-sensitive",
        recommended_exit: "US/supported region",
        notes: "Google AI — geo-restricted in some regions",
    },
];

pub fn build(controller: &AppController) -> View {
    let default_ob = controller.default_outbound_id();
    let outbounds = controller.outbounds();
    let exit_options: Vec<String> = outbounds.iter().map(|o| o.id.clone()).collect();

    let mut page = View::vstack()
        .child(View::text("Tools").bold().font_size(22.0))
        .child(
            View::text("Agent & IDE proxy templates with risk assessment")
                .font_size(12.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(16.0))
        .child(
            View::hstack()
                .child(View::search_field("Search tools…").width(250.0))
                .child(View::spacer().width(12.0))
                .child(View::segmented_control(vec![
                    "All".to_string(),
                    "Configured".to_string(),
                    "Unconfigured".to_string(),
                ]))
                .child(View::spacer()),
        )
        .child(View::spacer().height(16.0));

    for spec in TOOL_SPECS {
        let card = build_tool_card(spec, &default_ob, &exit_options);
        page = page.child(card).child(View::spacer().height(8.0));
    }

    page
}

fn build_tool_card(
    spec: &ToolSpec,
    default_exit: &str,
    exit_options: &[String],
) -> View {
    let mut exits = vec![default_exit.to_string()];
    for opt in exit_options {
        if opt != default_exit {
            exits.push(opt.clone());
        }
    }

    View::group_box(spec.name).child(
        View::vstack()
            .child(
                View::hstack()
                    .child(View::text(&format!("[{}]", spec.category)).font_size(10.0).bold())
                    .child(View::spacer().width(12.0))
                    .child(View::text(&format!("Status: {}", spec.status)).font_size(12.0))
                    .child(View::spacer().width(12.0))
                    .child(View::text(&format!("Risk: {}", spec.risk)).font_size(12.0))
                    .child(View::spacer()),
            )
            .child(View::spacer().height(6.0))
            .child(
                View::hstack()
                    .child(View::text("Exit:").font_size(11.0))
                    .child(View::spacer().width(4.0))
                    .child(View::dropdown(spec.recommended_exit, exits))
                    .child(View::spacer().width(12.0))
                    .child(View::button("Test Connection"))
                    .child(View::spacer()),
            )
            .child(View::spacer().height(4.0))
            .child(
                View::text(spec.notes)
                    .font_size(10.0)
                    .foreground("secondaryLabelColor"),
            ),
    )
}
