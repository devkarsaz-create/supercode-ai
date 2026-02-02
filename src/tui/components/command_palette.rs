//! Command Palette Component - Professional TUI
//!
//! کامپنت Command Palette با Ctrl+N
//! شامل جستجو، فیلتر، recent commands و categories

use crate::config::ThemeName;
use crate::tui::state::{AppConfig, TaskManager, SessionManager, Task, Session};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// دسته‌بندی دستورات
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Navigation,
    Editor,
    Agent,
    Session,
    System,
    Settings,
    Help,
    Models,
    Tasks,
}

impl CommandCategory {
    pub fn name(&self) -> &str {
        match self {
            CommandCategory::Navigation => "🚗 Navigation",
            CommandCategory::Editor => "✏️ Editor",
            CommandCategory::Agent => "🤖 Agent",
            CommandCategory::Session => "💻 Session",
            CommandCategory::System => "⚙️ System",
            CommandCategory::Settings => "🔧 Settings",
            CommandCategory::Help => "❓ Help",
            CommandCategory::Models => "🤖 Models",
            CommandCategory::Tasks => "📋 Tasks",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            CommandCategory::Navigation => "🚗",
            CommandCategory::Editor => "✏️",
            CommandCategory::Agent => "🤖",
            CommandCategory::Session => "💻",
            CommandCategory::System => "⚙️",
            CommandCategory::Settings => "🔧",
            CommandCategory::Help => "❓",
            CommandCategory::Models => "🤖",
            CommandCategory::Tasks => "📋",
        }
    }
}

/// یک دستور (Command)
#[derive(Debug, Clone)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub category: CommandCategory,
    pub shortcut: Option<String>,
    pub description: String,
    pub keywords: Vec<String>,
    pub action: CommandAction,
}

#[derive(Debug, Clone)]
pub enum CommandAction {
    OpenCommandPalette,
    QuickOpen,
    GoToLine,
    NewTask,
    ListTasks,
    NewSession,
    SwitchSession,
    CloseSession,
    SaveFile,
    SaveAll,
    ToggleSidebar,
    ToggleTerminal,
    OpenSettings,
    OpenHelp,
    ImportModel,
    ListModels,
    SwitchModel,
    ViewHistory,
    ViewMemory,
    Custom(String), // برای دستورات سفارشی
}

/// وضعیت Command Palette
#[derive(Clone)]
pub struct CommandPaletteState {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    pub filtered_commands: Vec<Command>,
    pub recent_commands: Vec<String>, // Command IDs
    pub categories: Vec<CommandCategory>,
    pub selected_category: Option<CommandCategory>,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            selected_index: 0,
            filtered_commands: Vec::new(),
            recent_commands: Vec::new(),
            categories: vec![
                CommandCategory::Navigation,
                CommandCategory::Editor,
                CommandCategory::Agent,
                CommandCategory::Session,
                CommandCategory::Tasks,
                CommandCategory::Models,
                CommandCategory::Settings,
                CommandCategory::Help,
            ],
            selected_category: None,
        }
    }
}

impl CommandPaletteState {
    pub fn new() -> Self {
        Self::default()
    }

    /// فیلتر کردن دستورات بر اساس query
    pub fn filter_commands(&mut self, commands: &[Command]) {
        let query = self.query.to_lowercase();
        
        if query.is_empty() {
            // اگر query خالی است، همه دستورات را نشان بده
            self.filtered_commands = commands.to_vec();
        } else {
            // فیلتر بر اساس title، description و keywords
            self.filtered_commands = commands
                .iter()
                .filter(|cmd| {
                    cmd.title.to_lowercase().contains(&query)
                        || cmd.description.to_lowercase().contains(&query)
                        || cmd.keywords.iter().any(|k| k.to_lowercase().contains(&query))
                })
                .cloned()
                .collect();
        }

        // مرتب‌سازی: دستورات با shortcut اول
        self.filtered_commands.sort_by(|a, b| {
            match (a.shortcut.is_some(), b.shortcut.is_some()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.title.cmp(&b.title),
            }
        });

        // محدود کردن به 20 مورد اول
        self.filtered_commands.truncate(20);

        // ریست کردن selected index
        self.selected_index = 0;
    }

