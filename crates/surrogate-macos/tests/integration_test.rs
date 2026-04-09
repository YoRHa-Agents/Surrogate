//! Integration tests for dispatcher and navigation (runs on Linux CI; no Cocoa UI).

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use surrogate_macos_lib::dispatcher::{AppController, UiMode};
use surrogate_macos_lib::navigation::{NavigationState, Page, TaskGroup};

/// Same structure as `surrogate-app` `DEFAULT_CONFIG` (listen uses port 0 for tests to avoid bind clashes).
const TEST_SURROGATE_CONFIG: &str = r#"listen = "127.0.0.1:0"
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"

[[outbounds]]
id = "reject"
type = "reject"
"#;

fn unique_config_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("surrogate_macos_integration_{label}_{nanos}.toml"))
}

fn write_test_config(path: &std::path::Path) {
    std::fs::write(path, TEST_SURROGATE_CONFIG).expect("write temp config");
}

/// `AppController` uses `block_on` internally; run it on the blocking pool so this can be a
/// `#[tokio::test]` without nesting Tokio runtimes.
#[tokio::test]
async fn test_dispatcher_lifecycle() {
    tokio::task::spawn_blocking(|| {
        let path = unique_config_path("lifecycle");
        write_test_config(&path);
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        let controller = AppController::new(path.clone(), rt);

        assert!(!controller.is_running(), "proxy should start stopped");

        let addr = controller.start_proxy().expect("start_proxy should succeed");
        assert!(!addr.is_empty(), "listen address should be non-empty");
        assert!(controller.is_running());

        let status = controller.status();
        assert!(status.running, "status should report running");

        controller.stop_proxy().expect("stop_proxy should succeed");
        assert!(!controller.is_running());

        std::fs::remove_file(&path).expect("remove temp config");
    })
    .await
    .expect("spawn_blocking join failed");
}

#[test]
fn test_ui_mode_persistence() {
    let path = unique_config_path("uimode");
    write_test_config(&path);
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let controller = AppController::new(path.clone(), rt);

    controller.set_ui_mode(UiMode::Expert);
    assert_eq!(controller.ui_mode(), UiMode::Expert);

    std::fs::remove_file(&path).expect("remove temp config");
}

#[test]
fn test_navigation_mode_filtering() {
    let nav = NavigationState::default();

    let simple = nav.visible_groups(UiMode::Simple);
    assert_eq!(simple.len(), 4);
    assert_eq!(
        simple,
        vec![
            TaskGroup::Home,
            TaskGroup::Workflows,
            TaskGroup::Diagnose,
            TaskGroup::System,
        ]
    );

    let advanced = nav.visible_groups(UiMode::Advanced);
    assert_eq!(advanced.len(), 5);
    assert_eq!(
        advanced,
        vec![
            TaskGroup::Home,
            TaskGroup::Workflows,
            TaskGroup::Network,
            TaskGroup::Diagnose,
            TaskGroup::System,
        ]
    );

    let expert = nav.visible_groups(UiMode::Expert);
    assert_eq!(expert.len(), 6);
    assert_eq!(expert, TaskGroup::all().to_vec());
}

#[test]
fn test_navigation_group_pages() {
    assert_eq!(
        TaskGroup::Home.pages(),
        &[Page::Overview, Page::AbilityLens]
    );
    assert_eq!(
        TaskGroup::Workflows.pages(),
        &[Page::Apps, Page::Tools]
    );
    assert_eq!(
        TaskGroup::Network.pages(),
        &[Page::Profiles, Page::Rules]
    );
    assert_eq!(
        TaskGroup::Diagnose.pages(),
        &[Page::Test, Page::Observe]
    );
    assert_eq!(TaskGroup::System.pages(), &[Page::Settings]);
    assert_eq!(
        TaskGroup::Advanced.pages(),
        &[
            Page::Components,
            Page::Plugins,
            Page::ImportLab,
            Page::EgressLab,
        ]
    );
}

#[test]
fn test_config_save_validates() {
    let path = unique_config_path("savecfg");
    write_test_config(&path);
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let controller = AppController::new(path.clone(), rt);

    let err = controller
        .save_config_content("this is not [[valid]] toml {{{")
        .expect_err("invalid TOML should be rejected");
    assert!(
        err.contains("invalid TOML"),
        "unexpected error message: {err}"
    );

    std::fs::remove_file(&path).expect("remove temp config");
}

#[test]
fn test_all_pages_for_mode() {
    let simple_pages = NavigationState::all_pages_for_mode(UiMode::Simple);
    assert_eq!(simple_pages.len(), 7);
    assert_eq!(simple_pages[0], Page::Overview);
    assert_eq!(simple_pages[1], Page::AbilityLens);
    assert!(!simple_pages.contains(&Page::Profiles));
    assert!(!simple_pages.contains(&Page::Components));

    let expert_pages = NavigationState::all_pages_for_mode(UiMode::Expert);
    assert_eq!(expert_pages.len(), 13);
    assert!(expert_pages.contains(&Page::Profiles));
    assert!(expert_pages.contains(&Page::Components));
    assert!(expert_pages.contains(&Page::EgressLab));
}

#[test]
fn test_page_group_mapping() {
    assert_eq!(Page::Overview.group(), TaskGroup::Home);
    assert_eq!(Page::AbilityLens.group(), TaskGroup::Home);
    assert_eq!(Page::Apps.group(), TaskGroup::Workflows);
    assert_eq!(Page::Tools.group(), TaskGroup::Workflows);
    assert_eq!(Page::Profiles.group(), TaskGroup::Network);
    assert_eq!(Page::Rules.group(), TaskGroup::Network);
    assert_eq!(Page::Test.group(), TaskGroup::Diagnose);
    assert_eq!(Page::Observe.group(), TaskGroup::Diagnose);
    assert_eq!(Page::Settings.group(), TaskGroup::System);
    assert_eq!(Page::Components.group(), TaskGroup::Advanced);
    assert_eq!(Page::Plugins.group(), TaskGroup::Advanced);
    assert_eq!(Page::ImportLab.group(), TaskGroup::Advanced);
    assert_eq!(Page::EgressLab.group(), TaskGroup::Advanced);
}

#[test]
fn test_dispatcher_config_access() {
    let path = unique_config_path("cfgaccess");
    write_test_config(&path);
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let controller = AppController::new(path.clone(), rt);

    let doc = controller.config_document().expect("config_document should parse");
    assert_eq!(doc.listen, "127.0.0.1:0");
    assert_eq!(doc.default_outbound, "direct");
    assert_eq!(doc.outbounds.len(), 2);

    let outbounds = controller.outbounds();
    assert_eq!(outbounds.len(), 2);
    assert_eq!(outbounds[0].id, "direct");
    assert_eq!(outbounds[1].id, "reject");

    let rules = controller.rules();
    assert!(rules.is_empty());

    assert_eq!(controller.default_outbound_id(), "direct");

    let (total, sessions, errors) = controller.event_counts();
    assert_eq!(total, 0);
    assert_eq!(sessions, 0);
    assert_eq!(errors, 0);

    std::fs::remove_file(&path).expect("remove temp config");
}
