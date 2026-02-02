//! Advanced Command Palette with Slash Commands
//!
//! کامند پالیت پیشرفته با:
//! - Ctrl+P برای باز شدن
//! - / برای slash commands
//! - جستجوی هوشمند با fuzzy matching
//! - Tab برای اجرای مستقیم

use crate::tui::keyboard::{KeyAction, KeyboardManager};
use crate::config::ThemeName;
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
use std::cmp::Ordering;

/// نوع دستور slash
#[derive(Debug, Clone, PartialEq)]
pub enum SlashCommand {
    // Agent Commands
    Task(String),           // /task <name>
    Agent(String),          // /agent <name>
    Model(String),          // /model <name>
    Context,                // /context
    Memory,                 // /memory
    History,                // /history
    
    // Editor Commands
    Edit,                   // /edit
    Find,                   // /find
    Replace,                // /replace
    Format,                 // /format
    
    // System Commands
    Clear,                  // /clear
    Settings,               // /settings
    Help,                   // /help
    Shortcuts,              // /shortcuts
    
    // Navigation Commands
    Goto(String),           // /goto <file:line>
    Open(String),           // /open <file>
    Close,                  // /close
    
    // Custom
    Custom(String),         // /<custom>
}

/// یک slash command
#[derive(Debug, Clone)]
pub struct SlashCommandDefinition {
    pub command: String,
    pub alias: Vec<String>,
    pub description: String,
    pub category: SlashCategory,
    pub action: SlashCommand,
    pub arguments: Vec<ArgumentDef>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SlashCategory {
    Agent,
    Editor,
    System,
    Navigation,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ArgumentDef {
    pub name: String,
    pub required: bool,
    pub description: String,
    pub default: Option<String>,
}

/// وضعیت Command Palette پیشرفته
#[derive(Clone)]
pub struct AdvancedCommandPaletteState {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    pub filtered_commands: Vec<CommandMatch>,
    pub mode: PaletteMode,
    pub argument_input: Option<String>,
    pub current_argument: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaletteMode {
    Command,      // جستجوی دستور
    Argument,     // وارد کردن آرگومان
    Results,      // نمایش نتایج
}

/// نتیجه تطبیق دستور
#[derive(Debug, Clone)]
pub struct CommandMatch {
    pub command: SlashCommandDefinition,
    pub score: f64,              // امتیاز تطبیق
    pub match_type: MatchType,   // نوع تطبیق
    pub highlight: Vec<(usize, usize)>, // بخش‌های匹配 شده
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,           // تطبیق کامل
    Prefix,          // پیشوند
    Fuzzy,           // fuzzy matching
    Contains,        // شامل می‌شود
}

/// مدیر slash commands
#[derive(Clone)]
pub struct SlashCommandManager {
    pub commands: Arc<RwLock<HashMap<String, SlashCommandDefinition>>>,
    pub state: Arc<RwLock<AdvancedCommandPaletteState>>,
    pub keyboard: Arc<KeyboardManager>,
}

impl SlashCommandManager {
    pub fn new(keyboard: Arc<KeyboardManager>) -> Self {
        let mut manager = Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(AdvancedCommandPaletteState::new())),
            keyboard,
        };
        manager.register_default_commands();
        manager
    }

    /// ثبت دستورات پیش‌فرض
    fn register_default_commands(&mut self) {
        let default_commands = vec![
            // ========== Agent Commands ==========
            SlashCommandDefinition {
                command: "task".to_string(),
                alias: vec!["new", "create", "make".to_string()],
                description: "Create a new task for the agent",
                category: SlashCategory::Agent,
                action: SlashCommand::Task(String::new()),
                arguments: vec![
                    ArgumentDef {
                        name: "name".to_string(),
                        required: true,
                        description: "Task name or description".to_string(),
                        default: None,
                    },
                ],
                examples: vec![
                    "/task Implement login feature".to_string(),
                    "/task Fix bug in TUI".to_string(),
                ],
            },
            SlashCommandDefinition {
                command: "agent".to_string(),
                alias: vec!["use", "switch".to_string()],
                description: "Switch to a different agent",
                category: SlashCategory::Agent,
                action: SlashCommand::Agent(String::new()),
                arguments: vec![
                    ArgumentDef {
                        name: "agent".to_string(),
                        required: true,
                        description: "Agent name (planner, executor, critic)".to_string(),
                        default: Some("planner".to_string()),
                    },
                ],
                examples: vec![
                    "/agent planner".to_string(),
                    "/agent executor".to_string(),
                ],
            },
            SlashCommandDefinition {
                command: "model".to_string(),
                alias: vec!["llm", "use".to_string()],
                description: "Switch to a different LLM model",
                category: SlashCategory::Agent,
                action: SlashCommand::Model(String::new()),
                arguments: vec![
                    ArgumentDef {
                        name: "model".to_string(),
                        required: true,
                        description: "Model name (llama3.2, codellama, etc.)".to_string(),
                        default: None,
                    },
                ],
                examples: vec![
                    "/model llama3.2".to_string(),
                    "/model codellama".to_string(),
                ],
            },
            SlashCommandDefinition {
                command: "context".to_string(),
                alias: vec!["ctx".to_string()],
                description: "View current conversation context",
                category: SlashCategory::Agent,
                action: SlashCommand::Context,
                arguments: vec![],
                examples: vec!["/context".to_string()],
            },
            SlashCommandDefinition {
                command: "memory".to_string(),
                alias: vec!["mem".to_string()],
                description: "View and manage agent memory",
                category: SlashCategory::Agent,
                action: SlashCommand::Memory,
                arguments: vec![],
                examples: vec!["/memory".to_string(), "/memory clear".to_string()],
            },
            SlashCommandDefinition {
                command: "history".to_string(),
                alias: vec!["hist".to_string()],
                description: "View command and task history",
                category: SlashCategory::Agent,
                action: SlashCommand::History,
                arguments: vec![],
                examples: vec!["/history".to_string()],
            },
            
            // ========== Editor Commands ==========
            SlashCommandDefinition {
                command: "edit".to_string(),
                alias: vec!["modify".to_string()],
                description: "Edit a file or code block",
                category: SlashCategory::Editor,
                action: SlashCommand::Edit,
                arguments: vec![
                    ArgumentDef {
                        name: "file".to_string(),
                        required: true,
                        description: "File path to edit".to_string(),
                        default: None,
                    },
                ],
                examples: vec!["/edit src/main.rs".to_string()],
            },
            SlashCommandDefinition {
                command: "find".to_string(),
                alias: vec!["search", "grep".to_string()],
                description: "Find text in files",
                category: SlashCategory::Editor,
                action: SlashCommand::Find,
                arguments: vec![
                    ArgumentDef {
                        name: "query".to_string(),
                        required: true,
                        description: "Text to find".to_string(),
                        default: None,
                    },
                ],
                examples: vec!["/find fn main".to_string()],
            },
            SlashCommandDefinition {
                command: "format".to_string(),
                alias: vec!["fmt".to_string()],
                description: "Format code in file",
                category: SlashCategory::Editor,
                action: SlashCommand::Format,
                arguments: vec![
                    ArgumentDef {
                        name: "file".to_string(),
                        required: false,
                        description: "File path (current if not specified)".to_string(),
                        default: Some("current".to_string()),
                    },
                ],
                examples: vec!["/format".to_string(), "/format src/main.rs".to_string()],
            },
            
            // ========== System Commands ==========
            SlashCommandDefinition {
                command: "clear".to_string(),
                alias: vec!["cls".to_string()],
                description: "Clear terminal output",
                category: SlashCategory::System,
                action: SlashCommand::Clear,
                arguments: vec![],
                examples: vec!["/clear".to_string()],
            },
            SlashCommandDefinition {
                command: "settings".to_string(),
                alias: vec!["config", "prefs".to_string()],
                description: "Open settings panel",
                category: SlashCategory::System,
                action: SlashCommand::Settings,
                arguments: vec![],
                examples: vec!["/settings".to_string()],
            },
            SlashCommandDefinition {
                command: "help".to_string(),
                alias: vec!["?".to_string()],
                description: "Show help and documentation",
                category: SlashCategory::System,
                action: SlashCommand::Help,
                arguments: vec![],
                examples: vec!["/help".to_string(), "/help commands".to_string()],
            },
            SlashCommandDefinition {
                command: "shortcuts".to_string(),
                alias: vec!["keys", "bindings".to_string()],
                description: "Show keyboard shortcuts",
                category: SlashCategory::System,
                action: SlashCommand::Shortcuts,
                arguments: vec![],
                examples: vec!["/shortcuts".to_string()],
            },
            
            // ========== Navigation Commands ==========
            SlashCommandDefinition {
                command: "goto".to_string(),
                alias: vec!["line", "jump".to_string()],
                description: "Go to specific line in file",
                category: SlashCategory::Navigation,
                action: SlashCommand::Goto(String::new()),
                arguments: vec![
                    ArgumentDef {
                        name: "location".to_string(),
                        required: true,
                        description: "File:line (e.g., src/main.rs:10)".to_string(),
                        default: None,
                    },
                ],
                examples: vec!["/goto src/main.rs:10".to_string(), "/goto 50".to_string()],
            },
            SlashCommandDefinition {
                command: "open".to_string(),
                alias: vec!["file", "load".to_string()],
                description: "Open a file",
                category: SlashCategory::Navigation,
                action: SlashCommand::Open(String::new()),
                arguments: vec![
                    ArgumentDef {
                        name: "file".to_string(),
                        required: true,
                        description: "File path to open".to_string(),
                        default: None,
                    },
                ],
                examples: vec!["/open Cargo.toml".to_string()],
            },
            SlashCommandDefinition {
                command: "close".to_string(),
                alias: vec!["quit".to_string()],
                description: "Close current tab or panel",
                category: SlashCategory::Navigation,
                action: SlashCommand::Close,
                arguments: vec![],
                examples: vec!["/close".to_string()],
            },
        ];

        for cmd in default_commands {
            self.commands.write().insert(cmd.command.clone(), cmd);
        }
    }

    /// باز کردن palette
    pub fn open(&self) {
        let mut state = self.state.write();
        state.is_open = true;
        state.query.clear();
        state.selected_index = 0;
        state.mode = PaletteMode::Command;
        state.argument_input = None;
    }

    /// بستن palette
    pub fn close(&self) {
        let mut state = self.state.write();
        state.is_open = false;
        state.query.clear();
        state.mode = PaletteMode::Command;
    }

    /// toggle کردن
    pub fn toggle(&self) {
        let mut state = self.state.write();
        if state.is_open {
            state.is_open = false;
        } else {
            state.is_open = true;
            state.query.clear();
            state.selected_index = 0;
            state.mode = PaletteMode::Command;
        }
    }

    /// مدیریت ورودی کیبورد
    pub fn handle_key(&self, key: KeyEvent) -> Option<SlashCommand> {
        let mut state = self.state.write();

        if !state.is_open {
            return None;
        }

        match state.mode {
            PaletteMode::Command => self.handle_command_mode(key, &mut state),
            PaletteMode::Argument => self.handle_argument_mode(key, &mut state),
            PaletteMode::Results => self.handle_results_mode(key, &mut state),
        }
    }

    fn handle_command_mode(&self, key: KeyEvent, state: &mut AdvancedCommandPaletteState) -> Option<SlashCommand> {
        match key.code {
            KeyCode::Esc => {
                state.is_open = false;
                state.query.clear();
                return None;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if state.selected_index > 0 {
                    state.selected_index -= 1;
                }
                return None;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if state.selected_index + 1 < state.filtered_commands.len() {
                    state.selected_index += 1;
                }
                return None;
            }
            KeyCode::Enter => {
                if let Some(cmd_match) = state.filtered_commands.get(state.selected_index) {
                    let cmd = &cmd_match.command;
                    
                    // اگر دستور آرگومان دارد، به حالت argument برو
                    if !cmd.arguments.is_empty() {
                        state.mode = PaletteMode::Argument;
                        state.argument_input = Some(String::new());
                        state.current_argument = 0;
                        return None;
                    }
                    
                    // اگر دستور آرگومان ندارد، اجرا کن
                    state.is_open = false;
                    state.query.clear();
                    return Some(cmd.action.clone());
                }
                return None;
            }
            KeyCode::Tab => {
                // اجرای مستقیم دستور بدون Enter
                if let Some(cmd_match) = state.filtered_commands.get(state.selected_index) {
                    let cmd = &cmd_match.command;
                    state.is_open = false;
                    state.query.clear();
                    return Some(cmd.action.clone());
                }
                return None;
            }
            KeyCode::Backspace => {
                state.query.pop();
                self.filter_commands(state);
                return None;
            }
            KeyCode::Char(c) => {
                state.query.push(c);
                self.filter_commands(state);
                return None;
            }
            _ => return None,
        }
    }

    fn handle_argument_mode(&self, key: KeyEvent, state: &mut AdvancedCommandPaletteState) -> Option<SlashCommand> {
        match key.code {
            KeyCode::Esc => {
                state.mode = PaletteMode::Command;
                state.argument_input = None;
                return None;
            }
            KeyCode::Enter => {
                // ذخیره آرگومان و برگشت به command mode
                state.mode = PaletteMode::Command;
                state.argument_input = None;
                return None;
            }
            KeyCode::Backspace => {
                if let Some(input) = &mut state.argument_input {
                    input.pop();
                }
                return None;
            }
            KeyCode::Char(c) => {
                if let Some(input) = &mut state.argument_input {
                    input.push(c);
                }
                return None;
            }
            _ => return None,
        }
    }

    fn handle_results_mode(&self, key: KeyEvent, state: &mut AdvancedCommandPaletteState) -> Option<SlashCommand> {
        // مشابه command mode
        self.handle_command_mode(key, state)
    }

    /// فیلتر کردن دستورات با fuzzy matching
    fn filter_commands(&self, state: &mut AdvancedCommandPaletteState) {
        let query = state.query.to_lowercase();
        
        if query.is_empty() {
            // اگر query خالی است، همه دستورات را نشان بده
            let all_cmds: Vec<SlashCommandDefinition> = 
                self.commands.read().values().cloned().collect();
            
            state.filtered_commands = all_cmds.into_iter()
                .map(|cmd| CommandMatch {
                    command: cmd,
                    score: 1.0,
                    match_type: MatchType::Contains,
                    highlight: vec![],
                })
                .collect();
        } else {
            // جستجوی fuzzy
            let all_cmds: Vec<SlashCommandDefinition> = 
                self.commands.read().values().cloned().collect();
            
            state.filtered_commands = all_cmds.into_iter()
                .filter_map(|cmd| {
                    let mut best_match: Option<(f64, MatchType, Vec<(usize, usize)>)> = None;
                    
                    // تطبیق با command اصلی
                    if let Some((score, match_type, highlights)) = 
                        self.fuzzy_match(&query, &cmd.command) {
                        best_match = Some((score, match_type, highlights));
                    }
                    
                    // تطبیق با alias‌ها
                    for alias in &cmd.alias {
                        if let Some((score, match_type, highlights)) = 
                            self.fuzzy_match(&query, alias) {
                            if score > best_match.as_ref().map(|(s, _, _)| *s).unwrap_or(0.0) {
                                best_match = Some((score, match_type, highlights));
                            }
                        }
                    }
                    
                    // تطبیق در description
                    if let Some((score, match_type, _)) = 
                        self.fuzzy_match(&query, &cmd.description) {
                        if score > best_match.as_ref().map(|(s, _, _)| *s).unwrap_or(0.0) {
                            best_match = Some((score, match_type, vec![]));
                        }
                    }
                    
                    best_match.map(|(score, match_type, highlights)| CommandMatch {
                        command: cmd,
                        score,
                        match_type,
                        highlight: highlights,
                    })
                })
                .filter(|cmd| cmd.score > 0.3)  // حداقل امتیاز
                .sorted_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal))
                .take(15)  // حداکثر 15 نتیجه
                .collect();
        }
        
