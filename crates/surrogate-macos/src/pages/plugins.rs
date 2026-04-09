use crate::dispatcher::AppController;
use crate::theme::{yorha_group_box, BODY, CAPTION, TITLE_LG};
use cocoanut::prelude::*;

struct PluginSpec {
    name: &'static str,
    description: &'static str,
    capabilities: &'static [&'static str],
    enabled: bool,
}

const BUILTIN_PLUGINS: &[PluginSpec] = &[
    PluginSpec {
        name: "core-bootstrap",
        description: "Core proxy lifecycle and system proxy management",
        capabilities: &["ProxyBootstrap", "SystemProxy"],
        enabled: true,
    },
    PluginSpec {
        name: "diagnostics",
        description: "Built-in diagnostic test suite (Identity, Transport, Streaming)",
        capabilities: &["Diagnostic", "HealthCheck"],
        enabled: true,
    },
    PluginSpec {
        name: "region-risk",
        description: "Region policy compliance and geo-restriction detection",
        capabilities: &["RegionRisk", "GeoCheck"],
        enabled: true,
    },
    PluginSpec {
        name: "import-clash",
        description: "Clash / Mihomo configuration import and conversion",
        capabilities: &["ConfigImport"],
        enabled: true,
    },
    PluginSpec {
        name: "import-surge",
        description: "Surge configuration import and conversion",
        capabilities: &["ConfigImport"],
        enabled: true,
    },
];

fn build_plugin_card(spec: &PluginSpec) -> View {
    let badge = if spec.enabled { "[ENABLED]" } else { "[DISABLED]" };
    let caps = spec.capabilities.join(", ");

    yorha_group_box(spec.name).child(
        View::vstack()
            .child(
                View::hstack()
                    .child(View::text(badge).font_size(BODY).bold())
                    .child(View::spacer()),
            )
            .child(View::spacer().height(4.0))
            .child(View::text(spec.description).font_size(BODY))
            .child(View::spacer().height(4.0))
            .child(
                View::text(&format!("CAPABILITIES: {caps}"))
                    .font_size(CAPTION),
            ),
    )
}

pub fn build(_controller: &AppController) -> View {
    let mut page = View::vstack()
        .child(View::text("PLUGINS").bold().font_size(TITLE_LG))
        .child(View::spacer().height(8.0))
        .child(
            View::text("Extension modules for protocol support, import, and diagnostics.")
                .font_size(BODY),
        )
        .child(View::spacer().height(12.0));

    for spec in BUILTIN_PLUGINS {
        page = page.child(build_plugin_card(spec));
        page = page.child(View::spacer().height(8.0));
    }

    page = page.child(View::spacer().height(4.0));

    let registry_rows: Vec<Vec<String>> = BUILTIN_PLUGINS
        .iter()
        .map(|p| {
            vec![
                p.name.to_string(),
                if p.enabled { "ENABLED" } else { "DISABLED" }.to_string(),
                p.capabilities.join(", "),
            ]
        })
        .collect();

    page = page.child(
        yorha_group_box("PLUGIN REGISTRY").child(View::table_view(
            vec![
                "NAME".to_string(),
                "STATUS".to_string(),
                "CAPABILITIES".to_string(),
            ],
            registry_rows,
        )),
    );

    page
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_plugins_count() {
        assert_eq!(BUILTIN_PLUGINS.len(), 5);
    }

    #[test]
    fn all_plugin_names_non_empty() {
        for p in BUILTIN_PLUGINS {
            assert!(!p.name.is_empty());
            assert!(!p.description.is_empty());
            assert!(!p.capabilities.is_empty());
        }
    }

    #[test]
    fn expected_plugin_names() {
        let names: Vec<&str> = BUILTIN_PLUGINS.iter().map(|p| p.name).collect();
        assert!(names.contains(&"core-bootstrap"));
        assert!(names.contains(&"diagnostics"));
        assert!(names.contains(&"region-risk"));
        assert!(names.contains(&"import-clash"));
        assert!(names.contains(&"import-surge"));
    }
}
