//! Settings Panel for Professional TUI
//!
//! پنل تنظیمات کامل:
//! - General settings
//! - UI/Theme settings
//! - Keyboard shortcuts
//! - Agent settings
//! - Model settings
//! - LSP settings
//! - Memory settings

use crate::config::{Config, ThemeName, Theme};
use crate::tui::keyboard::KeyBindings;
use std::sync::{Arc, RwLock};

/// تنظیمات کامل برنامه
#[derive(Debug, Clone, Default)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub ui: UiSettings,
    pub keyboard: KeyboardSettings,
    pub agent: AgentSettings,
    pub model: ModelSettings,
    pub lsp: LspSettings,
    pub memory: MemorySettings,
}

#[derive(Debug, Clone, Default)]
pub struct GeneralSettings {
    pub auto_save: bool,
    pub save_interval: u64,
    pub max_history: usize,
    pub log_level: String,
    pub confirm_exit: bool,
    pub startup_session: String,
}

#[derive(Debug, Clone, Default)]
pub struct UiSettings {
    pub theme: ThemeName,
    pub show_line_numbers: bool,
    pub show_whitespace: bool,
    pub tab_width: u8,
    pub font_size: u8,
    pub status_bar_position: String,
    pub sidebar_width: u16,
    pub animations: bool,
}

#[derive(Debug, Clone, Default)]
pub struct KeyboardSettings {
    pub keybindings: KeyBindings,
    pub vim_mode: bool,
    pub quick_navigation: bool,
    pub alt_as_meta: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AgentSettings {
    pub default_agent: String,
    pub max_concurrent_tasks: usize,
    pub timeout_seconds: u64,
    pub retry_count: u8,
    pub auto_plan: bool,
    pub confirmation_required: bool,
    pub tool_timeout: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ModelSettings {
    pub default_model: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub context_window: u32,
    pub streaming: bool,
    pub api_base: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Default)]
pub struct LspSettings {
    pub enabled: bool,
    pub auto_start: bool,
    pub diagnostics: bool,
    pub completion: bool,
    pub hover: bool,
    pub servers: Vec<LspServerConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct LspServerConfig {
    pub name: String,
    pub language: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MemorySettings {
    pub short_term_limit: usize,
    pub long_term_enabled: bool,
    pub compression: bool,
    pub auto_prune: bool,
    pub prune_interval: u64,
}

/// مدیریت تنظیمات
#[derive(Clone)]
pub struct SettingsManager {
    pub settings: Arc<RwLock<AppSettings>>,
    pub config: Arc<RwLock<Config>>,
    pub modified: Arc<RwLock<bool>>,
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self {
            settings: Arc::new(RwLock::new(AppSettings::default())),
            config: Arc::new(RwLock::new(Config::default())),
            modified: Arc::new(RwLock::new(false)),
        }
    }
}

impl SettingsManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// بارگذاری تنظیمات از فایل
    pub fn load(&self) -> Result<(), String> {
        // TODO: بارگذاری از config file
        Ok(())
    }

    /// ذخیره تنظیمات
    pub fn save(&self) -> Result<(), String> {
        // TODO: ذخیره در فایل
        *self.modified.write() = false;
        Ok(())
    }

    /// تغییر تم
    pub fn set_theme(&self, theme: ThemeName) {
        let mut settings = self.settings.write();
        settings.ui.theme = theme;
        *self.modified.write() = true;
    }

    /// تغییر تنظیمات عمومی
    pub fn set_general(&self, general: GeneralSettings) {
        let mut settings = self.settings.write();
        settings.general = general;
        *self.modified.write() = true;
    }

    /// دریافت تمام تنظیمات
    pub fn get_all(&self) -> AppSettings {
        self.settings.read().clone()
    }

    /// بررسی تغییرات
    pub fn is_modified(&self) -> bool {
        *self.modified.read()
    }
}

/// تب‌های تنظیمات
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    UI,
    Keyboard,
    Agent,
    Model,
    LSP,
    Memory,
    About,
}

/// صفحه تنظیمات
#[derive(Clone)]
pub struct SettingsPage {
    pub manager: Arc<SettingsManager>,
    pub active_tab: SettingsTab,
    pub tabs: Vec<SettingsTab>,
    pub selected_item: usize,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self {
            manager: Arc::new(SettingsManager::new()),
            active_tab: SettingsTab::General,
            tabs: vec![
                SettingsTab::General,
                SettingsTab::UI,
                SettingsTab::Keyboard,
                SettingsTab::Agent,
                SettingsTab::Model,
                SettingsTab::LSP,
                SettingsTab::Memory,
                SettingsTab::About,
            ],
            selected_item: 0,
        }
    }
}

impl SettingsPage {
    pub fn new() -> Self {
        Self::default()
    }

    /// تعویض تب
    pub fn switch_tab(&mut self, tab: SettingsTab) {
        self.active_tab = tab;
        self.selected_item = 0;
    }

    /// حرکت به آیتم بعدی
    pub fn next_item(&mut self, count: usize) {
        self.selected_item = (self.selected_item + 1).min(count - 1);
    }

