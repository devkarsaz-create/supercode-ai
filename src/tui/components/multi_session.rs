//! Multi-Session Manager & Unified Memory System
//!
//! سیستم پیشرفته:
//! - Multi-session management
//! - Unified memory بین تمام agent‌ها
//! - Shared history
//! - Cross-session context sharing

use crate::tui::state::{Session, SessionState, SessionManager as BaseSessionManager, SessionId};
use crate::memory::store::{MemoryStore, Message};
use crate::types::Message as BaseMessage;
use chrono::{DateTime, Local};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

/// یک Session کامل
#[derive(Debug, Clone)]
pub struct ProfessionalSession {
    pub id: SessionId,
    pub name: String,
    pub state: SessionState,
    pub current_task: Option<String>,
    pub model_name: String,
    pub created_at: DateTime<Local>,
    pub last_active: DateTime<Local>,
    pub memory: Arc<MemoryStore>,
    pub history: SessionHistory,
    pub context: SharedContext,
    pub settings: SessionSettings,
    pub tabs: Vec<SessionTab>,
    pub active_tab: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SessionHistory {
    pub messages: VecDeque<HistoryEntry>,
    pub commands: VecDeque<String>,
    pub outputs: VecDeque<String>,
    pub max_entries: usize,
}

impl SessionHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            commands: VecDeque::new(),
            outputs: VecDeque::new(),
            max_entries,
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push_back(HistoryEntry {
            id: Uuid::new_v4().to_string(),
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Local::now(),
        });
        self.trim();
    }

    pub fn add_command(&mut self, command: &str) {
        self.commands.push_back(command.to_string());
        self.trim();
    }

    pub fn add_output(&mut self, output: &str) {
        self.outputs.push_back(output.to_string());
        self.trim();
    }

    fn trim(&mut self) {
        while self.messages.len() > self.max_entries {
            self.messages.pop_front();
        }
        while self.commands.len() > self.max_entries / 2 {
            self.commands.pop_front();
        }
        while self.outputs.len() > self.max_entries / 2 {
            self.outputs.pop_front();
        }
    }
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Local>,
}

/// Context مشترک بین Session‌ها
#[derive(Debug, Clone, Default)]
pub struct SharedContext {
    pub global_memory: Arc<MemoryStore>,
    pub shared_files: Vec<String>,
    pub shared_variables: HashMap<String, String>,
    pub cross_session_events: broadcast::Sender<SessionEvent>,
    pub event_history: VecDeque<SessionEvent>,
}

impl SharedContext {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            global_memory: Arc::new(MemoryStore::new()),
            shared_files: vec![],
            shared_variables: HashMap::new(),
            cross_session_events: sender,
            event_history: VecDeque::new(),
        }
    }

    pub fn share_variable(&mut self, key: &str, value: &str) {
        self.shared_variables.insert(key.to_string(), value.to_string());
    }

    pub fn get_variable(&self, key: &str) -> Option<String> {
        self.shared_variables.get(key).cloned()
    }

    pub fn broadcast_event(&mut self, event: SessionEvent) {
        let _ = self.cross_session_events.send(event.clone());
        self.event_history.push_back(event);
        if self.event_history.len() > 100 {
            self.event_history.pop_front();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    Message { from_session: String, to_session: Option<String>, content: String },
    TaskUpdate { session: String, task_id: String, status: String },
    VariableChange { key: String, old_value: Option<String>, new_value: Option<String> },
    FileChange { session: String, file: String, action: String },
    AgentAction { session: String, agent: String, action: String },
}

#[derive(Debug, Clone, Default)]
pub struct SessionSettings {
    pub auto_save: bool,
    pub save_interval: u64, // ثانیه
    pub context_limit: usize,
    pub output_capture: bool,
    pub agent_isolation: bool,
}

/// یک تب در session
#[derive(Debug, Clone)]
pub struct SessionTab {
    pub id: String,
    pub name: String,
    pub tab_type: TabType,
    pub content: String,
    pub modified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabType {
    Chat,
    Terminal,
    FileEditor,
    TaskView,
    MemoryView,
    Logs,
}

impl Default for SessionTab {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: String::new(),
            tab_type: TabType::Chat,
            content: String::new(),
            modified: false,
        }
    }
}

impl SessionTab {
    pub fn new_chat(name: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            tab_type: TabType::Chat,
            content: String::new(),
            modified: false,
        }
    }

    pub fn new_terminal(name: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: format!("⚡ {}", name),
            tab_type: TabType::Terminal,
            content: String::new(),
            modified: false,
        }
    }
}