        state.selected_index = 0;
    }

    /// الگوریتم fuzzy matching
    fn fuzzy_match(&self, query: &str, text: &str) -> Option<(f64, MatchType, Vec<(usize, usize)>)> {
        let text_lower = text.to_lowercase();
        let query_chars: Vec<char> = query.chars().collect();
        let text_chars: Vec<char> = text_lower.chars().collect();
        
        if query_chars.is_empty() {
            return Some((1.0, MatchType::Contains, vec![]));
        }
        
        // تطبیق کامل
        if text_lower == query {
            return Some((1.0, MatchType::Exact, vec![(0, text_chars.len())]));
        }
        
        // تطبیق پیشوند
        if text_chars.starts_with(&query_chars) {
            return Some((0.9, MatchType::Prefix, vec![(0, query_chars.len())]));
        }
        
        // جستجوی characters به ترتیب
        let mut last_pos = 0;
        let mut highlights = Vec::new();
        let mut found = true;
        
        for (i, qc) in query_chars.iter().enumerate() {
            let mut found_pos = None;
            for (pos, tc) in text_chars[last_pos..].iter().enumerate() {
                if qc == tc {
                    found_pos = Some(last_pos + pos);
                    break;
                }
            }
            
            if let Some(pos) = found_pos {
                highlights.push((pos, pos + 1));
                last_pos = pos + 1;
            } else {
                found = false;
                break;
            }
        }
        
        if found {
            let score = query.len() as f64 / text_chars.len() as f64;
            return Some((score * 0.7, MatchType::Fuzzy, highlights));
        }
        
        // جستجویContains
        if text_lower.contains(&query) {
            if let Some(pos) = text_lower.find(&query) {
                return Some((0.5, MatchType::Contains, vec![(pos, pos + query.len())]));
            }
        }
        
        None
    }

    /// دریافت دستورات مشابه
    pub fn get_similar_commands(&self, partial: &str) -> Vec<SlashCommandDefinition> {
        let all_cmds: Vec<SlashCommandDefinition> = 
            self.commands.read().values().cloned().collect();
        
        all_cmds.into_iter()
            .filter(|cmd| {
                let cmd_lower = cmd.command.to_lowercase();
                let partial_lower = partial.to_lowercase();
                cmd_lower.starts_with(&partial_lower) || 
                cmd.alias.iter().any(|a| a.to_lowercase().starts_with(&partial_lower))
            })
            .collect()
    }
}

