use crate::dispatcher::UiMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskGroup {
    Home,
    Workflows,
    Network,
    Diagnose,
    System,
    Advanced,
}

impl TaskGroup {
    pub fn all() -> &'static [TaskGroup] {
        &[
            TaskGroup::Home,
            TaskGroup::Workflows,
            TaskGroup::Network,
            TaskGroup::Diagnose,
            TaskGroup::System,
            TaskGroup::Advanced,
        ]
    }

    pub fn visible_in(self, mode: UiMode) -> bool {
        match (self, mode) {
            (TaskGroup::Home, _) => true,
            (TaskGroup::Workflows, _) => true,
            (TaskGroup::Diagnose, _) => true,
            (TaskGroup::System, _) => true,
            (TaskGroup::Network, UiMode::Simple) => false,
            (TaskGroup::Network, _) => true,
            (TaskGroup::Advanced, UiMode::Expert) => true,
            (TaskGroup::Advanced, _) => false,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TaskGroup::Home => "Home",
            TaskGroup::Workflows => "Workflows",
            TaskGroup::Network => "Network",
            TaskGroup::Diagnose => "Diagnose",
            TaskGroup::System => "System",
            TaskGroup::Advanced => "Advanced",
        }
    }

    pub fn pages(self) -> &'static [Page] {
        match self {
            TaskGroup::Home => &[Page::Overview, Page::AbilityLens],
            TaskGroup::Workflows => &[Page::Apps, Page::Tools],
            TaskGroup::Network => &[Page::Profiles, Page::Rules],
            TaskGroup::Diagnose => &[Page::Test, Page::Observe],
            TaskGroup::System => &[Page::Settings],
            TaskGroup::Advanced => &[
                Page::Components,
                Page::Plugins,
                Page::ImportLab,
                Page::EgressLab,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    AbilityLens,
    Apps,
    Tools,
    Profiles,
    Rules,
    Test,
    Observe,
    Settings,
    Components,
    Plugins,
    ImportLab,
    EgressLab,
}

impl Page {
    pub fn label(self) -> &'static str {
        match self {
            Page::Overview => "Overview",
            Page::AbilityLens => "Ability Lens",
            Page::Apps => "Apps",
            Page::Tools => "Tools",
            Page::Profiles => "Profiles",
            Page::Rules => "Rules",
            Page::Test => "Test",
            Page::Observe => "Observe",
            Page::Settings => "Settings",
            Page::Components => "Components",
            Page::Plugins => "Plugins",
            Page::ImportLab => "Import Lab",
            Page::EgressLab => "Egress Lab",
        }
    }

    pub fn observe_needs_advanced(self) -> bool {
        matches!(self, Page::Observe)
    }
}

#[derive(Debug, Clone)]
pub struct NavigationState {
    pub active_group: TaskGroup,
    pub active_page: Page,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self {
            active_group: TaskGroup::Home,
            active_page: Page::Overview,
        }
    }
}

impl NavigationState {
    pub fn select_group(&mut self, group: TaskGroup) {
        self.active_group = group;
        self.active_page = group.pages()[0];
    }

    pub fn select_page(&mut self, page: Page) {
        self.active_page = page;
    }

    pub fn visible_groups(&self, mode: UiMode) -> Vec<TaskGroup> {
        TaskGroup::all()
            .iter()
            .copied()
            .filter(|g| g.visible_in(mode))
            .collect()
    }

    pub fn all_pages_for_mode(mode: UiMode) -> Vec<Page> {
        TaskGroup::all()
            .iter()
            .filter(|g| g.visible_in(mode))
            .flat_map(|g| g.pages().iter().copied())
            .collect()
    }
}

impl Page {
    pub fn group(self) -> TaskGroup {
        match self {
            Page::Overview | Page::AbilityLens => TaskGroup::Home,
            Page::Apps | Page::Tools => TaskGroup::Workflows,
            Page::Profiles | Page::Rules => TaskGroup::Network,
            Page::Test | Page::Observe => TaskGroup::Diagnose,
            Page::Settings => TaskGroup::System,
            Page::Components | Page::Plugins | Page::ImportLab | Page::EgressLab => {
                TaskGroup::Advanced
            }
        }
    }
}
