//! Sidebar Component - Professional Monitoring Panel
//!
//! Sidebar حرفه‌ای مشابه OpenCode شامل:
//! - Task Panel
//! - Sessions Panel
//! - Models Panel
//! - Memory Panel

use crate::config::ThemeName;
use crate::tui::state::{Task, TaskStatus, Session, SessionState, TaskManager, SessionManager};
use crate::models::native::{NativeModelInfo, NativeProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::path::PathBuf;

/// نوع پنل‌های Sidebar
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SidebarPanel {
    Tasks,
    Sessions,
    Models,
    Memory,
}

impl SidebarPanel {
    pub fn name(&self) -> &str {
        match self {
            SidebarPanel::Tasks => "📋 Tasks",
            SidebarPanel::Sessions => "💻 Sessions",
            SidebarPanel::Models => "🤖 Models",
            SidebarPanel::Memory => "🧠 Memory",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            SidebarPanel::Tasks => "📋",
            SidebarPanel::Sessions => "💻",
            SidebarPanel::Models => "🤖",
            SidebarPanel::Memory => "🧠",
        }
    }
}

/// وضعیت Sidebar
#[derive(Clone)]
pub struct SidebarState {
    pub panels: Vec<SidebarPanel>,
    pub selected_panel: SidebarPanel,
    pub expanded_panels: Vec<SidebarPanel>,
    pub tasks_state: TasksPanelState,
    pub sessions_state: SessionsPanelState,
    pub models_state: ModelsPanelState,
    pub memory_state: MemoryPanelState,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            panels: vec![
                SidebarPanel::Tasks,
                SidebarPanel::Sessions,
                SidebarPanel::Models,
                SidebarPanel::Memory,
            ],
            selected_panel: SidebarPanel::Tasks,
            expanded_panels: vec![
                SidebarPanel::Tasks,
                SidebarPanel::Sessions,
            ],
            tasks_state: TasksPanelState::default(),
            sessions_state: SessionsPanelState::default(),
            models_state: ModelsPanelState::default(),
            memory_state: MemoryPanelState::default(),
        }
    }
}

impl SidebarState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_panel(&mut self) {
        let current_idx = self.panels.iter().position(|p| *p == self.selected_panel).unwrap_or(0);
        let next_idx = (current_idx + 1) % self.panels.len();
        self.selected_panel = self.panels[next_idx].clone();
    }

    pub fn prev_panel(&mut self) {
        let current_idx = self.panels.iter().position(|p| *p == self.selected_panel).unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.panels.len() - 1
        } else {
            current_idx - 1
        };
        self.selected_panel = self.panels[prev_idx].clone();
    }

    pub fn toggle_panel_expand(&mut self, panel: SidebarPanel) {
        if self.expanded_panels.contains(&panel) {
            self.expanded_panels.retain(|p| p != &panel);
        } else {
            self.expanded_panels.push(panel);
        }
    }

    pub fn is_panel_expanded(&self, panel: &SidebarPanel) -> bool {
        self.expanded_panels.contains(panel)
    }
}

/// وضعیت پنل Tasks
#[derive(Clone, Default)]
pub struct TasksPanelState {
    pub filter: TaskFilter,
    pub sort: TaskSort,
    pub selected_task: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub enum TaskFilter {
    All,
    InProgress,
    Pending,
    Completed,
}

#[derive(Clone, Debug, Default)]
pub enum TaskSort {
    Created,
    Updated,
    Priority,
    Title,
}

/// وضعیت پنل Sessions
#[derive(Clone, Default)]
pub struct SessionsPanelState {
    pub selected_session: Option<String>,
}

/// وضعیت پنل Models
#[derive(Clone, Default)]
pub struct ModelsPanelState {
    pub loaded_model: Option<String>,
    pub models: Vec<NativeModelInfo>,
}

/// وضعیت پنل Memory
#[derive(Clone, Default)]
pub struct MemoryPanelState {
    pub short_term_count: usize,
    pub long_term_count: usize,
    pub tokens_used: u64,
    pub tokens_limit: u64,
    pub cache_size: String,
}

/// مدیر Sidebar
#[derive(Clone)]
pub struct SidebarManager {
    pub state: Arc<RwLock<SidebarState>>,
    pub task_manager: Arc<TaskManager>,
    pub session_manager: Arc<SessionManager>,
}

impl SidebarManager {
    pub fn new(task_manager: Arc<TaskManager>, session_manager: Arc<SessionManager>) -> Self {
        Self {
            state: Arc::new(RwLock::new(SidebarState::new())),
            task_manager,
            session_manager,
        }
    }

