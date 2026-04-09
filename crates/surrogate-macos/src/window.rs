#![allow(unexpected_cfgs)]

use crate::dispatcher::{view_tags, AppController, UiMode};
use crate::navigation::{NavigationState, Page};
use crate::theme;
use cocoanut::prelude::*;

use std::sync::atomic::{AtomicUsize, Ordering};

static MAIN_WINDOW_PTR: AtomicUsize = AtomicUsize::new(0);

/// Called from a setup thread after the window is created.
/// Captures the NSWindow pointer and prevents it from being deallocated on close.
pub fn configure_main_window() {
    std::thread::sleep(std::time::Duration::from_millis(300));
    unsafe {
        let ns_app: *mut objc::runtime::Object =
            msg_send![class!(NSApplication), sharedApplication];
        let windows: *mut objc::runtime::Object = msg_send![ns_app, windows];
        let count: usize = msg_send![windows, count];
        if count > 0 {
            let win: *mut objc::runtime::Object =
                msg_send![windows, objectAtIndex: 0usize];
            if !win.is_null() {
                let _: () = msg_send![win, setReleasedWhenClosed: false];
                MAIN_WINDOW_PTR.store(win as usize, Ordering::Release);
            }
        }
    }
}

/// Re-show the main window (called from tray "Open Main Window" handler).
pub fn show_main_window() {
    let ptr = MAIN_WINDOW_PTR.load(Ordering::Acquire);
    if ptr == 0 {
        if let Err(e) = std::process::Command::new("osascript")
            .args([
                "-e",
                "tell application \"System Events\" to set frontmost of process \"Surrogate\" to true",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            eprintln!("[surrogate] fallback window activate failed: {e}");
        }
        return;
    }

    unsafe {
        let win = ptr as *mut objc::runtime::Object;
        let _: () = msg_send![
            win,
            makeKeyAndOrderFront: std::ptr::null::<objc::runtime::Object>()
        ];
        let ns_app: *mut objc::runtime::Object =
            msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
    }
}

pub fn run_app(controller: AppController) {
    let snap = controller.snapshot();
    let nav = NavigationState::default();

    let status_bar = build_status_bar(&snap);
    let sidebar = build_sidebar(&controller, &nav, snap.mode);
    let content = build_content_area(&controller, &nav, snap.mode);
    let inspector = build_inspector(&nav);

    let center_pane = View::vstack()
        .child(status_bar)
        .child(theme::yorha_divider())
        .child(content);

    let main_layout = View::split_view()
        .child(sidebar)
        .child(center_pane)
        .child(inspector);

    app("Surrogate")
        .size(1200.0, 800.0)
        .on_close_fn(|| {
            let ptr = MAIN_WINDOW_PTR.load(Ordering::Acquire);
            if ptr != 0 {
                unsafe {
                    let win = ptr as *mut objc::runtime::Object;
                    let _: () = msg_send![
                        win,
                        orderOut: std::ptr::null::<objc::runtime::Object>()
                    ];
                }
            } else {
                unsafe {
                    let ns_app: *mut objc::runtime::Object =
                        msg_send![class!(NSApplication), sharedApplication];
                    let win: *mut objc::runtime::Object = msg_send![ns_app, keyWindow];
                    if !win.is_null() {
                        let _: () = msg_send![win, setReleasedWhenClosed: false];
                        MAIN_WINDOW_PTR.store(win as usize, Ordering::Release);
                        let _: () = msg_send![
                            win,
                            orderOut: std::ptr::null::<objc::runtime::Object>()
                        ];
                    }
                }
            }
        })
        .build()
        .root(main_layout)
        .run()
        .expect("failed to run macOS application");
}

fn build_status_bar(snap: &crate::dispatcher::UiState) -> View {
    let proxy_indicator = if snap.running {
        format!(
            "● PROXY: {}",
            snap.listen_addr.as_deref().unwrap_or("Running")
        )
    } else {
        "○ PROXY: STOPPED".to_string()
    };

    let sys_proxy_text = if snap.system_proxy {
        "SYSTEM PROXY: ON"
    } else {
        "SYSTEM PROXY: OFF"
    };

    let uptime_text = snap
        .uptime_secs
        .map(|s| format!("UP: {}", format_uptime(s)))
        .unwrap_or_else(|| "UP: —".to_string());

    let error_display = if snap.error_count > 0 {
        format!("ERRORS: {}", snap.error_count)
    } else {
        "ERRORS: 0".to_string()
    };

    View::hstack()
        .child(
            View::text(&proxy_indicator)
                .font_size(theme::MICRO)
                .bold()
                .tag(view_tags::STATUS_PROXY),
        )
        .child(View::spacer().width(20.0))
        .child(
            View::text(sys_proxy_text)
                .font_size(theme::MICRO)
                .tag(view_tags::STATUS_SYS_PROXY),
        )
        .child(View::spacer().width(20.0))
        .child(
            View::text(&format!("MODE: {:?}", snap.mode).to_uppercase())
                .font_size(theme::MICRO)
                .tag(view_tags::STATUS_MODE),
        )
        .child(View::spacer())
        .child(
            View::text(&uptime_text)
                .font_size(theme::MICRO)
                .tag(view_tags::STATUS_UPTIME),
        )
        .child(View::spacer().width(16.0))
        .child(
            View::text(&format!("EVENTS: {}", snap.total_events))
                .font_size(theme::MICRO)
                .tag(view_tags::STATUS_EVENTS),
        )
        .child(View::spacer().width(16.0))
        .child(
            View::text(&error_display)
                .font_size(theme::MICRO)
                .tag(view_tags::STATUS_ERRORS),
        )
        .padding(6.0)
}

fn build_sidebar(controller: &AppController, nav: &NavigationState, mode: UiMode) -> View {
    let groups = nav.visible_groups(mode);

    let mut sidebar = View::vstack()
        .child(
            View::text("SURROGATE")
                .bold()
                .font_size(theme::TITLE_MD),
        )
        .child(
            View::text(&format!("v{}", env!("CARGO_PKG_VERSION")))
                .font_size(theme::CAPTION),
        )
        .child(View::spacer().height(20.0))
        .child(theme::yorha_section_header("NAVIGATION"));

    for group in &groups {
        let is_active = *group == nav.active_group;
        let label = group.label().to_uppercase();

        let indicator = if is_active { "▌ " } else { "   " };
        let display = format!("{indicator}{label}");

        let pages = group.pages();
        let sublabel = if pages.len() > 1 {
            let names: Vec<&str> = pages.iter().map(|p| p.label()).collect();
            format!("     {}", names.join(" · "))
        } else {
            String::new()
        };

        let ctrl = controller.clone();
        let grp = *group;
        let mut entry = View::vstack().child(
            View::button(&display)
                .font_size(theme::TITLE_SM)
                .on_click_fn(move || {
                    let _ = &ctrl;
                    let _ = grp;
                    // Navigation state update will be handled by reactive rebuild
                }),
        );
        if !sublabel.is_empty() {
            entry = entry.child(
                View::text(&sublabel)
                    .font_size(theme::CAPTION),
            );
        }
        sidebar = sidebar.child(entry);
        sidebar = sidebar.child(View::spacer().height(2.0));
    }

    sidebar = sidebar
        .child(View::spacer())
        .child(theme::yorha_section_header("STATUS"));

    let running = controller.is_running();
    let status_text = if running {
        "● RUNNING"
    } else {
        "○ STOPPED"
    };
    sidebar = sidebar.child(View::text(status_text).font_size(theme::BODY));

    let sp = controller.is_system_proxy_enabled();
    let sp_text = if sp {
        "◉ SYSTEM PROXY"
    } else {
        "○ SYSTEM PROXY"
    };
    sidebar = sidebar.child(View::text(sp_text).font_size(theme::BODY));

    sidebar = sidebar.child(
        View::text(&format!("MODE: {:?}", controller.ui_mode()).to_uppercase())
            .font_size(theme::BODY),
    );

    sidebar = sidebar
        .child(View::spacer().height(12.0))
        .child(
            View::text(&format!("v{}", env!("CARGO_PKG_VERSION")))
                .font_size(theme::MICRO),
        );

    sidebar.width(200.0).padding(12.0)
}

fn build_content_area(controller: &AppController, nav: &NavigationState, mode: UiMode) -> View {
    let active_group = nav.active_group;
    let group_pages: Vec<Page> = active_group
        .pages()
        .iter()
        .copied()
        .filter(|p| {
            if mode == UiMode::Simple && p.observe_needs_advanced() {
                return false;
            }
            true
        })
        .collect();

    let tab_labels: Vec<String> = group_pages
        .iter()
        .map(|p| p.label().to_uppercase())
        .collect();

    let mut tabs = View::tab_view(tab_labels);
    for page in &group_pages {
        let page_view = View::scroll_view().child(
            build_page(controller, *page).padding(16.0),
        );
        tabs = tabs.child(page_view);
    }

    tabs
}

fn build_inspector(nav: &NavigationState) -> View {
    let page = nav.active_page;
    let group = page.group();

    let purpose = page_purpose(page);
    let module_context = page_module_context(page);

    View::vstack()
        .child(theme::yorha_section_header("INSPECTOR"))
        .child(View::spacer().height(8.0))
        .child(
            theme::yorha_group_box("PAGE INFO").child(
                View::vstack()
                    .child(
                        View::text(&page.label().to_uppercase())
                            .bold()
                            .font_size(theme::TITLE_SM),
                    )
                    .child(View::spacer().height(4.0))
                    .child(
                        View::text(&format!("Group: {}", group.label().to_uppercase()))
                            .font_size(theme::CAPTION),
                    )
                    .child(View::spacer().height(8.0))
                    .child(View::text(purpose).font_size(theme::BODY)),
            ),
        )
        .child(View::spacer().height(12.0))
        .child(
            theme::yorha_group_box("MODULE CONTEXT").child(
                View::text(module_context).font_size(theme::BODY),
            ),
        )
        .child(View::spacer())
        .width(250.0)
        .padding(12.0)
}

fn page_purpose(page: Page) -> &'static str {
    match page {
        Page::Overview => "Central dashboard showing proxy state, traffic summary, alerts, and capability overview.",
        Page::AbilityLens => "Capability maturity projection across all proxy abilities.",
        Page::Apps => "Application binding management — per-app proxy policies and rule matching.",
        Page::Tools => "Agent/IDE tool templates with recommended proxy configurations.",
        Page::Profiles => "Outbound profiles, subscriptions, and egress strategy management.",
        Page::Rules => "Routing rule configuration, priority ordering, and conflict detection.",
        Page::Test => "Five-dimension diagnostic workbench for proxy capabilities.",
        Page::Observe => "Real-time event stream and session monitoring.",
        Page::Settings => "Proxy configuration, UI mode, and system proxy control.",
        Page::Components => "Architecture module status — BaseProxy, StreamingLayer, Protocols.",
        Page::Plugins => "Plugin registry management — enable, disable, inspect capabilities.",
        Page::ImportLab => "Configuration import from external proxy formats.",
        Page::EgressLab => "Outbound quality testing — latency, packet loss, connectivity.",
    }
}