    /// اضافه کردن recent command
    pub fn add_recent(&mut self, command_id: &str) {
        self.recent_commands.retain(|id| id != command_id);
        self.recent_commands.insert(0, command_id.to_string());
        // نگه‌داشتن فقط 10 recent
        self.recent_commands.truncate(10);
    }

    /// انتخاب command بعدی
    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.filtered_commands.len() {
            self.selected_index += 1;
        }
    }

    /// انتخاب command قبلی
    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// دریافت command انتخاب‌شده
    pub fn get_selected_command(&self) -> Option<&Command> {
        self.filtered_commands.get(self.selected_index)
    }
}

/// مدیر دستورات
#[derive(Clone)]
pub struct CommandManager {
    commands: Arc<RwLock<HashMap<String, Command>>>,
    pub palette_state: Arc<RwLock<CommandPaletteState>>,
}

impl CommandManager {
    pub fn new() -> Self {
        let mut manager = Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            palette_state: Arc::new(RwLock::new(CommandPaletteState::new())),
        };
        manager.register_default_commands();
        manager
    }

    /// ثبت دستورات پیش‌فرض
    fn register_default_commands(&mut self) {
        let default_commands = vec![
            // Navigation
            Command {
                id: "command_palette".to_string(),
                title: "Command Palette".to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("Ctrl+N".to_string()),
                description: "Open command palette for quick access".to_string(),
                keywords: vec!["search", "run", "execute", "palette".to_string()],
                action: CommandAction::OpenCommandPalette,
            },
            Command {
                id: "quick_open".to_string(),
                title: "Quick Open".to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("Ctrl+O".to_string()),
                description: "Quickly open files or commands".to_string(),
                keywords: vec!["open", "file", "search", "goto".to_string()],
                action: CommandAction::QuickOpen,
            },
            Command {
                id: "goto_line".to_string(),
                title: "Go to Line".to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("Ctrl+G".to_string()),
                description: "Jump to specific line number".to_string(),
                keywords: vec!["line", "jump", "goto", "move".to_string()],
                action: CommandAction::GoToLine,
            },
            
            // Agent
            Command {
                id: "new_task".to_string(),
                title: "New Task".to_string(),
                category: CommandCategory::Agent,
                shortcut: Some("Ctrl+T".to_string()),
                description: "Create a new task for the agent".to_string(),
                keywords: vec!["task", "new", "create", "agent".to_string()],
                action: CommandAction::NewTask,
            },
            Command {
                id: "list_tasks".to_string(),
                title: "List Tasks".to_string(),
                category: CommandCategory::Agent,
                shortcut: Some("Ctrl+Alt+T".to_string()),
                description: "View all active and pending tasks".to_string(),
                keywords: vec!["task", "list", "view", "show".to_string()],
                action: CommandAction::ListTasks,
            },
            
            // Session
            Command {
                id: "new_session".to_string(),
                title: "New Session".to_string(),
                category: CommandCategory::Session,
                shortcut: Some("Ctrl+Shift+N".to_string()),
                description: "Create a new session".to_string(),
                keywords: vec!["session", "new", "create", "window".to_string()],
                action: CommandAction::NewSession,
            },
            Command {
                id: "switch_session".to_string(),
                title: "Switch Session".to_string(),
                category: CommandCategory::Session,
                shortcut: Some("Ctrl+Tab".to_string()),
                description: "Switch between active sessions".to_string(),
                keywords: vec!["session", "switch", "change", "alternate".to_string()],
                action: CommandAction::SwitchSession,
            },
            Command {
                id: "close_session".to_string(),
                title: "Close Session".to_string(),
                category: CommandCategory::Session,
                shortcut: Some("Ctrl+Shift+W".to_string()),
                description: "Close current session".to_string(),
                keywords: vec!["session", "close", "end", "quit".to_string()],
                action: CommandAction::CloseSession,
            },
            
            // System
            Command {
                id: "toggle_sidebar".to_string(),
                title: "Toggle Sidebar".to_string(),
                category: CommandCategory::System,
                shortcut: Some("Ctrl+B".to_string()),
                description: "Show or hide the sidebar".to_string(),
                keywords: vec!["sidebar", "toggle", "show", "hide".to_string()],
                action: CommandAction::ToggleSidebar,
            },
            Command {
                id: "toggle_terminal".to_string(),
                title: "Toggle Terminal".to_string(),
                category: CommandCategory::System,
                shortcut: Some("`".to_string()),
                description: "Show or hide the terminal panel".to_string(),
                keywords: vec!["terminal", "toggle", "console", "output".to_string()],
                action: CommandAction::ToggleTerminal,
            },
            Command {
                id: "save_file".to_string(),
                title: "Save File".to_string(),
                category: CommandCategory::System,
                shortcut: Some("Ctrl+S".to_string()),
                description: "Save current file".to_string(),
                keywords: vec!["save", "file", "write", "store".to_string()],
                action: CommandAction::SaveFile,
            },
            
            // Settings
            Command {
                id: "settings".to_string(),
                title: "Open Settings".to_string(),
                category: CommandCategory::Settings,
                shortcut: Some("Ctrl+,".to_string()),
                description: "Open application settings".to_string(),
                keywords: vec!["settings", "config", "preferences", "options".to_string()],
                action: CommandAction::OpenSettings,
            },
            
            // Help
            Command {
                id: "help".to_string(),
                title: "Help".to_string(),
                category: CommandCategory::Help,
                shortcut: Some("F1".to_string()),
                description: "Open help documentation".to_string(),
                keywords: vec!["help", "docs", "documentation", "guide".to_string()],
                action: CommandAction::OpenHelp,
            },
            
            // Models
            Command {
                id: "import_model".to_string(),
                title: "Import Model".to_string(),
                category: CommandCategory::Models,
                description: "Import a new model from path".to_string(),
                keywords: vec!["model", "import", "add", "new".to_string()],
                action: CommandAction::ImportModel,
            },
            Command {
                id: "list_models".to_string(),
                title: "List Models".to_string(),
                category: CommandCategory::Models,
                description: "List all available models".to_string(),
                keywords: vec!["model", "list", "show", "view".to_string()],
                action: CommandAction::ListModels,
            },
            
            // Tasks
            Command {
                id: "view_memory".to_string(),
                title: "View Memory".to_string(),
                category: CommandCategory::Tasks,
                description: "View agent memory and context".to_string(),
                keywords: vec!["memory", "context", "history", "view".to_string()],
                action: CommandAction::ViewMemory,
            },
            Command {
                id: "view_history".to_string(),
                title: "View History".to_string(),
                category: CommandCategory::Tasks,
                description: "View command and task history".to_string(),
                keywords: vec!["history", "past", "previous", "log".to_string()],
                action: CommandAction::ViewHistory,
            },
        ];

        for cmd in default_commands {
            self.commands.write().insert(cmd.id.clone(), cmd);
        }
    }

    /// دریافت تمام دستورات
    pub fn get_all_commands(&self) -> Vec<Command> {
        self.commands.read().values().cloned().collect()
    }

    /// دریافت دستور با ID
    pub fn get_command(&self, id: &str) -> Option<Command> {
        self.commands.read().get(id).cloned()
    }

    /// اضافه کردن دستور سفارشی
    pub fn register_command(&self, command: Command) {
        self.commands.write().insert(command.id.clone(), command);
    }

    /// باز کردن palette
    pub fn open(&self) {
        let mut state = self.palette_state.write();
        state.is_open = true;
        state.query.clear();
        state.selected_index = 0;
        
        // فیلتر کردن دستورات
        let all_commands = self.get_all_commands();
        state.filter_commands(&all_commands);
    }

    /// بستن palette
    pub fn close(&self) {
        let mut state = self.palette_state.write();
        state.is_open = false;
        state.query.clear();
    }

    /// toggle کردن
    pub fn toggle(&self) {
        let mut state = self.palette_state.write();
        if state.is_open {
            state.is_open = false;
        } else {
            state.is_open = true;
            state.query.clear();
            state.selected_index = 0;
            let all_commands = self.get_all_commands();
            state.filter_commands(&all_commands);
        }
    }

    /// مدیریت ورودی کیبورد
    pub fn handle_key(&self, key: KeyEvent) -> Option<CommandAction> {
        let mut state = self.palette_state.write();

        if !state.is_open {
            return None;
        }

        match key.code {
            KeyCode::Esc => {
                state.is_open = false;
                state.query.clear();
                return None;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                state.select_prev();
                return None;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.select_next();
                return None;
            }
            KeyCode::Enter => {
                if let Some(cmd) = state.get_selected_command() {
                    state.add_recent(&cmd.id);
                    state.is_open = false;
                    state.query.clear();
                    return Some(cmd.action.clone());
                }
                return None;
            }
            KeyCode::Backspace => {
                state.query.pop();
                let all_commands = self.get_all_commands();
                state.filter_commands(&all_commands);
                return None;
            }
            KeyCode::Char(c) if key.modifiers == KeyModifiers::CONTROL => {
                // Ctrl combinations are handled elsewhere
                return None;
            }
            KeyCode::Char(c) => {
                state.query.push(c);
                let all_commands = self.get_all_commands();
                state.filter_commands(&all_commands);
                return None;
            }
            _ => return None,
        }
    }
}