/// مدیر پیشرفته Session
#[derive(Clone)]
pub struct ProfessionalSessionManager {
    pub sessions: Arc<RwLock<HashMap<SessionId, Arc<RwLock<ProfessionalSession>>>>>,
    pub active_session: Arc<RwLock<Option<SessionId>>>,
    pub base_manager: Arc<BaseSessionManager>,
    pub shared_context: Arc<RwLock<SharedContext>>,
    pub event_sender: mpsc::UnboundedSender<SessionEvent>,
    pub event_receiver: Arc<RwLock<mpsc::UnboundedReceiver<SessionEvent>>>,
}

impl ProfessionalSessionManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            active_session: Arc::new(RwLock::new(None)),
            base_manager: Arc::new(BaseSessionManager::new()),
            shared_context: Arc::new(RwLock::new(SharedContext::new())),
            event_sender: sender.clone(),
            event_receiver: Arc::new(RwLock::new(receiver)),
        }
    }

    /// ایجاد Session جدید
    pub async fn create_session(&self, name: &str, model: &str) -> SessionId {
        let mut session = ProfessionalSession {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            state: SessionState::Active,
            current_task: None,
            model_name: model.to_string(),
            created_at: Local::now(),
            last_active: Local::now(),
            memory: Arc::new(MemoryStore::new()),
            history: SessionHistory::new(1000),
            context: SharedContext::new(),
            settings: SessionSettings::default(),
            tabs: vec![
                SessionTab::new_chat("Chat"),
                SessionTab::new_terminal("Terminal"),
            ],
            active_tab: 0,
        };

        let session_id = session.id.clone();
        let session_arc = Arc::new(RwLock::new(session));
        
        self.sessions.write().insert(session_id.clone(), session_arc.clone());
        self.active_session.write().replace(session_id.clone());
        
        // اطلاع‌رسانی
        let event = SessionEvent::Message {
            from_session: session_id.clone(),
            to_session: None,
            content: format!("Session '{}' created", name),
        };
        self.shared_context.write().broadcast_event(event);
        
        session_id
    }

    /// دریافت Session فعال
    pub async fn get_active_session(&self) -> Option<Arc<RwLock<ProfessionalSession>>> {
        let active_id = self.active_session.read().clone()?;
        self.sessions.read().get(&active_id).cloned()
    }

    /// تعویض Session
    pub async fn switch_session(&self, from_id: SessionId, to_id: SessionId) {
        // Pause session فعلی
        {
            let sessions = self.sessions.read();
            if let Some(session) = sessions.get(&from_id) {
                let mut s = session.write().await;
                s.state = SessionState::Paused;
            }
        }
        
        // Activate session جدید
        {
            let sessions = self.sessions.read();
            if let Some(session) = sessions.get(&to_id) {
                let mut s = session.write().await;
                s.state = SessionState::Active;
                s.last_active = Local::now();
            }
        }
        
        *self.active_session.write() = Some(to_id);
        
        // اطلاع‌رسانی
        let event = SessionEvent::Message {
            from_session: from_id,
            to_session: Some(to_id),
            content: "Session switched".to_string(),
        };
        self.shared_context.write().broadcast_event(event);
    }

    /// ارسال پیام بین Session‌ها
    pub async fn send_message(&self, from_id: &SessionId, to_id: &SessionId, content: &str) {
        let sessions = self.sessions.read();
        if let Some(to_session) = sessions.get(to_id) {
            let mut s = to_session.write().await;
            s.history.add_message("peer", content);
            s.last_active = Local::now();
            
            // اطلاع‌رسانی
            SessionEvent::Message {
 let event = Session                from_session: from_id.to_string(),
                to_session: Some(to_id.to_string()),
                content: content.to_string(),
            };
            self.shared_context.write().broadcast_event(event);
        }
    }

    /// اضافه کردن پیام به session
    pub async fn add_message(&self, session_id: &SessionId, role: &str, content: &str) {
        let sessions = self.sessions.read();
        if let Some(session) = sessions.get(session_id) {
            let mut s = session.write().await;
            s.history.add_message(role, content);
            s.memory.add_short(Message::new(role, content));
            s.last_active = Local::now();
        }
    }

    /// اضافه کردن تب جدید
    pub async fn add_tab(&self, session_id: &SessionId, tab: SessionTab) {
        let sessions = self.sessions.read();
        if let Some(session) = sessions.get(session_id) {
            let mut s = session.write().await;
            s.tabs.push(tab);
        }
    }

    /// تعویض تب
    pub async fn switch_tab(&self, session_id: &SessionId, tab_index: usize) {
        let sessions = self.sessions.read();
        if let Some(session) = sessions.get(session_id) {
            let mut s = session.write().await;
            if tab_index < s.tabs.len() {
                s.active_tab = tab_index;
            }
        }
    }

    /// دریافت تمام session‌ها
    pub async fn get_all_sessions(&self) -> Vec<Arc<RwLock<ProfessionalSession>>> {
        self.sessions.read().values().cloned().collect()
    }

    /// دریافت آمار
    pub async fn get_stats(&self) -> SessionStats {
        let sessions = self.sessions.read();
        let mut stats = SessionStats::default();
        
        for session in sessions.values() {
            let s = session.read().await;
            match s.state {
                SessionState::Active => stats.active += 1,
                SessionState::Background => stats.background += 1,
                SessionState::Paused => stats.paused += 1,
                SessionState::Terminated => stats.terminated += 1,
            }
            
            stats.total_messages += s.history.messages.len();
            stats.total_commands += s.history.commands.len();
            stats.total_tabs += s.tabs.len();
        }
        
        stats.total_sessions = sessions.len();
        stats
    }

    /// حذف session
    pub async fn delete_session(&self, session_id: &SessionId) {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get(session_id) {
            let mut s = session.write().await;
            s.state = SessionState::Terminated;
        }
        sessions.remove(session_id);
        
        // اگر session فعال بود، پاک کن
        if self.active_session.read().as_ref() == Some(session_id) {
            self.active_session.write().take();
        }
    }
}