    /// حرکت به آیتم قبلی
    pub fn prev_item(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
        }
    }

    /// ذخیره تنظیمات
    pub fn save(&self) -> Result<(), String> {
        self.manager.save()
    }
}

/// رندر کردن Settings Panel
pub fn render_settings_panel<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    page: &SettingsPage,
    area: Rect,
    theme: &ThemeName,
) {
    let (bg_color, fg_color, highlight_color, accent_color) = match theme {
        ThemeName::DarkPlus => (
            Color::Rgb(30, 30, 30),
            Color::White,
            Color::Rgb(50, 50, 50),
            Color::Cyan,
        ),
        _ => (
            Color::Rgb(30, 30, 30),
            Color::White,
            Color::Rgb(50, 50, 50),
            Color::Cyan,
        ),
    };

    // رندر تب‌ها
    let tabs_content = page.tabs.iter().enumerate().map(|(i, tab)| {
        let prefix = if *tab == page.active_tab { "▶ " } else { "  " };
        let name = match tab {
            SettingsTab::General => "⚙️  General",
            SettingsTab::UI => "🎨  UI",
            SettingsTab::Keyboard => "⌨️  Keyboard",
            SettingsTab::Agent => "🤖  Agent",
            SettingsTab::Model => "🧠  Model",
            SettingsTab::LSP => "🔍  LSP",
            SettingsTab::Memory => "💾  Memory",
            SettingsTab::About => "ℹ️  About",
        };
        format!("{}{}", prefix, name)
    }).collect::<Vec<_>>().join("  ");

    let tabs_widget = Paragraph::new(tabs_content)
        .style(Style::default().bg(Color::Rgb(25, 25, 25)).fg(fg_color))
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(tabs_widget, Rect::new(area.x, area.y, area.width, 1));

    // محتوای تب فعال
    let content_area = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
    let content = render_settings_content(page, theme);

    let content_widget = Paragraph::new(content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(content_widget, content_area);
}

fn render_settings_content(page: &SettingsPage, theme: &ThemeName) -> String {
    let settings = page.manager.get_all();
    
    match page.active_tab {
        SettingsTab::General => format!(
            "⚙️ GENERAL SETTINGS\n\n\
            ☐ Auto Save              [{}]\n\
            ☐ Save Interval          [{} seconds]\n\
            ☐ Max History            [{} entries]\n\
            ☐ Log Level              [{}]\n\
            ☐ Confirm Exit           [{}]\n\
            ☐ Startup Session        [{}]",
            if settings.general.auto_save { "✓" } else { "✗" },
            settings.general.save_interval,
            settings.general.max_history,
            settings.general.log_level,
            if settings.general.confirm_exit { "✓" } else { "✗" },
            settings.general.startup_session
        ),
        SettingsTab::UI => format!(
            "🎨 UI SETTINGS\n\n\
            Theme:                   [{}]\n\
            ☐ Show Line Numbers      [{}]\n\
            ☐ Show Whitespace        [{}]\n\
            Tab Width:               [{} spaces]\n\
            Font Size:               [{}px]\n\
            Status Bar Position:     [{}]\n\
            Sidebar Width:           [{}]\n\
            ☐ Animations             [{}]",
            settings.ui.theme,
            if settings.ui.show_line_numbers { "✓" } else { "✗" },
            if settings.ui.show_whitespace { "✓" } else { "✗" },
            settings.ui.tab_width,
            settings.ui.font_size,
            settings.ui.status_bar_position,
            settings.ui.sidebar_width,
            if settings.ui.animations { "✓" } else { "✗" }
        ),
        SettingsTab::Keyboard => format!(
            "⌨️ KEYBOARD SETTINGS\n\n\
            ☐ Vim Mode               [{}]\n\
            ☐ Quick Navigation       [{}]\n\
            ☐ Alt as Meta            [{}]\n\
            \n\
            KEY BINDINGS:\n\
            Ctrl+P - Command Palette\n\
            Ctrl+M - Models\n\
            Ctrl+S - Settings\n\
            Ctrl+H - Help\n\
            Ctrl+X - Execute\n\
            Tab    - Switch Panel\n\
            Esc    - Close",
            if settings.keyboard.vim_mode { "✓" } else { "✗" },
            if settings.keyboard.quick_navigation { "✓" } else { "✗" },
            if settings.keyboard.alt_as_meta { "✓" } else { "✗" }
        ),
        SettingsTab::Agent => format!(
            "🤖 AGENT SETTINGS\n\n\
            Default Agent:           [{}]\n\
            Max Concurrent Tasks:    [{}]\n\
            Timeout:                 [{} seconds]\n\
            Retry Count:             [{}]\n\
            ☐ Auto Plan              [{}]\n\
            ☐ Confirmation Required  [{}]\n\
            Tool Timeout:            [{} seconds]",
            settings.agent.default_agent,
            settings.agent.max_concurrent_tasks,
            settings.agent.timeout_seconds,
            settings.agent.retry_count,
            if settings.agent.auto_plan { "✓" } else { "✗" },
            if settings.agent.confirmation_required { "✓" } else { "✗" },
            settings.agent.tool_timeout
        ),
        SettingsTab::Model => format!(
            "🧠 MODEL SETTINGS\n\n\
            Default Model:           [{}]\n\
            Temperature:             [{:.2}]\n\
            Max Tokens:              [{}]\n\
            Context Window:          [{}]\n\
            ☐ Streaming              [{}]\n\
            API Base:                [{}]\n\
            API Key:                 [{}]",
            settings.model.default_model,
            settings.model.temperature,
            settings.model.max_tokens,
            settings.model.context_window,
            if settings.model.streaming { "✓" } else { "✗" },
            settings.model.api_base,
            if settings.model.api_key.is_empty() { "Not set" } else { "••••••••" }
        ),
        SettingsTab::LSP => format!(
            "🔍 LSP SETTINGS\n\n\
            ☐ Enabled                [{}]\n\
            ☐ Auto Start             [{}]\n\
            ☐ Diagnostics            [{}]\n\
            ☐ Completion             [{}]\n\
            ☐ Hover                  [{}]\n\
            \n\
            CONFIGURED SERVERS:",
            if settings.lsp.enabled { "✓" } else { "✗" },
            if settings.lsp.auto_start { "✓" } else { "✗" },
            if settings.lsp.diagnostics { "✓" } else { "✗" },
            if settings.lsp.completion { "✓" } else { "✗" },
            if settings.lsp.hover { "✓" } else { "✗" }
        ),
        SettingsTab::Memory => format!(
            "💾 MEMORY SETTINGS\n\n\
            Short-term Limit:        [{} entries]\n\
            ☐ Long-term Enabled      [{}]\n\
            ☐ Compression            [{}]\n\
            ☐ Auto Prune             [{}]\n\
            Prune Interval:          [{} seconds]",
            settings.memory.short_term_limit,
            if settings.memory.long_term_enabled { "✓" } else { "✗" },
            if settings.memory.compression { "✓" } else { "✗" },
            if settings.memory.auto_prune { "✓" } else { "✗" },
            settings.memory.prune_interval
        ),
        SettingsTab::About => format!(
            "ℹ️ ABOUT\n\n\
            Super-Agent v0.1\n\
            Multi-Agent CLI System\n\
            \n\
            Built with:\n\
            • Rust 1.75+\n\
            • Tokio Async Runtime\n\
            • Ratatui TUI\n\
            • LLM Integration\n\
            \n\
            GitHub: supercode-ai\n\
            License: MIT"
        ),
    }
}

/// خلاصه فایل‌های ایجاد شده
const FILES_SUMMARY: &str = r#"
## 📁 فایل‌های TUI ایجاد شده

### Core Components
├── src/tui/keyboard.rs              ⌨️ 30+ کیبورد استاندارد
├── src/tui/state/mod.rs             📊 State Management
├── src/tui/components/mod.rs        📦 Export Module

### UI Components
├── src/tui/components/sidebar.rs    📊 Sidebar Monitoring
├── src/tui/components/command_palette.rs  🎯 Command Palette
├── src/tui/components/slash_command.rs    ⚡ Slash Commands
├── src/tui/components/task_manager.rs     ✅ Task Queue
├── src/tui/components/multi_session.rs    💻 Multi-Session
├── src/tui/components/lsp_support.rs      🔍 LSP Support
└── src/tui/components/settings_panel.rs   ⚙️ Settings Panel

## 🚀 ویژگی‌های پیاده‌سازی شده

### Keyboard Shortcuts (30+)
- Ctrl+P - Command Palette
- Ctrl+M - Models Panel
- Ctrl+S - Settings
- Ctrl+H - Help
- Ctrl+A - Agent Settings
- Ctrl+X - Execute
- Ctrl+T - New Task
- Ctrl+N - New Session
- Tab - Switch Panel
- Esc - Close
- F1-F12 - Function Keys

### Command Palette
- 16 دستور پایه
- 8 دسته‌بندی
- Fuzzy matching
- Recent commands

### Slash Commands
- /task - مدیریت تسک
- /agent - تنظیمات agent
- /model - تغییر مدل
- /context - مدیریت context
- /memory - مدیریت حافظه
- /history - تاریخچه
- /edit - ویرایش
- /find - جستجو
- /format - فرمت
- /settings - تنظیمات
- /help - راهنما
- /goto - رفتن به خط
- /open - باز کردن فایل
- /close - بستن

### Task Manager
- صف اولویت‌دار
- وابستگی تسک‌ها
- مراحل تسک
- آمار و گزارش
- Non-blocking execution

### Multi-Session
- ایجاد/حذف/تغییر session
- تب‌های متعدد
- Shared memory
- Cross-session events

### LSP Support
- Diagnostics
- Auto-completion
- Hover
- Goto definition
- Find references

### Settings Panel
- General
- UI/Theme
- Keyboard
- Agent
- Model
- LSP
- Memory
- About
"#;

pub fn get_files_summary() -> &'static str {
    FILES_SUMMARY
}