    /// مدیریت ورودی کیبورد
    pub fn handle_key(&self, key: KeyEvent) -> bool {
        let mut state = self.state.write();

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.navigate_tasks(-1);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.navigate_tasks(1);
                true
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.prev_panel();
                true
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.next_panel();
                true
            }
            KeyCode::Enter => {
                self.toggle_selected();
                true
            }
            _ => false,
        }
    }

    fn navigate_tasks(&self, direction: i32) {
        let mut state = self.state.write();
        match state.selected_panel {
            SidebarPanel::Tasks => {
                let tasks = self.task_manager.get_all_tasks();
                let current_idx = state.tasks_state.selected_task.as_ref()
                    .and_then(|id| tasks.iter().position(|t| &t.id == id))
                    .unwrap_or(-1);
                
                let new_idx = ((current_idx as i32 + direction + tasks.len() as i32) % tasks.len() as i32) as usize;
                state.tasks_state.selected_task = Some(tasks[new_idx].id.clone());
            }
            SidebarPanel::Sessions => {
                let sessions = self.session_manager.get_all_sessions();
                let current_idx = state.sessions_state.selected_session.as_ref()
                    .and_then(|id| sessions.iter().position(|s| &s.id == id))
                    .unwrap_or(-1);
                
                if !sessions.is_empty() {
                    let new_idx = ((current_idx as i32 + direction + sessions.len() as i32) % sessions.len() as i32) as usize;
                    state.sessions_state.selected_session = Some(sessions[new_idx].id.clone());
                }
            }
            _ => {}
        }
    }

    fn next_panel(&self) {
        let mut state = self.state.write();
        state.next_panel();
    }

    fn prev_panel(&self) {
        let mut state = self.state.write();
        state.prev_panel();
    }

    fn toggle_selected(&self) {
        let mut state = self.state.write();
        state.toggle_panel_expand(state.selected_panel.clone());
    }

    /// به‌روزرسانی وضعیت مدل
    pub fn update_model_state(&self, loaded_model: Option<String>, models: Vec<NativeModelInfo>) {
        let mut state = self.state.write();
        state.models_state.loaded_model = loaded_model;
        state.models_state.models = models;
    }

    /// به‌روزرسانی وضعیت memory
    pub fn update_memory_state(&self, short: usize, long: usize, tokens: u64, cache: &str) {
        let mut state = self.state.write();
        state.memory_state.short_term_count = short;
        state.memory_state.long_term_count = long;
        state.memory_state.tokens_used = tokens;
        state.memory_state.cache_size = cache.to_string();
    }
}

