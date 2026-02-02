//! Professional Keyboard Shortcuts Manager
//!
//! کیبورد شورتکات‌های استاندارد مطابق IDE‌های حرفه‌ای و CLI‌های مدرن

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// نوع عملیات
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyAction {
    // Command Palette & Navigation
    CommandPalette,
    QuickOpen,
    GoToLine,
    GoToFile,
    GoToSymbol,
    
    // Editing
    Save,
    SaveAll,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    SelectAll,
    DeleteLine,
    
    // View
    ToggleSidebar,
    ToggleTerminal,
    ToggleFullscreen,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    
    // Tasks & Sessions
    NewTask,
    ListTasks,
    TaskDetails,
    NewSession,
    SwitchSession,
    CloseSession,
    
    // Models
    ListModels,
    SwitchModel,
    ImportModel,
    
    // Settings & Help
    Settings,
    Help,
    AgentsSettings,
    
    // Agent Interaction
    SendMessage,
    SwitchToTerminal,
    SwitchToChat,
    InterruptAgent,
    
    // Navigation
    NextTab,
    PrevTab,
    CloseTab,
    NextPanel,
    PrevPanel,
    
    // Special
    Escape,
    Enter,
    Tab,
    F1,
    F2,
    F5,
    F12,
}

/// تنظیمات کیبورد
#[derive(Debug, Clone)]
pub struct KeyBindings {
    bindings: HashMap<KeyAction, Vec<KeyEvent>>,
    reverse_bindings: HashMap<String, KeyAction>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyBindings {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();
        let mut reverse = HashMap::new();

        // ========== Command Palette & Navigation ==========
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::CommandPalette, vec![
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT), // Alt+P also works
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::QuickOpen, vec![
            KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::GoToLine, vec![
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::GoToFile, vec![
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::GoToSymbol, vec![
            KeyEvent::new(KeyCode::Char('o'), KeyModifiers::ALT),
        ]);

        // ========== Editing ==========
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::Save, vec![
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::SaveAll, vec![
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::Undo, vec![
            KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::Redo, vec![
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::Copy, vec![
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::Cut, vec![
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::Paste, vec![
            KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::SelectAll, vec![
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
        ]);

        // ========== View ==========
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::ToggleSidebar, vec![
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::ToggleTerminal, vec![
            KeyEvent::new(KeyCode::Char('`'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::ZoomIn, vec![
            KeyEvent::new(KeyCode::Char('+'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('='), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::ZoomOut, vec![
            KeyEvent::new(KeyCode::Char('-'), KeyModifiers::CONTROL),
        ]);

        // ========== Tasks & Sessions ==========
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::NewTask, vec![
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::ListTasks, vec![
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::ALT),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::NewSession, vec![
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::SwitchSession, vec![
            KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::CloseSession, vec![
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        ]);

        // ========== Models ==========
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::ListModels, vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::SwitchModel, vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::ALT),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::ImportModel, vec![
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        ]);

        // ========== Settings & Help ==========
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::Settings, vec![
            KeyEvent::new(KeyCode::Char(','), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::Help, vec![
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::AgentsSettings, vec![
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
        ]);

        // ========== Agent Interaction ==========
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::SendMessage, vec![
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::SwitchToTerminal, vec![
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::SwitchToChat, vec![
            KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::InterruptAgent, vec![
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ]);

        // ========== Navigation ==========
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::NextTab, vec![
            KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::PrevTab, vec![
            KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::CloseTab, vec![
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::NextPanel, vec![
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::ALT),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::PrevPanel, vec![
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT),
        ]);

        // ========== Special ==========
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::Escape, vec![
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::Tab, vec![
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::F5, vec![
            KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE),
        ]);
        
        Self::add_binding(&mut bindings, &mut reverse, KeyAction::F12, vec![
            KeyEvent::new(KeyCode::F(12), KeyModifiers::NONE),
        ]);

        Self { bindings, reverse_bindings: reverse }
    }

    fn add_binding(
        bindings: &mut HashMap<KeyAction, Vec<KeyEvent>>,
        reverse: &mut HashMap<String, KeyAction>,
        action: KeyAction,
        events: Vec<KeyEvent>,
    ) {
        for event in &events {
            let key = format!("{:?}+{:?}+{:?}", 
                event.modifiers, 
                event.code, 
                if event.modifiers.contains(KeyModifiers::SHIFT) { "Shift" } else { "" }
            );
            reverse.insert(key.clone(), action.clone());
        }
        bindings.insert(action, events);
    }

    /// پیدا کردن action对应的键绑定
    pub fn find_action(&self, key: KeyEvent) -> Option<KeyAction> {
        // دقیق تطبیق
        if let Some(action) = self.reverse_bindings.get(&format!("{:?}+{:?}+", key.modifiers, key.code)) {
            return Some(action.clone());
        }
        
        // تطبیق با shift
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            let without_shift = KeyEvent::new(key.code, key.modifiers - KeyModifiers::SHIFT);
            if let Some(action) = self.reverse_bindings.get(&format!("{:?}+{:?}+", without_shift.modifiers, without_shift.code)) {
                return Some(action.clone());
            }
        }

        None
    }

    /// دریافت تمام binding‌های یک action
    pub fn get_bindings(&self, action: &KeyAction) -> Vec<KeyEvent> {
        self.bindings.get(action).cloned().unwrap_or_default()
    }

    /// بررسی اینکه آیا این کلید یک shortcut استاندارد است
    pub fn is_standard_shortcut(&self, key: KeyEvent) -> bool {
        self.find_action(key).is_some()
    }

    /// نمایش تمام shortcut‌ها
    pub fn print_bindings(&self) {
        println!("\n📋 Professional Keyboard Shortcuts\n");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let categories = [
            ("🎯 Command Palette & Navigation", vec![
                KeyAction::CommandPalette, KeyAction::QuickOpen, KeyAction::GoToLine,
                KeyAction::GoToFile, KeyAction::GoToSymbol,
            ]),
            ("✏️ Editing", vec![
                KeyAction::Save, KeyAction::SaveAll, KeyAction::Undo, KeyAction::Redo,
                KeyAction::Copy, KeyAction::Cut, KeyAction::Paste,
            ]),
            ("👁️ View", vec![
                KeyAction::ToggleSidebar, KeyAction::ToggleTerminal,
                KeyAction::ZoomIn, KeyAction::ZoomOut,
            ]),
            ("📋 Tasks & Sessions", vec![
                KeyAction::NewTask, KeyAction::ListTasks, KeyAction::NewSession,
                KeyAction::SwitchSession, KeyAction::CloseSession,
            ]),
            ("🤖 Models", vec![
                KeyAction::ListModels, KeyAction::SwitchModel, KeyAction::ImportModel,
            ]),
            ("⚙️ Settings & Help", vec![
                KeyAction::Settings, KeyAction::Help, KeyAction::AgentsSettings,
            ]),
            ("💬 Agent Interaction", vec![
                KeyAction::SendMessage, KeyAction::SwitchToTerminal,
                KeyAction::SwitchToChat, KeyAction::InterruptAgent,
            ]),
            ("🔀 Navigation", vec![
                KeyAction::NextTab, KeyAction::PrevTab, KeyAction::CloseTab,
                KeyAction::NextPanel, KeyAction::PrevPanel,
            ]),
        ];

        for (category, actions) in categories {
            println!("\n{}", category);
            println!("───");
            for action in actions {
                if let Some(bindings) = self.bindings.get(action) {
                    for binding in bindings {
                        let key_str = format_key_event(binding);
                        println!("  {}  →  {}", format_action_name(action), key_str);
                    }
                }
            }
        }
        
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}

/// فرمت کردن نام action
fn format_action_name(action: &KeyAction) -> String {
    match action {
        KeyAction::CommandPalette => "Command Palette".to_string(),
        KeyAction::QuickOpen => "Quick Open".to_string(),
        KeyAction::GoToLine => "Go to Line".to_string(),
        KeyAction::GoToFile => "Go to File".to_string(),
        KeyAction::GoToSymbol => "Go to Symbol".to_string(),
        KeyAction::Save => "Save".to_string(),
        KeyAction::SaveAll => "Save All".to_string(),
        KeyAction::Undo => "Undo".to_string(),
        KeyAction::Redo => "Redo".to_string(),
        KeyAction::Copy => "Copy".to_string(),
        KeyAction::Cut => "Cut".to_string(),
        KeyAction::Paste => "Paste".to_string(),
        KeyAction::SelectAll => "Select All".to_string(),
        KeyAction::DeleteLine => "Delete Line".to_string(),
        KeyAction::ToggleSidebar => "Toggle Sidebar".to_string(),
        KeyAction::ToggleTerminal => "Toggle Terminal".to_string(),
        KeyAction::ToggleFullscreen => "Toggle Fullscreen".to_string(),
        KeyAction::ZoomIn => "Zoom In".to_string(),
        KeyAction::ZoomOut => "Zoom Out".to_string(),
        KeyAction::ResetZoom => "Reset Zoom".to_string(),
        KeyAction::NewTask => "New Task".to_string(),
        KeyAction::ListTasks => "List Tasks".to_string(),
        KeyAction::TaskDetails => "Task Details".to_string(),
        KeyAction::NewSession => "New Session".to_string(),
        KeyAction::SwitchSession => "Switch Session".to_string(),
        KeyAction::CloseSession => "Close Session".to_string(),
        KeyAction::ListModels => "List Models".to_string(),
        KeyAction::SwitchModel => "Switch Model".to_string(),
        KeyAction::ImportModel => "Import Model".to_string(),
        KeyAction::Settings => "Settings".to_string(),
        KeyAction::Help => "Help".to_string(),
        KeyAction::AgentsSettings => "Agents Settings".to_string(),
        KeyAction::SendMessage => "Send Message".to_string(),
        KeyAction::SwitchToTerminal => "Switch to Terminal".to_string(),
        KeyAction::SwitchToChat => "Switch to Chat".to_string(),
        KeyAction::InterruptAgent => "Interrupt Agent".to_string(),
        KeyAction::NextTab => "Next Tab".to_string(),
        KeyAction::PrevTab => "Prev Tab".to_string(),
        KeyAction::CloseTab => "Close Tab".to_string(),
        KeyAction::NextPanel => "Next Panel".to_string(),
        KeyAction::PrevPanel => "Prev Panel".to_string(),
        KeyAction::Escape => "Escape".to_string(),
        KeyAction::Enter => "Enter".to_string(),
        KeyAction::Tab => "Tab".to_string(),
        KeyAction::F1 => "F1".to_string(),
        KeyAction::F2 => "F2".to_string(),
        KeyAction::F5 => "F5".to_string(),
        KeyAction::F12 => "F12".to_string(),
    }
}

/// فرمت کردن نمایش کلید
fn format_key_event(event: &KeyEvent) -> String {
    let mut parts = Vec::new();
    
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl".to_string());
    }
    if event.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if event.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift".to_string());
    }
    
    let key = match event.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => format!("{:?}", event.code),
    };
    
    parts.push(key);
    parts.join("+")
}

/// مدیر کیبورد مرکزی
#[derive(Clone)]
pub struct KeyboardManager {
    bindings: Arc<KeyBindings>,
    input_mode: Arc<RwLock<InputMode>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Chat,      // پیامک به agent
    Terminal,  // اجرای دستورات
    Command,   // در command palette
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Chat
    }
}

impl KeyboardManager {
    pub fn new() -> Self {
        Self {
            bindings: Arc::new(KeyBindings::new()),
            input_mode: Arc::new(RwLock::new(InputMode::Chat)),
        }
    }

    /// مدیریت کلید و برگرداندن action
    pub fn handle_key(&self, key: KeyEvent) -> Option<KeyAction> {
        // بررسی کلیدهای عمومی
        if let Some(action) = self.bindings.find_action(key) {
            return Some(action);
        }
        
        // مدیریت Tab برای سوییچ بین terminal و chat
        if key.code == KeyCode::Tab {
            let mut mode = self.input_mode.write();
            *mode = match *mode {
                InputMode::Chat => InputMode::Terminal,
                InputMode::Terminal => InputMode::Chat,
                InputMode::Command => InputMode::Chat,
            };
            return Some(KeyAction::Tab);
        }
        
        None
    }

    /// دریافت input mode فعلی
    pub fn get_input_mode(&self) -> InputMode {
        *self.input_mode.read()
    }

    /// تنظیم input mode
    pub fn set_input_mode(&self, mode: InputMode) {
        *self.input_mode.write() = mode;
    }

    /// نمایش راهنمای کیبورد
    pub fn print_shortcuts_help(&self) {
        self.bindings.print_bindings();
    }

    /// دریافت راهنمای context-aware
    pub fn get_context_help(&self) -> String {
        let mode = self.get_input_mode();
        match mode {
            InputMode::Chat => {
                "💬 Chat Mode\n\
                ━━━━━━━━━━━━━\n\
                • Type message → Enter to send\n\
                • /command → Command palette\n\
                • Tab → Switch to Terminal\n\
                • Ctrl+C → Interrupt agent".to_string()
            }
            InputMode::Terminal => {
                "💻 Terminal Mode\n\
                ━━━━━━━━━━━━━━━━\n\
                • Type command → Enter to run\n\
                • Ctrl+Z → Undo\n\
                • Ctrl+C → Cancel\n\
                • Tab → Switch to Chat".to_string()
            }
            InputMode::Command => {
                "🎯 Command Mode\n\
                ━━━━━━━━━━━━━━━━\n\
                • Type to filter commands\n\
                • ↑/↓ or j/k → Navigate\n\
                • Enter → Execute\n\
                • Esc → Cancel".to_string()
            }
        }
    }
}
