//! Application State Management
//!
//! مدیریت وضعیت کل برنامه شامل task، session و تنظیمات

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Local};

/// نوع‌های شناسایی
pub type TaskId = String;
pub type SessionId = String;

/// وضعیت Task
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// اولویت Task
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// یک مرحله از Task
#[derive(Debug, Clone)]
pub struct TaskStep {
    pub title: String,
    pub completed: bool,
}

/// Task اصلی
#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub steps: Vec<TaskStep>,
    pub progress: f64, // 0.0 - 1.0
    pub agent_name: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Default for Task {
    fn default() -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: String::new(),
            description: String::new(),
            status: TaskStatus::Pending,
            priority: Priority::Medium,
            steps: Vec::new(),
            progress: 0.0,
            agent_name: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Task {
    pub fn new(title: &str) -> Self {
        let mut task = Self::default();
        task.title = title.to_string();
        task
    }

    pub fn update_progress(&mut self) {
        if self.steps.is_empty() {
            self.progress = match self.status {
                TaskStatus::Completed => 1.0,
                TaskStatus::InProgress => 0.5,
                _ => 0.0,
            };
        } else {
            let completed: usize = self.steps.iter().filter(|s| s.completed).count();
            self.progress = completed as f64 / self.steps.len() as f64;
        }
        self.updated_at = Local::now();
    }
}

/// وضعیت Session
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Active,
    Background,
    Paused,
    Terminated,
}

/// Session
#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub state: SessionState,
    pub current_task: Option<TaskId>,
    pub model_name: String,
    pub created_at: DateTime<Local>,
    pub last_active: DateTime<Local>,
}

impl Default for Session {
    fn default() -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: String::new(),
            state: SessionState::Active,
            current_task: None,
            model_name: String::new(),
            created_at: now,
            last_active: now,
        }
    }
}

impl Session {
    pub fn new(name: &str, model: &str) -> Self {
        let mut session = Self::default();
        session.name = name.to_string();
        session.model_name = model.to_string();
        session
    }
}

/// مدیر Task‌ها
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
    active_task: Arc<RwLock<Option<TaskId>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            active_task: Arc::new(RwLock::new(None)),
        }
    }

    pub fn add_task(&self, task: Task) {
        self.tasks.write().insert(task.id.clone(), task);
    }

    pub fn get_task(&self, id: &TaskId) -> Option<Task> {
        self.tasks.read().get(id).cloned()
    }

    pub fn get_all_tasks(&self) -> Vec<Task> {
        self.tasks.read().values().cloned().collect()
    }

    pub fn update_task(&self, id: &TaskId, mut task: Task) {
        task.update_progress();
        self.tasks.write().insert(id.clone(), task);
    }

    pub fn delete_task(&self, id: &TaskId) {
        self.tasks.write().remove(id);
    }

    pub fn set_active_task(&self, id: Option<TaskId>) {
        *self.active_task.write() = id;
    }

    pub fn get_active_task(&self) -> Option<Task> {
        self.active_task.read().as_ref().and_then(|id| self.get_task(id))
    }

    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<Task> {
        self.tasks.read()
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }
}

/// مدیر Session‌ها
#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    active_session: Arc<RwLock<Option<SessionId>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            active_session: Arc::new(RwLock::new(None)),
        }
    }

    pub fn add_session(&self, session: Session) {
        self.sessions.write().insert(session.id.clone(), session);
    }

    pub fn get_session(&self, id: &SessionId) -> Option<Session> {
        self.sessions.read().get(id).cloned()
    }

    pub fn get_all_sessions(&self) -> Vec<Session> {
        self.sessions.read().values().cloned().collect()
    }

    pub fn set_active_session(&self, id: Option<SessionId>) {
        *self.active_session.write() = id;
        if let Some(sid) = &id {
            if let Some(session) = self.sessions.write().get_mut(sid) {
                session.last_active = Local::now();
            }
        }
    }

    pub fn get_active_session(&self) -> Option<Session> {
        self.active_session.read().as_ref().and_then(|id| self.get_session(id))
    }

    pub fn switch_session(&self, from_id: SessionId, to_id: SessionId) {
        // Pause current session
        if let Some(session) = self.sessions.write().get_mut(&from_id) {
            session.state = SessionState::Paused;
        }
        // Activate new session
        if let Some(session) = self.sessions.write().get_mut(&to_id) {
            session.state = SessionState::Active;
            session.last_active = Local::now();
        }
        *self.active_session.write() = Some(to_id);
    }
}

/// تنظیمات UI
#[derive(Debug, Clone)]
pub struct UiSettings {
    pub show_sidebar: bool,
    pub show_status_bar: bool,
    pub show_tabs: bool,
    pub sidebar_width: u16,
    pub terminal_height: u16,
    pub font_size: u16,
    pub theme: String,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            show_status_bar: true,
            show_tabs: true,
            sidebar_width: 25,
            terminal_height: 20,
            font_size: 14,
            theme: String::from("DarkPlus"),
        }
    }
}

/// تنظیمات کیبورد
#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub command_palette: Vec<String>,
    pub new_task: Vec<String>,
    pub new_session: Vec<String>,
    pub save: Vec<String>,
    pub undo: Vec<String>,
    pub redo: Vec<String>,
    pub copy: Vec<String>,
    pub paste: Vec<String>,
    pub toggle_sidebar: Vec<String>,
    pub next_tab: Vec<String>,
    pub prev_tab: Vec<String>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            command_palette: vec!["Ctrl+n".to_string()],
            new_task: vec!["Ctrl+t".to_string()],
            new_session: vec!["Ctrl+Shift+n".to_string()],
            save: vec!["Ctrl+s".to_string()],
            undo: vec!["Ctrl+z".to_string()],
            redo: vec!["Ctrl+y".to_string()],
            copy: vec!["Ctrl+c".to_string()],
            paste: vec!["Ctrl+v".to_string()],
            toggle_sidebar: vec!["Ctrl+b".to_string()],
            next_tab: vec!["Ctrl+Tab".to_string()],
            prev_tab: vec!["Ctrl+Shift+Tab".to_string()],
        }
    }
}

/// تنظیمات کامل برنامه
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub ui: UiSettings,
    pub keybindings: KeyBindings,
    pub default_model: String,
    pub llm_endpoint: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ui: UiSettings::default(),
            keybindings: KeyBindings::default(),
            default_model: String::from("llama3.2"),
            llm_endpoint: String::from("http://127.0.0.1:8080"),
        }
    }
}
