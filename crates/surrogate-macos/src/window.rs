use crate::dispatcher::{AppController, UiMode};
use crate::navigation::{NavigationState, Page, TaskGroup};
use cocoanut::prelude::*;

pub fn run_app(controller: AppController) {
    let mode = controller.ui_mode();
    let nav = NavigationState::default();

    let status_bar = build_status_bar(&controller);
    let sidebar = build_sidebar(&controller, &nav);
    let content = build_content_area(&controller, mode);

    let right_pane = View::vstack()
        .child(status_bar)
        .child(View::spacer().height(2.0))
        .child(content);

    let main_layout = View::split_view()
        .child(sidebar)
        .child(right_pane);

    app("Surrogate")
        .size(1200.0, 800.0)
        .build()
        .root(main_layout)
        .run()
        .expect("failed to run macOS application");
}

fn build_status_bar(controller: &AppController) -> View {
    let status = controller.status();
    let sys_proxy = controller.is_system_proxy_enabled();
    let mode = controller.ui_mode();

    let proxy_indicator = if status.running {
        format!(
            "● Proxy: {}",
            status.listen_addr.as_deref().unwrap_or("Running")
        )
    } else {
        "○ Proxy: Stopped".to_string()
    };

    let sys_proxy_text = if sys_proxy {
        "System Proxy: On"
    } else {
        "System Proxy: Off"
    };

    let uptime_text = status
        .uptime_secs
        .map(|s| format!("Up: {}", format_uptime(s)))
        .unwrap_or_default();

    View::hstack()
        .child(View::text(&proxy_indicator).font_size(12.0).bold())
        .child(View::spacer().width(16.0))
        .child(View::text(sys_proxy_text).font_size(11.0))
        .child(View::spacer().width(16.0))
        .child(View::text(&format!("Mode: {:?}", mode)).font_size(11.0))
        .child(View::spacer().width(16.0))
        .child(View::text(&uptime_text).font_size(11.0))
        .child(View::spacer())
        .child(
            View::text(&format!("Events: {}", status.total_events)).font_size(11.0),
        )
        .padding(8.0)
        .background("windowBackgroundColor")
}

fn build_sidebar(controller: &AppController, nav: &NavigationState) -> View {
    let mode = controller.ui_mode();
    let groups = nav.visible_groups(mode);
    let sys_proxy = controller.is_system_proxy_enabled();

    let mut sidebar = View::vstack()
        .child(View::text("Surrogate").bold().font_size(18.0))
        .child(
            View::text(&format!("v{}", env!("CARGO_PKG_VERSION")))
                .font_size(10.0)
                .foreground("secondaryLabelColor"),
        )
        .child(View::spacer().height(20.0))
        .child(View::text("NAVIGATION").font_size(9.0).bold().foreground("tertiaryLabelColor"));

    for group in &groups {
        let is_active = *group == nav.active_group;
        let label = if is_active {
            format!("▸ {}", group.label())
        } else {
            format!("  {}", group.label())
        };

        let page_count = group.pages().len();
        let sublabel = if page_count > 1 {
            let names: Vec<&str> = group.pages().iter().map(|p| p.label()).collect();
            names.join(" · ")
        } else {
            String::new()
        };

        let mut entry = View::vstack()
            .child(View::text(&label).font_size(13.0));
        if !sublabel.is_empty() {
            entry = entry.child(
                View::text(&format!("    {sublabel}"))
                    .font_size(10.0)
                    .foreground("secondaryLabelColor"),
            );
        }
        sidebar = sidebar.child(entry);
        sidebar = sidebar.child(View::spacer().height(4.0));
    }

    sidebar = sidebar
        .child(View::spacer())
        .child(View::spacer().height(12.0))
        .child(View::text("STATUS").font_size(9.0).bold().foreground("tertiaryLabelColor"))
        .child(View::spacer().height(4.0));

    let running = controller.is_running();
    let status_text = if running { "● Running" } else { "○ Stopped" };
    sidebar = sidebar.child(View::text(status_text).font_size(11.0));

    let sp_text = if sys_proxy { "◉ System Proxy" } else { "○ System Proxy" };
    sidebar = sidebar.child(View::text(sp_text).font_size(11.0));

    sidebar = sidebar.child(View::text(&format!("Mode: {:?}", mode)).font_size(11.0));

    sidebar.width(200.0).padding(12.0)
}

fn build_content_area(controller: &AppController, mode: UiMode) -> View {
    let pages = NavigationState::all_pages_for_mode(mode);
    let tab_labels: Vec<String> = pages.iter().map(|p| p.label().to_string()).collect();
    let mut tabs = View::tab_view(tab_labels);

    for page in &pages {
        let page_view = View::scroll_view().child(
            build_page(controller, *page).padding(16.0),
        );
        tabs = tabs.child(page_view);
    }

    tabs
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
