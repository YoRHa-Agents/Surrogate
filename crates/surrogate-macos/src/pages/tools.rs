use crate::dispatcher::AppController;
use crate::theme;
use cocoanut::prelude::*;

struct ToolSpec {
    name: &'static str,
    description: &'static str,
    recommended_entry: &'static str,
    recommended_exit: &'static str,
    risk_level: &'static str,
}

const TOOL_SPECS: &[ToolSpec] = &[
    ToolSpec {
        name: "Claude Code",
        description: "Anthropic CLI agent — region restrictions apply, requires HTTP proxy for API access",
        recommended_entry: "HTTP",
        recommended_exit: "US/EU endpoint",
        risk_level: "REGION-SENSITIVE",
    },
    ToolSpec {
        name: "Cursor",
        description: "AI-powered IDE — uses HTTPS for API calls, proxy-aware with env variable support",
        recommended_entry: "HTTP",
        recommended_exit: "Any stable exit",
        risk_level: "LOW",
    },
    ToolSpec {
        name: "Codex CLI",
        description: "OpenAI CLI agent — may require specific regions for API access",
        recommended_entry: "HTTP",
        recommended_exit: "US endpoint",
        risk_level: "REGION-SENSITIVE",
    },
    ToolSpec {
        name: "GitHub Copilot",
        description: "GitHub AI assistant — needs stable low-latency connection to GitHub infrastructure",
        recommended_entry: "HTTP/SOCKS5",
        recommended_exit: "GitHub-region exit",
        risk_level: "ELEVATED",
    },
    ToolSpec {
        name: "Gemini Code Assist",
        description: "Google AI coding assistant — geo-restricted in some regions, requires proxy bypass",
        recommended_entry: "HTTP",
        recommended_exit: "US/supported region",
        risk_level: "REGION-SENSITIVE",
    },
    ToolSpec {
        name: "Remote Server Mode",
        description: "SSH/remote dev environments — tunnel all traffic through proxy for consistent access",
        recommended_entry: "SOCKS5",
        recommended_exit: "Nearest stable exit",
        risk_level: "LOW",
    },
];

pub fn build(controller: &AppController) -> View {
    let default_ob = controller.default_outbound_id();
    let outbounds = controller.outbounds();
    let exit_options: Vec<String> = outbounds.iter().map(|o| o.id.clone()).collect();

    let registry_rows: Vec<Vec<String>> = TOOL_SPECS
        .iter()
        .map(|s| {
            vec![
                s.name.to_string(),
                s.risk_level.to_string(),
                "CONFIGURED".to_string(),
                s.recommended_exit.to_string(),
            ]
        })
        .collect();

    let mut page = View::vstack()
        .child(View::text("TOOLS").bold().font_size(theme::TITLE_LG))
        .child(
            View::text("AGENT & IDE PROXY TEMPLATES WITH RISK ASSESSMENT")
                .font_size(theme::CAPTION)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(16.0));

    for spec in TOOL_SPECS {
        let card = build_tool_card(spec, &default_ob, &exit_options, controller);
        page = page.child(card).child(View::spacer().height(8.0));
    }

    page = page
        .child(View::spacer().height(8.0))
        .child(
            theme::yorha_group_box("TOOL REGISTRY").child(
                View::table_view(
                    vec![
                        "TOOL".to_string(),
                        "RISK".to_string(),
                        "STATUS".to_string(),
                        "EXIT".to_string(),
                    ],
                    registry_rows,
                )
                .height(180.0),
            ),
        );

    page
}

fn build_tool_card(
    spec: &ToolSpec,
    default_exit: &str,
    exit_options: &[String],
    controller: &AppController,
) -> View {
    let mut exits = vec![default_exit.to_string()];
    for opt in exit_options {
        if opt != default_exit {
            exits.push(opt.clone());
        }
    }

    let tool_name = spec.name.to_string();
    let ctrl = controller.clone();

    theme::yorha_group_box(spec.name).child(
        View::vstack()
            .child(
                View::hstack()
                    .child(
                        View::text(&spec.name.to_uppercase())
                            .bold()
                            .font_size(theme::TITLE_SM),
                    )
                    .child(View::spacer().width(12.0))
                    .child(
                        View::text(&format!("[{}]", spec.risk_level))
                            .font_size(theme::CAPTION)
                            .bold(),
                    )
                    .child(View::spacer()),
            )
            .child(View::spacer().height(4.0))
            .child(View::text(spec.description).font_size(theme::BODY))
            .child(View::spacer().height(6.0))
            .child(
                View::hstack()
                    .child(
                        View::text(&format!("ENTRY: {}", spec.recommended_entry))
                            .font_size(theme::CAPTION),
                    )
                    .child(View::spacer().width(16.0))
                    .child(View::text("EXIT:").font_size(theme::CAPTION))
                    .child(View::spacer().width(4.0))
                    .child(View::dropdown(spec.recommended_exit, exits))
                    .child(View::spacer()),
            )
            .child(View::spacer().height(6.0))
            .child(
                View::hstack()
                    .child(
                        theme::yorha_button("TEST CONNECTION").on_click_fn(move || {
                            let c = ctrl.clone();
                            let name = tool_name.clone();
                            std::thread::spawn(move || {
                                let running = c.is_running();
                                eprintln!(
                                    "[surrogate] TEST CONNECTION: {name} — proxy running: {running}"
                                );
                            });
                        }),
                    )
                    .child(View::spacer()),
            ),
    )
}