/// آمار Session‌ها
#[derive(Debug, Default, Clone)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active: usize,
    pub background: usize,
    pub paused: usize,
    pub terminated: usize,
    pub total_messages: usize,
    pub total_commands: usize,
    pub total_tabs: usize,
}

/// رندر کردن Session Panel
pub fn render_session_panel<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &ProfessionalSessionManager,
    area: Rect,
    theme: &crate::config::ThemeName,
) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let stats = rt.block_on(manager.get_stats());
    
    let (bg_color, fg_color, highlight_color) = match theme {
        crate::config::ThemeName::DarkPlus => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan),
        _ => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan),
    };

    let content = format!(
        "💻 SESSIONS - {}\n\n\
        🟢 Active: {} | 🟡 Background: {} | 🔴 Paused: {}\n\
        📝 Total Messages: {}\n\
        ⌨️ Total Commands: {}\n\
        📑 Total Tabs: {}\n\n\
        📋 SESSION LIST\n",
        stats.total_sessions,
        stats.active,
        stats.background,
        stats.paused,
        stats.total_messages,
        stats.total_commands,
        stats.total_tabs
    );

    let panel = Paragraph::new(content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(Block::default().title("💻 Sessions").borders(Borders::NONE));

    frame.render_widget(panel, area);
}

/// رندر کردن Unified Memory Panel
pub fn render_unified_memory_panel<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &ProfessionalSessionManager,
    area: Rect,
    theme: &crate::config::ThemeName,
) {
    let (bg_color, fg_color, _) = match theme {
        crate::config::ThemeName::DarkPlus => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan),
        _ => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan),
    };

    let shared_ctx = manager.shared_context.read();
    
    let content = format!(
        "🧠 UNIFIED MEMORY\n\n\
        💾 GLOBAL MEMORY\n\
        ├── Short-term: messages\n\
        └── Long-term: sessions\n\n\
        🔗 SHARED VARIABLES\n\
        ├── Total: {}\n\
        └── Keys: {}\n\n\
        📡 CROSS-SESSION EVENTS\n\
        ├── Queued: {}\n\
        └── Broadcasting: active\n\n\
        📁 SHARED FILES\n\
        └── Total: {}",
        shared_ctx.shared_variables.len(),
        shared_ctx.shared_variables.keys().cloned().collect::<Vec<_>>().join(", "),
        shared_ctx.event_history.len(),
        shared_ctx.shared_files.len()
    );

    let panel = Paragraph::new(content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(Block::default().title("🧠 Unified Memory").borders(Borders::NONE));

    frame.render_widget(panel, area);
}
