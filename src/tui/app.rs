use crate::config::{RuntimeConfig, ThemeName};
use crate::models::native::{NativeModelManager, NativeProvider, NativeModelInfo, LoadState};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, Terminal, widgets::{Block, Borders, Paragraph, Wrap, List, ListItem}, text::{Span, Spans}, style::{Style, Color, Modifier}};
use std::io;
use std::sync::{Arc};
use std::time::{Duration, Instant};

pub enum ModelProviderType {
    /// استفاده از Native Provider (اجرای مستقیم مدل)
    Native,
    /// استفاده از llama.cpp
    LlamaCpp,
    /// استفاده از Ollama
    Ollama,
}

pub struct TuiApp {
    last_tick: Instant,
    tick_rate: Duration,
    pub config: Arc<std::sync::RwLock<RuntimeConfig>>,
    pub input: String,
    pub messages: Vec<String>,
    pub logs: Vec<String>,
    pub palette_open: bool,
    pub palette_query: String,
    pub theme_list: Vec<ThemeName>,
    pub selected_theme: usize,
    pub model_panel_open: bool,
    pub models_cached: Vec<String>,
    pub model_selected: usize,
    pub expecting_import: bool,
    // Native Models
    pub native_model_panel_open: bool,
    pub native_models_cached: Vec<String>,
    pub native_model_selected: usize,
    pub native_model_loaded: Option<String>, // نام مدل بارگذاری شده
    pub current_provider: ModelProviderType,
}

impl TuiApp {
    pub fn new(config: RuntimeConfig) -> anyhow::Result<Self> {
        let theme_list = vec![
            ThemeName::DarkPlus,
            ThemeName::Light,
            ThemeName::Monokai,
            ThemeName::SolarizedDark,
            ThemeName::SolarizedLight,
            ThemeName::Dracula,
            ThemeName::OneDark,
            ThemeName::Nord,
            ThemeName::Gruvbox,
            ThemeName::Peacocks,
        ];
        Ok(Self {
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(100),
            config: Arc::new(std::sync::RwLock::new(config)),
            input: String::new(),
            messages: vec![],
            logs: vec!["SuperAgentCli started".into(), "Native Provider support enabled".into()],
            palette_open: false,
            palette_query: String::new(),
            theme_list,
            selected_theme: 0,
            model_panel_open: false,
            models_cached: vec![],
            model_selected: 0,
            expecting_import: false,
            // Native Models
            native_model_panel_open: false,
            native_models_cached: vec![],
            native_model_selected: 0,
            native_model_loaded: None,
            current_provider: ModelProviderType::LlamaCpp, // پیش‌فرض
        })
    }

    fn resolve_theme_style(&self, t: &ThemeName) -> Style {
        match t {
            ThemeName::DarkPlus => Style::default().bg(Color::Black).fg(Color::White),
            ThemeName::Light => Style::default().bg(Color::White).fg(Color::Black),
            ThemeName::Monokai => Style::default().bg(Color::Black).fg(Color::Green),
            ThemeName::SolarizedDark => Style::default().bg(Color::Rgb(0,43,54)).fg(Color::Rgb(131,148,150)),
            ThemeName::SolarizedLight => Style::default().bg(Color::Rgb(253,246,227)).fg(Color::Rgb(101,123,131)),
            ThemeName::Dracula => Style::default().bg(Color::Rgb(40,42,54)).fg(Color::Rgb(248,248,242)),
            ThemeName::OneDark => Style::default().bg(Color::Rgb(40,44,52)).fg(Color::Rgb(171,178,191)),
            ThemeName::Nord => Style::default().bg(Color::Rgb(46,52,64)).fg(Color::Rgb(216,222,233)),
            ThemeName::Gruvbox => Style::default().bg(Color::Rgb(40,40,40)).fg(Color::Rgb(235,219,178)),
            ThemeName::Peacocks => Style::default().bg(Color::Rgb(10,20,30)).fg(Color::Rgb(80,220,150)),
        }
    }