impl Default for AdvancedCommandPaletteState {
    fn default() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            selected_index: 0,
            filtered_commands: vec![],
            mode: PaletteMode::Command,
            argument_input: None,
            current_argument: 0,
        }
    }
}

/// رندر کردن Command Palette پیشرفته
pub fn render_advanced_command_palette<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &SlashCommandManager,
    area: Rect,
    theme: &ThemeName,
) {
    let state = manager.state.read();
    
    if !state.is_open {
        return;
    }

    // استایل‌ها بر اساس تم
    let (bg_color, fg_color, highlight_color, accent_color) = match theme {
        ThemeName::DarkPlus => (Color::Rgb(20, 20, 20), Color::White, Color::Cyan, Color::Yellow),
        ThemeName::Light => (Color::White, Color::Black, Color::Blue, Color::Magenta),
        ThemeName::Monokai => (Color::Rgb(39, 40, 34), Color::White, Color::Yellow, Color::Green),
        ThemeName::Dracula => (Color::Rgb(40, 42, 54), Color::White, Color::Cyan, Color::Pink),
        _ => (Color::Rgb(20, 20, 20), Color::White, Color::Cyan, Color::Yellow),
    };

    // محاسبه اندازه palette (وسط صفحه، 60% عرض، 50% ارتفاع)
    let palette_width = (area.width as f64 * 0.6) as u16;
    let palette_height = (area.height as f64 * 0.5) as u16;
    let palette_x = area.x + (area.width - palette_width) / 2;
    let palette_y = area.y + (area.height - palette_height) / 2;

    let palette_area = Rect {
        x: palette_x,
        y: palette_y,
        width: palette_width,
        height: palette_height,
    };

    // عنوان بالا
    let title = match state.mode {
        PaletteMode::Command => "🔍 Command Palette (type / for commands)",
        PaletteMode::Argument => "📝 Enter Arguments",
        PaletteMode::Results => "📋 Results",
    };

    let block = Block::default()
        .title(Line::from(title).alignment(Alignment::Center))
        .borders(Borders::ALL)
        .style(Style::default().bg(bg_color).fg(fg_color));

    frame.render_widget(block, palette_area);

    let inner_area = Rect {
        x: palette_area.x + 1,
        y: palette_area.y + 1,
        width: palette_area.width - 2,
        height: palette_area.height - 2,
    };

    // نمایش input یا لیست نتایج
    match state.mode {
        PaletteMode::Command | PaletteMode::Results => {
            // نمایش input
            let input_style = Style::default()
                .bg(Color::Rgb(40, 40, 40))
                .fg(Color::White);
            let input_text = if state.query.is_empty() {
                "/".to_string()
            } else {
                state.query.clone()
            };
            let input = Paragraph::new(input_text)
                .style(input_style)
                .block(Block::default().borders(Borders::BOTTOM));
            
            frame.render_widget(input, Rect {
                x: inner_area.x,
                y: inner_area.y,
                width: inner_area.width,
                height: 3,
            });

            // لیست دستورات
            if state.filtered_commands.is_empty() {
                let no_results = Paragraph::new("No commands found")
                    .style(Style::default().bg(bg_color).fg(Color::Gray))
                    .alignment(Alignment::Center);
                frame.render_widget(no_results, Rect {
                    x: inner_area.x,
                    y: inner_area.y + 4,
                    width: inner_area.width,
                    height: inner_area.height - 4,
                });
            } else {
                let items: Vec<ListItem> = state.filtered_commands
                    .iter()
                    .enumerate()
                    .map(|(i, cmd_match)| {
                        let cmd = &cmd_match.command;
                        let is_selected = i == state.selected_index;
                        
                        // ساخت متن با highlighting
                        let prefix = if is_selected { "👉" } else { "  " };
                        let category_icon = match cmd.category {
                            SlashCategory::Agent => "🤖",
                            SlashCategory::Editor => "✏️",
                            SlashCategory::System => "⚙️",
                            SlashCategory::Navigation => "🧭",
                            SlashCategory::Custom => "📦",
                        };
                        
                        // ساخت توضیحات با arguments
                        let args_str = if cmd.arguments.is_empty() {
                            String::new()
                        } else {
                            let args: Vec<String> = cmd.arguments.iter()
                                .map(|a| {
                                    if a.required {
                                        format!("<{}>", a.name)
                                    } else {
                                        format!("[{}]", a.name)
                                    }
                                })
                                .collect();
                            format!(" {}", args.join(" "))
                        };
                        
                        let line = format!(
                            "{} {} /{}{}{}",
                            prefix,
                            category_icon,
                            cmd.command,
                            args_str,
                            if !cmd.alias.is_empty() { 
                                format!(" (aliases: {})", cmd.alias.join(", ")) 
                            } else { 
                                String::new() }
                        );
                        
                        let style = if is_selected {
                            Style::default()
                                .bg(highlight_color)
                                .fg(Color::Black)
                        } else {
                            Style::default()
                                .bg(bg_color)
                                .fg(fg_color)
                        };
                        
                        ListItem::new(line).style(style)
                    })
                    .collect();

                let mut list_state = ratatui::widgets::ListState::default();
                list_state.select(Some(state.selected_index));

                let list = List::new(items)
                    .style(Style::default().bg(bg_color).fg(fg_color))
                    .highlight_style(Style::default().bg(highlight_color).fg(Color::Black));

                frame.render_stateful_widget(list, Rect {
                    x: inner_area.x,
                    y: inner_area.y + 4,
                    width: inner_area.width,
                    height: inner_area.height - 5,
                }, &mut list_state);
            }
        }
        PaletteMode::Argument => {
            // نمایش فرم آرگومان
            let content = Paragraph::new("Argument input mode - implement form rendering")
                .style(Style::default().bg(bg_color).fg(fg_color))
                .alignment(Alignment::Center);
            frame.render_widget(content, inner_area);
        }
    }

    // نمایش راهنما در پایین
    let help_text = match state.mode {
        PaletteMode::Command => "↑↓ Navigate  |  Enter Execute  |  Tab Quick Execute  |  Esc Cancel",
        PaletteMode::Argument => "↑↓ Navigate  |  Enter Confirm  |  Esc Cancel",
        PaletteMode::Results => "↑↓ Navigate  |  Enter Select",
    };
    
    let help = Paragraph::new(help_text)
        .style(Style::default().bg(bg_color).fg(Color::Gray))
        .alignment(Alignment::Center);
    
    frame.render_widget(help, Rect {
        x: inner_area.x,
        y: inner_area.y + inner_area.height - 1,
        width: inner_area.width,
        height: 1,
    });
}