/// رندر کردن Command Palette
pub fn render_command_palette<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    state: &CommandPaletteState,
    area: Rect,
    theme: &ThemeName,
) {
    if !state.is_open {
        return;
    }

    // استایل‌ها بر اساس تم
    let (bg_color, fg_color, highlight_color) = match theme {
        ThemeName::DarkPlus => (Color::Black, Color::White, Color::Cyan),
        ThemeName::Light => (Color::White, Color::Black, Color::Blue),
        ThemeName::Monokai => (Color::Black, Color::Green, Color::Yellow),
        ThemeName::SolarizedDark => (Color::Rgb(0, 43, 54), Color::Rgb(131, 148, 150), Color::Cyan),
        ThemeName::Dracula => (Color::Rgb(40, 42, 54), Color::Rgb(248, 248, 242), Color::Cyan),
        _ => (Color::Black, Color::White, Color::Cyan),
    };

    let selected_style = Style::default()
        .bg(highlight_color)
        .fg(Color::Black);

    // عنوان بالای palette
    let title = if state.query.is_empty() {
        "🔍 Type to search commands..."
    } else {
        "🔍 Searching..."
    };

    // ساخت لیست دستورات
    let items: Vec<ListItem> = state
        .filtered_commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let shortcut = cmd.shortcut.as_ref().map(|s| s.as_str()).unwrap_or("");
            let prefix = if i == state.selected_index { "👉 " } else { "  " };
            
            // ساخت متن خط
            let line = if !shortcut.is_empty() {
                format!("{} {}  [{}]", prefix, cmd.title, shortcut)
            } else {
                format!("{} {}", prefix, cmd.title)
            };
            
            ListItem::new(line)
        })
        .collect();

    // پیمایش در لیست
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(state.selected_index));

    // رندر کردن
    let block = Block::default()
        .title(Line::from(title).alignment(Alignment::Center))
        .borders(Borders::ALL)
        .style(Style::default().bg(bg_color).fg(fg_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if !items.is_empty() {
        let list = List::new(items)
            .style(Style::default().bg(bg_color).fg(fg_color))
            .highlight_style(selected_style);
        
        frame.render_stateful_widget(list, inner_area, &mut list_state);
    } else {
        let no_results = Paragraph::new("No commands found")
            .style(Style::default().bg(bg_color).fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(no_results, inner_area);
    }

    // نمایش تعداد نتایج
    let count_text = format!("{}/{} commands", state.filtered_commands.len(), state.filtered_commands.len());
    let footer = Paragraph::new(count_text)
        .style(Style::default().bg(bg_color).fg(Color::Gray))
        .alignment(Alignment::Right);
    frame.render_widget(footer, Rect {
        x: inner_area.x,
        y: inner_area.y + inner_area.height.saturating_sub(1),
        width: inner_area.width,
        height: 1,
    });
}

/// بررسی اینکه آیا کلید برای command palette است
pub fn is_palette_shortcut(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('n') && key.modifiers == KeyModifiers::CONTROL
}