fn page_module_context(page: Page) -> &'static str {
    match page {
        Page::Overview => "surrogate-app::ProxyManager, surrogate-control::AbilityLens",
        Page::AbilityLens => "surrogate-control::ability_lens",
        Page::Apps | Page::Rules => "surrogate-control::rule_compiler, surrogate-contract::config",
        Page::Tools => "surrogate-control::plugin_registry, surrogate-control::builtin_plugins",
        Page::Profiles => "surrogate-contract::config::OutboundConfig",
        Page::Test => "surrogate-control::test_workbench",
        Page::Observe => "surrogate-contract::events::Event",
        Page::Settings => "surrogate-contract::config, surrogate-app::system_proxy",
        Page::Components => "surrogate-kernel (BaseProxy, StreamingLayer)",
        Page::Plugins => "surrogate-control::plugin_registry",
        Page::ImportLab => "surrogate-control::import_engine",
        Page::EgressLab => "surrogate-control::test_workbench",
    }
}

fn build_page(controller: &AppController, page: Page) -> View {
    match page {
        Page::Overview => crate::pages::overview::build(controller),
        Page::AbilityLens => crate::pages::ability_lens::build(controller),
        Page::Apps => crate::pages::apps::build(controller),
        Page::Tools => crate::pages::tools::build(controller),
        Page::Profiles => crate::pages::profiles::build(controller),
        Page::Rules => crate::pages::rules::build(controller),
        Page::Test => crate::pages::test::build(controller),
        Page::Observe => crate::pages::observe::build(controller),
        Page::Settings => crate::pages::settings::build(controller),
        Page::Components => crate::pages::components::build(controller),
        Page::Plugins => crate::pages::plugins::build(controller),
        Page::ImportLab => crate::pages::import_lab::build(controller),
        Page::EgressLab => crate::pages::egress_lab::build(controller),
    }
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
