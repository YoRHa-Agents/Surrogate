use crate::dispatcher::AppController;
use crate::navigation::{NavigationState, Page, TaskGroup};
use cocoanut::prelude::*;

pub fn run_app(controller: AppController) {
    let nav = NavigationState::default();

    let sidebar = build_sidebar(&controller, &nav);
    let content = build_content_area(&controller, &nav);

    let main_layout = View::split_view()
        .child(sidebar)
        .child(content);

    app("Surrogate")
        .size(960.0, 700.0)
        .build()
        .root(main_layout)
        .run()
        .expect("failed to run macOS application");
}

fn build_sidebar(controller: &AppController, nav: &NavigationState) -> View {
    let mode = controller.ui_mode();
    let groups = nav.visible_groups(mode);

    let mut sidebar = View::vstack()
        .child(View::text("Surrogate").bold().font_size(16.0))
        .child(View::spacer().height(16.0));

    for group in groups {
        let is_active = group == nav.active_group;
        let label = if is_active {
            format!("▸ {}", group.label())
        } else {
            format!("  {}", group.label())
        };
        sidebar = sidebar.child(View::button(&label).width(180.0));
    }

    sidebar = sidebar.child(View::spacer());

    let mode_label = format!("Mode: {:?}", mode);
    sidebar = sidebar.child(View::text(&mode_label).font_size(11.0));

    sidebar.width(200.0)
}

fn build_content_area(controller: &AppController, nav: &NavigationState) -> View {
    let pages = nav.active_group.pages();
    let mut content = View::vstack();

    if pages.len() > 1 {
        let mut tabs = View::hstack();
        for page in pages {
            let is_active = *page == nav.active_page;
            let label = if is_active {
                format!("[{}]", page.label())
            } else {
                page.label().to_string()
            };
            tabs = tabs.child(View::button(&label));
        }
        content = content.child(tabs);
        content = content.child(View::spacer().height(8.0));
    }

    let page_content = build_page(controller, nav.active_page);
    content = content.child(page_content);

    content
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
