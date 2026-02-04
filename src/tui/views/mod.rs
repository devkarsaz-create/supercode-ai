pub mod agents;
pub mod dashboard;
pub mod models;
pub mod settings;
pub mod tasks;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewId {
    Dashboard,
    Agents,
    Models,
    Tasks,
    Settings,
}

impl ViewId {
    pub fn all() -> [ViewId; 5] {
        [
            ViewId::Dashboard,
            ViewId::Agents,
            ViewId::Models,
            ViewId::Tasks,
            ViewId::Settings,
        ]
    }

    pub fn next(self) -> Self {
        match self {
            ViewId::Dashboard => ViewId::Agents,
            ViewId::Agents => ViewId::Models,
            ViewId::Models => ViewId::Tasks,
            ViewId::Tasks => ViewId::Settings,
            ViewId::Settings => ViewId::Dashboard,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            ViewId::Dashboard => "Mission Control",
            ViewId::Agents => "Agents",
            ViewId::Models => "Models",
            ViewId::Tasks => "Tasks",
            ViewId::Settings => "Settings",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ViewId::Dashboard => "dashboard",
            ViewId::Agents => "agents",
            ViewId::Models => "models",
            ViewId::Tasks => "tasks",
            ViewId::Settings => "settings",
        }
    }
}