/// رندر کردن Sidebar کامل
pub fn render_sidebar<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &SidebarManager,
    area: Rect,
    theme: &ThemeName,
) {
    let state = manager.state.read();
    
    // استایل‌ها بر اساس تم
    let (bg_color, fg_color, highlight_color, border_color) = match theme {
        ThemeName::DarkPlus => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan, Color::DarkGray),
        ThemeName::Light => (Color::Rgb(240, 240, 240), Color::Black, Color::Blue, Color::Gray),
        ThemeName::Monokai => (Color::Rgb(39, 40, 34), Color::White, Color::Yellow, Color::DarkGray),
        ThemeName::SolarizedDark => (Color::Rgb(0, 43, 54), Color::Rgb(131, 148, 150), Color::Cyan, Color::DarkGray),
        ThemeName::Dracula => (Color::Rgb(40, 42, 54), Color::Rgb(248, 248, 242), Color::Cyan, Color::DarkGray),
        _ => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan, Color::DarkGray),
    };

    // عنوان بالا
    let title = Line::from(" SUPER-AGENT ").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    
    let block = Block::default()
        .title(title)
        .borders(Borders::RIGHT)
        .style(Style::default().bg(bg_color).fg(fg_color));

    frame.render_widget(block, area);

    // محاسبه area داخلی
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width - 1,
        height: area.height,
    };

    // عنوان پنل‌ها در بالا
    let panel_titles: Vec<Line> = state.panels.iter().map(|p| {
        let is_selected = *p == state.selected_panel;
        let icon = if state.is_panel_expanded(p) { "▼" } else { "▶" };
        let name = p.name();
        Line::from(format!("{} {}", icon, name))
    }).collect();

    let tabs = Tabs::new(panel_titles)
        .select(state.panels.iter().position(|p| *p == state.selected_panel).unwrap_or(0))
        .style(Style::default().bg(bg_color).fg(fg_color))
        .highlight_style(Style::default().bg(highlight_color).fg(Color::Black))
        .divider(" | ");

    frame.render_widget(tabs, Rect {
        x: inner_area.x,
        y: inner_area.y,
        width: inner_area.width,
        height: 2,
    });

    // رندر پنل فعال
    let content_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 2,
        width: inner_area.width,
        height: inner_area.height - 2,
    };

    match state.selected_panel {
        SidebarPanel::Tasks => render_tasks_panel(frame, manager, content_area, theme, &state),
        SidebarPanel::Sessions => render_sessions_panel(frame, manager, content_area, theme, &state),
        SidebarPanel::Models => render_models_panel(frame, manager, content_area, theme, &state),
        SidebarPanel::Memory => render_memory_panel(frame, manager, content_area, theme, &state),
    }
}

/// رندر پنل Tasks
fn render_tasks_panel<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &SidebarManager,
    area: Rect,
    theme: &ThemeName,
    state: &SidebarState,
) {
    let tasks = manager.task_manager.get_all_tasks();
    
    let (bg_color, fg_color, highlight_color) = get_theme_colors(theme);
    
    // تفکیک task‌ها بر اساس وضعیت
    let in_progress: Vec<&Task> = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).collect();
    let pending: Vec<&Task> = tasks.iter().filter(|t| t.status == TaskStatus::Pending).collect();
    let completed: Vec<&Task> = tasks.iter().filter(|t| t.status == TaskStatus::Completed).collect();

    // ساخت محتوا
    let mut content = String::new();
    
    // In Progress
    content.push_str("🔄 IN PROGRESS\n");
    for task in &in_progress {
        let progress_bar = create_progress_bar(task.progress);
        content.push_str(&format!("  📌 {}\n", task.title));
        content.push_str(&format!("     {}\n", progress_bar));
        if let Some(agent) = &task.agent_name {
            content.push_str(&format!("     🤖 Agent: {}\n", agent));
        }
    }
    
    if in_progress.is_empty() {
        content.push_str("  (none)\n");
    }

    // Pending
    content.push_str("\n⏳ PENDING\n");
    for task in &pending {
        content.push_str(&format!("  📋 {}\n", task.title));
        if let Some(agent) = &task.agent_name {
            content.push_str(&format!("     🤖 Agent: {}\n", agent));
        }
    }
    
    if pending.is_empty() {
        content.push_str("  (none)\n");
    }

    // Completed
    content.push_str("\n✅ COMPLETED\n");
    for task in &completed {
        content.push_str(&format!("  ✓ {}\n", task.title));
    }
    
    if completed.is_empty() {
        content.push_str("  (none)\n");
    }

    let panel = Paragraph::new(content.trim())
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(panel, area);
}