    fn get_config_clone(&self) -> RuntimeConfig {
        match self.config.read() {
            Ok(cfg) => cfg.clone(),
            Err(_) => RuntimeConfig::default(),
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let stdout = io::stdout();
        crossterm::terminal::enable_raw_mode()?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        loop {
            // drain any UI status messages from background tasks
            while let Ok(msg) = rx.try_recv() {
                // Handle special messages
                if msg.starts_with("__native_models__::") {
                    let data = msg.strip_prefix("__native_models__::").unwrap_or("");
                    if data.is_empty() {
                        self.native_models_cached = vec![];
                    } else {
                        self.native_models_cached = data.split("||").map(|s| s.to_string()).collect();
                    }
                    continue;
                }
                if msg.starts_with("__loaded__::") {
                    let name = msg.strip_prefix("__loaded__::").unwrap_or("").to_string();
                    self.native_model_loaded = Some(name.clone());
                    self.logs.push(format!("Native model loaded: {}", name));
                    continue;
                }
                if msg.starts_with("__deleted__::") {
                    let name = msg.strip_prefix("__deleted__::").unwrap_or("").to_string();
                    if self.native_model_loaded.as_ref() == Some(&name) {
                        self.native_model_loaded = None;
                    }
                    // Refresh list
                    let cfg = self.get_config_clone();
                    if let Ok(manager) = NativeModelManager::new(cfg.model_dir.clone()) {
                        let txc = tx.clone();
                        tokio::spawn(async move {
                            if let Ok(ms) = manager.discover().await {
                                let names: Vec<String> = ms.into_iter()
                                    .map(|m| format!("{} ({})", m.name, m.format.display_name()))
                                    .collect();
                                let _ = txc.send(format!("__native_models__::{}", names.join("||")));
                            }
                        });
                    }
                    self.logs.push(format!("Native model deleted: {}", name));
                    continue;
                }
                if msg.starts_with("__error__::") {
                    let error = msg.strip_prefix("__error__::").unwrap_or("");
                    self.logs.push(format!("Error: {}", error));
                    continue;
                }
                
                self.logs.push(msg);
                if self.logs.len() > 500 { self.logs.remove(0); }
            }
            let cfg = self.get_config_clone();
            let style = self.resolve_theme_style(&cfg.theme);

            terminal.draw(|f| {
                let size = f.size();

                // top title
                let top = Rect { x: size.x, y: size.y, width: size.width, height: 3 };
                let title = Paragraph::new(Spans::from(vec![Span::styled("SuperAgentCli", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]))
                    .block(Block::default().borders(Borders::ALL).title(Span::styled(" SuperAgentCli ", Style::default().add_modifier(Modifier::BOLD))));
                f.render_widget(title, top);

                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)].as_ref())
                    .split(Rect { x: size.x, y: size.y + 3, width: size.width, height: size.height - 3 });

                // Left - graph placeholder
                let left = Paragraph::new("Agent Graph\n- planner\n- executor\n- critic")
                    .block(Block::default().borders(Borders::ALL).title("Graph"))
                    .wrap(Wrap { trim: true });
                f.render_widget(left.clone().style(style), chunks[0]);

                // Center - chat messages
                let msg_items: Vec<ListItem> = self.messages.iter().map(|m| ListItem::new(Spans::from(Span::raw(m.clone())))).collect();
                let msg_list = List::new(msg_items).block(Block::default().borders(Borders::ALL).title("Chat"));
                f.render_widget(msg_list.clone().style(style), chunks[1]);

                // Right - logs and settings quick
                let logs_items: Vec<ListItem> = self.logs.iter().rev().take(10).map(|l| ListItem::new(Spans::from(Span::raw(l.clone())))).collect();
                let logs_list = List::new(logs_items).block(Block::default().borders(Borders::ALL).title("Logs / Settings"));
                f.render_widget(logs_list.clone().style(style), chunks[2]);

                // If models panel open, render overlay
                if self.model_panel_open {
                    let area = centered_rect(50, 40, size);
                    let items: Vec<ListItem> = self.models_cached.iter().map(|n| ListItem::new(Spans::from(Span::raw(n.clone())))).collect();
                    let mut state = ratatui::widgets::ListState::default();
                    if !self.models_cached.is_empty() {
                        state.select(Some(self.model_selected.min(self.models_cached.len()-1)));
                    }
                    let title = format!("External Models (llama.cpp/Ollama) - Press 'L' for Native Models");
                    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
                    f.render_stateful_widget(list, &mut state, area);
                }

                // If native models panel open, render overlay
                if self.native_model_panel_open {
                    let area = centered_rect(60, 50, size);
                    
                    // ساخت لیست با اطلاعات بیشتر
                    let items: Vec<ListItem> = self.native_models_cached.iter().map(|n| {
                        let display = if let Some(loaded) = &self.native_model_loaded {
                            if n.contains(loaded.as_str()) {
                                format!("{} [LOADED]", n)
                            } else {
                                n.clone()
                            }
                        } else {
                            n.clone()
                        };
                        ListItem::new(Spans::from(Span::raw(display)))
                    }).collect();
                    
                    let mut state = ratatui::widgets::ListState::default();
                    if !self.native_models_cached.is_empty() {
                        state.select(Some(self.native_model_selected.min(self.native_models_cached.len()-1)));
                    }
                    
                    let provider_text = match self.current_provider {
                        ModelProviderType::Native => "Native (Direct)",
                        ModelProviderType::LlamaCpp => "llama.cpp",
                        ModelProviderType::Ollama => "Ollama",
                    };
                    
                    let title = format!("Native Models (GGUF/SafeTensors) | Provider: {} | 'L' switch | 'Enter' load/unload", provider_text);
                    let help = "[Enter] Load/Unload  |  [D] Delete  |  [I] Import path  |  [L] Switch provider  |  [Esc] Close";
                    let panel = Paragraph::new(help)
                        .block(Block::default().borders(Borders::TOP).title("Help"));
                    
                    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
                    f.render_widget(list.clone().style(style), area);
                    f.render_widget(panel, Rect { x: area.x, y: area.y + area.height, width: area.width, height: 3 });
                    f.render_stateful_widget(list, &mut state, area);
                }

                // bottom input box
                let bottom = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(3), Constraint::Length(3)]).split(chunks[1]);
                let input = Paragraph::new(self.input.as_ref()).block(Block::default().borders(Borders::ALL).title("/ to open command palette | Enter to send | Ctrl-s save config"));
                f.render_widget(input.clone().style(style), bottom[1]);

                // If palette open, render overlay
                if self.palette_open {
                    let area = centered_rect(60, 20, size);
                    let q = if self.palette_query.is_empty() { "Type command..." } else { &self.palette_query };
                    let items = vec!["Settings", "Themes", "Models", "Agents", "Graph", "Logs", "Exit"];
                    let filtered: Vec<&str> = items.iter().cloned().filter(|it| it.to_lowercase().contains(&q.to_lowercase())).collect();
                    let list_items: Vec<ListItem> = filtered.iter().map(|s| ListItem::new(Spans::from(Span::raw(*s)))).collect();
                    let list = List::new(list_items).block(Block::default().borders(Borders::ALL).title("Command Palette"));
                    f.render_widget(list, area);
                }
            })?;

            // Input handling
            if crossterm::event::poll(self.tick_rate)? {
                if let CEvent::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') if key.modifiers == KeyModifiers::NONE => {
                            break;
                        }
                        // Native Models Panel - 'n' key
                        KeyCode::Char('n') if key.modifiers == KeyModifiers::NONE => {
                            self.native_model_panel_open = true;
                            self.model_panel_open = false;
                            // refresh native model list
                            let cfg = self.get_config_clone();
                            if let Ok(manager) = NativeModelManager::new(cfg.model_dir.clone()) {
                                let txc = tx.clone();
                                tokio::spawn(async move {
                                    match manager.discover().await {
                                        Ok(ms) => {
                                            let names: Vec<String> = ms.into_iter()
                                                .map(|m| format!("{} ({})", m.name, m.format.display_name()))
                                                .collect();
                                            let _ = txc.send(format!("__native_models__::{}", names.join("||")));
                                        }
                                        Err(e) => {
                                            let _ = txc.send(format!("__error__::Native model discover: {}", e));
                                        }
                                    }
                                });
                                self.logs.push("Opened Native Models panel".into());
                            }
                        }
                        // Switch to llama.cpp models - 'l' key
                        KeyCode::Char('l') if key.modifiers == KeyModifiers::NONE => {
                            if self.native_model_panel_open {
                                // cycle through providers
                                self.current_provider = match self.current_provider {
                                    ModelProviderType::Native => ModelProviderType::LlamaCpp,
                                    ModelProviderType::LlamaCpp => ModelProviderType::Ollama,
                                    ModelProviderType::Ollama => ModelProviderType::Native,
                                };
                                let provider_name = match self.current_provider {
                                    ModelProviderType::Native => "Native",
                                    ModelProviderType::LlamaCpp => "llama.cpp",
                                    ModelProviderType::Ollama => "Ollama",
                                };
                                self.logs.push(format!("Switched to provider: {}", provider_name));
                            }
                        }
                        KeyCode::Char('m') if key.modifiers == KeyModifiers::NONE => {
                            // open models panel
                            self.model_panel_open = true;
                            // refresh model list
                            let cfg = self.get_config_clone();
                            if let Ok(mgr) = crate::models::ModelManager::new(Some(cfg.model_dir.clone())) {
                                match mgr.discover() {
                                    Ok(ms) => { self.models_cached = ms.into_iter().map(|m| format!("{} ({})", m.name, m.format)).collect(); }
                                    Err(e) => { self.logs.push(format!("model discover err: {}", e)); }
                                }
                            }
                        }
                        KeyCode::Char('i') if key.modifiers == KeyModifiers::NONE => {
                            if self.model_panel_open {
                                self.expecting_import = true;
                                self.input.clear();
                                self.logs.push("Type path to model and press Enter".into());
                            } else if self.native_model_panel_open {
                                self.expecting_import = true;
                                self.input.clear();
                                self.logs.push("Type path to native model (GGUF/SafeTensors) and press Enter".into());
                            }
                        }
                        KeyCode::Up => {
                            if self.model_panel_open && !self.models_cached.is_empty() {
                                if self.model_selected > 0 { self.model_selected -= 1; }
                            } else if self.native_model_panel_open && !self.native_models_cached.is_empty() {
                                if self.native_model_selected > 0 { self.native_model_selected -= 1; }
                            }
                        }
                        KeyCode::Down => {
                            if self.model_panel_open && !self.models_cached.is_empty() {
                                let lim = self.models_cached.len();
                                if self.model_selected + 1 < lim { self.model_selected += 1; }
                            } else if self.native_model_panel_open && !self.native_models_cached.is_empty() {
                                let lim = self.native_models_cached.len();
                                if self.native_model_selected + 1 < lim { self.native_model_selected += 1; }
                            }
                        }
                        // Native Models: Enter to load/unload
                        KeyCode::Enter if self.native_model_panel_open && !self.native_models_cached.is_empty() => {
                            if let Some(entry) = self.native_models_cached.get(self.native_model_selected).cloned() {
                                let name = entry.split(' ').next().unwrap_or("").to_string();
                                
                                // Check if already loaded
                                if self.native_model_loaded.as_ref() == Some(&name) {
                                    // Unload
                                    self.native_model_loaded = None;
                                    self.logs.push(format!("Unloaded native model: {}", name));
                                } else {
                                    // Load
                                    let cfg = self.get_config_clone();
                                    let txc = tx.clone();
                                    let name_c = name.clone();
                                    tokio::spawn(async move {
                                        // Create provider and load
                                        let provider = NativeProvider::new(cfg.model_dir.join(&name_c));
                                        match provider.load().await {
                                            Ok(_) => {
                                                let _ = txc.send(format!("__loaded__::{}", name_c));
                                            }
                                            Err(e) => {
                                                let _ = txc.send(format!("__error__::Failed to load {}: {}", name_c, e));
                                            }
                                        }
                                    });
                                    self.logs.push(format!("Loading native model: {}...", name));
                                }
                            }
                        }
                        // Native Models: 'd' to delete
                        KeyCode::Char('d') if self.native_model_panel_open && !self.native_models_cached.is_empty() => {
                            if let Some(entry) = self.native_models_cached.get(self.native_model_selected).cloned() {
                                let name = entry.split(' ').next().unwrap_or("").to_string();
                                let cfg = self.get_config_clone();
                                let txc = tx.clone();
                                tokio::spawn(async move {
                                    match NativeModelManager::new(cfg.model_dir.clone()).await {
                                        Ok(manager) => {
                                            match manager.remove_model(&name).await {
                                                Ok(_) => {
                                                    let _ = txc.send(format!("__deleted__::{}", name));
                                                }
                                                Err(e) => {
                                                    let _ = txc.send(format!("__error__::Delete failed: {}", e));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let _ = txc.send(format!("__error__::Manager error: {}", e));
                                        }
                                    }
                                });
                                self.logs.push(format!("Deleting: {}", name));
                            }
                        }
                        KeyCode::Char('s') if key.modifiers == KeyModifiers::NONE => {
                            if self.model_panel_open {
                                // start provider for selected in background
                                if let Some(entry) = self.models_cached.get(self.model_selected).cloned() {
                                    // entry format: "name (format)"
                                    let name = entry.split(' ').next().unwrap_or("").to_string();
                                    let cfg = self.get_config_clone();
                                    if let Ok(mgr) = crate::models::ModelManager::new(Some(cfg.model_dir.clone())) {
                                        if let Ok(ms) = mgr.discover() {
                                            if let Some(minfo) = ms.into_iter().find(|m| m.name == name) {
                                                let server = crate::models::ModelServer::new(std::sync::Arc::new(mgr), cfg.model_server_addr);
                                                let txc = tx.clone();
                                                let name_c = name.clone();
                                                let minfo_path = minfo.path.clone();
                                                let addr = cfg.model_server_addr;
                                                // spawn background task to start server and provider
                                                tokio::spawn(async move {
                                                    if let Err(e) = server.start_local_server().await {
                                                        let _ = txc.send(format!("failed to start local server: {}", e));
                                                        return;
                                                    }
                                                    let p = std::sync::Arc::new(crate::models::server::LlamaProvider::new(None, minfo_path, addr));
                                                    match p.start().await {
                                                        Ok(_) => {
                                                            if let Err(e) = server.register_provider(&name_c, std::sync::Arc::clone(&p)).await {
                                                                let _ = txc.send(format!("Started provider but register failed: {}", e));
                                                            } else {
                                                                let _ = txc.send(format!("Started llama provider for {}", name_c));
                                                            }
                                                        }
                                                        Err(e) => {
                                                            let _ = server.register_mock_for_model(&name_c).await;
                                                            let _ = txc.send(format!("Provider failed; registered mock: {}", e));
                                                        }
                                                    }
                                                });
                                                self.logs.push(format!("Starting provider for {}", name));
                                            }
                                        }
                                    } else {
                                        self.logs.push("Failed to construct ModelManager".into());
                                    }
                                }
                            }
                        }
                        KeyCode::Char('/') => {
                            self.palette_open = true;
                            self.palette_query.clear();
                        }
                        KeyCode::Char(c) => {
                            if self.palette_open {
                                self.palette_query.push(c);
                            } else {
                                self.input.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if self.palette_open {
                                self.palette_query.pop();
                            } else {
                                self.input.pop();
                            }
                        }
                        KeyCode::Enter => {
                            if self.palette_open {
                                // simple action: if contains 'theme' open themes menu (cycle)
                                if self.palette_query.to_lowercase().contains("theme") || self.palette_query.to_lowercase().contains("themes") {
                                    // cycle theme index
                                    self.selected_theme = (self.selected_theme + 1) % self.theme_list.len();
                                    match self.config.write() {
                                        Ok(mut cfg) => {
                                            cfg.theme = self.theme_list[self.selected_theme].clone();
                                            if let Err(e) = cfg.save() {
                                                self.logs.push(format!("Failed save theme: {}", e));
                                            } else {
                                                self.logs.push(format!("Theme changed to {:?}", cfg.theme));
                                            }
                                        }
                                        Err(_) => { self.logs.push("Failed to acquire config lock".into()); }
                                    }
                                }
                                self.palette_open = false;
                                self.palette_query.clear();
                            } else if self.model_panel_open && self.expecting_import {
                                let p = self.input.trim().to_string();
                                if !p.is_empty() {
                                    let cfg = self.get_config_clone();
                                    if let Ok(mgr) = crate::models::ModelManager::new(Some(cfg.model_dir.clone())) {
                                        match mgr.import(std::path::Path::new(&p)) {
                                            Ok(mi) => {
                                                self.logs.push(format!("Imported model {}", mi.name));
                                                self.models_cached.push(format!("{} ({})", mi.name, mi.format));
                                            }
                                            Err(e) => { self.logs.push(format!("Import failed: {}", e)); }
                                        }
                                    }
                                }
                                self.expecting_import = false;
                                self.input.clear();
                            } else if self.native_model_panel_open && self.expecting_import {
                                // Import native model
                                let p = self.input.trim().to_string();
                                if !p.is_empty() {
                                    let cfg = self.get_config_clone();
                                    let txc = tx.clone();
                                    let path = std::path::Path::new(&p).to_path_buf();
                                    tokio::spawn(async move {
                                        match NativeModelManager::new(cfg.model_dir.clone()) {
                                            Ok(manager) => {
                                                match manager.add_model(path).await {
                                                    Ok(mi) => {
                                                        let _ = txc.send(format!("__native_models__::{} ({})", mi.name, mi.format.display_name()));
                                                        let _ = txc.send(format!("Imported native model: {}", mi.name));
                                                    }
                                                    Err(e) => {
                                                        let _ = txc.send(format!("__error__::Import failed: {}", e));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                let _ = txc.send(format!("__error__::Manager error: {}", e));
                                            }
                                        }
                                    });
                                    self.logs.push(format!("Importing: {}", p));
                                }
                                self.expecting_import = false;
                                self.input.clear();
                            } else {
                                let s = self.input.trim().to_string();
                                if !s.is_empty() {
                                    self.messages.push(format!("You: {}", s.clone()));
                                    self.logs.push(format!("Sent message: {}", s));
                                    self.input.clear();
                                }
                            }
                        }
                        KeyCode::Esc => {
                            if self.palette_open {
                                self.palette_open = false;
                                self.palette_query.clear();
                            } else if self.native_model_panel_open {
                                self.native_model_panel_open = false;
                                self.native_models_cached = vec![];
                                self.native_model_selected = 0;
                            } else {
                                // clear input
                                self.input.clear();
                            }
                        }
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // save config
                            match self.config.write() {
                                Ok(cfg) => {
                                    if let Err(e) = cfg.save() {
                                        self.logs.push(format!("Failed save config: {}", e));
                                    } else {
                                        self.logs.push("Config saved".into());
                                    }
                                }
                                Err(_) => { self.logs.push("Failed to acquire config lock".into()); }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
