use crate::dispatcher::AppController;
use cocoanut::prelude::*;

struct PluginSpec {
    name: &'static str,
    capabilities: &'static str,
    enabled: bool,
    version: &'static str,
    description: &'static str,
}

const BUILTIN_PLUGINS: &[PluginSpec] = &[
    PluginSpec {
        name: "core-bootstrap",
        capabilities: "ProxyBootstrap, SystemProxy",
        enabled: true,
        version: "0.1.0",
        description: "Core proxy lifecycle and system proxy management",
    },
    PluginSpec {
        name: "diagnostics",
        capabilities: "Diagnostic, HealthCheck",
        enabled: true,
        version: "0.1.0",
        description: "Built-in diagnostic test suite (Identity, Transport, Streaming)",
    },
    PluginSpec {
        name: "region-risk",
        capabilities: "RegionRisk, GeoCheck",
        enabled: false,
        version: "0.1.0",
        description: "Region policy compliance and geo-restriction detection",
    },
    PluginSpec {
        name: "import-clash",
        capabilities: "ConfigImport",
        enabled: false,
        version: "0.1.0",
        description: "Clash / Mihomo configuration import and conversion",
    },
    PluginSpec {
        name: "import-singbox",
        capabilities: "ConfigImport",
        enabled: false,
        version: "0.1.0",
        description: "sing-box configuration import and conversion",
    },
];

pub fn build(_controller: &AppController) -> View {
    let plugin_rows: Vec<Vec<String>> = BUILTIN_PLUGINS
        .iter()
        .map(|p| {
            vec![
                p.name.to_string(),
                p.version.to_string(),
                p.capabilities.to_string(),
                if p.enabled { "● On" } else { "○ Off" }.to_string(),
            ]
        })
        .collect();

    let mut page = View::vstack()
        .child(View::text("Plugins").bold().font_size(22.0))
        .child(
            View::text("Extension modules for protocol support, import, and diagnostics")
                .font_size(12.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(16.0))
        .child(
            View::group_box("Installed Plugins").child(
                View::table_view(
                    vec![
                        "Plugin".to_string(),
                        "Version".to_string(),
                        "Capabilities".to_string(),
                        "State".to_string(),
                    ],
                    plugin_rows,
                ),
            ),
        )
        .child(View::spacer().height(12.0));

    for spec in BUILTIN_PLUGINS {
        page = page.child(build_plugin_detail(spec));
        page = page.child(View::spacer().height(8.0));
    }

    page = page.child(View::spacer().height(12.0)).child(
        View::group_box("Plugin Registry").child(
            View::vstack()
                .child(
                    View::hstack()
                        .child(View::search_field("Search available plugins…").width(300.0))
                        .child(View::spacer().width(8.0))
                        .child(View::button("Refresh"))
                        .child(View::spacer()),
                )
                .child(View::spacer().height(6.0))
                .child(
                    View::text("Plugin system uses the PluginRegistry from ControlPlane")
                        .font_size(10.0)
                        .foreground("secondaryLabelColor"),
                ),
        ),
    );

    page
}

fn build_plugin_detail(spec: &PluginSpec) -> View {
    View::group_box(spec.name).child(
        View::vstack()
            .child(
                View::hstack()
                    .child(
                        View::text(if spec.enabled { "● Enabled" } else { "○ Disabled" })
                            .font_size(12.0)
                            .bold(),
                    )
                    .child(View::spacer().width(12.0))
                    .child(View::text(&format!("v{}", spec.version)).font_size(11.0))
                    .child(View::spacer().width(12.0))
                    .child(
                        View::toggle(if spec.enabled {
                            "Disable"
                        } else {
                            "Enable"
                        }),
                    )
                    .child(View::spacer()),
            )
            .child(View::spacer().height(4.0))
            .child(View::text(spec.description).font_size(11.0))
            .child(View::spacer().height(2.0))
            .child(
                View::text(&format!("Capabilities: {}", spec.capabilities))
                    .font_size(10.0)
                    .foreground("secondaryLabelColor"),
            ),
    )
}