/// رندر پنل Sessions
fn render_sessions_panel<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &SidebarManager,
    area: Rect,
    theme: &ThemeName,
    state: &SidebarState,
) {
    let sessions = manager.session_manager.get_all_sessions();
    let active_session = manager.session_manager.get_active_session();
    
    let (bg_color, fg_color, _) = get_theme_colors(theme);

    let mut content = String::new();
    
    for session in &sessions {
        let is_active = active_session.as_ref().map(|s| s.id == session.id).unwrap_or(false);
        let status_icon = match session.state {
            SessionState::Active => "🟢",
            SessionState::Background => "🟡",
            SessionState::Paused => "🔴",
            SessionState::Terminated => "⚫",
        };
        
        let prefix = if is_active { "👉" } else { "  " };
        content.push_str(&format!("{} {} {}\n", prefix, status_icon, session.name));
        content.push_str(&format!("   🤖 Model: {}\n", session.model_name));
        
        if let Some(task_id) = &session.current_task {
            content.push_str(&format!("   📌 Task: {}\n", task_id));
        }
        
        content.push_str("\n");
    }

    let panel = Paragraph::new(content.trim())
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(panel, area);
}

/// رندر پنل Models
fn render_models_panel<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &SidebarManager,
    area: Rect,
    theme: &ThemeName,
    state: &SidebarState,
) {
    let models = &state.models_state.models;
    let loaded = state.models_state.loaded_model.clone();
    
    let (bg_color, fg_color, _) = get_theme_colors(theme);

    let mut content = String::new();
    
    for model in models {
        let is_loaded = loaded.as_ref().map(|l| l == &model.name).unwrap_or(false);
        let status = if is_loaded { "🟢" } else { "⚪" };
        let name = if is_loaded { format!("{} [LOADED]", model.name) } else { model.name.clone() };
        
        content.push_str(&format!("{} {}\n", status, name));
        content.push_str(&format!("   📦 {} | {} bytes\n", model.format.display_name(), model.size));
    }

    if models.is_empty() {
        content.push_str("🤖 No models loaded\n");
        content.push_str("Press 'i' to import a model");
    }

    let panel = Paragraph::new(content.trim())
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(panel, area);
}

/// رندر پنل Memory
fn render_memory_panel<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &SidebarManager,
    area: Rect,
    theme: &ThemeName,
    state: &SidebarState,
) {
    let mem = &state.memory_state;
    let (bg_color, fg_color, _) = get_theme_colors(theme);

    let content = format!(
        "🧠 MEMORY STATISTICS\n\n\
        📊 SHORT-TERM\n\
        ├── Messages: {}\n\
        └── Tokens: used\n\n\
        📊 LONG-TERM\n\
        ├── Sessions: {}\n\
        └── Historical: stored\n\n\
        💾 CACHE\n\
        └── Size: {}\n\n\
        ⚡ PERFORMANCE\n\
        └── Index: active",
        mem.short_term_count,
        mem.long_term_count,
        mem.cache_size
    );

    let panel = Paragraph::new(content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(panel, area);
}

/// کمکی: دریافت رنگ‌های تم
fn get_theme_colors(theme: &ThemeName) -> (Color, Color, Color) {
    match theme {
        ThemeName::DarkPlus => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan),
        ThemeName::Light => (Color::Rgb(240, 240, 240), Color::Black, Color::Blue),
        ThemeName::Monokai => (Color::Rgb(39, 40, 34), Color::White, Color::Yellow),
        ThemeName::SolarizedDark => (Color::Rgb(0, 43, 54), Color::Rgb(131, 148, 150), Color::Cyan),
        ThemeName::Dracula => (Color::Rgb(40, 42, 54), Color::Rgb(248, 248, 242), Color::Cyan),
        _ => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan),
    }
}

/// کمکی: ساخت نوار پیشرفت
fn create_progress_bar(progress: f64) -> String {
    let total = 20;
    let filled = (progress * total as f64) as usize;
    let empty = total - filled;
    
    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(empty);
    
    format!("[{}{}] {:.0}%", filled_str, empty_str, progress * 100.0)
}
