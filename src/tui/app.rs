use crate::config::{RuntimeConfig, ThemeName};
use crate::tui::theme::{AppTheme, ThemeCatalog};
use crate::tui::views::{agents::AgentsView, dashboard::DashboardView, models::ModelsView, settings::SettingsView, tasks::TasksView, ViewId};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::{Backend, CrosstermBackend}, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Span, Spans}, widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}, Frame, Terminal};
use std::io;
use std::time::{Duration, Instant};

const TICK_RATE_MS: u64 = 200;

pub struct TuiApp {
    pub config: RuntimeConfig,
    pub theme_catalog: ThemeCatalog,
    pub active_theme: AppTheme,
    pub view: ViewId,
    pub last_tick: Instant,
    pub input: String,
    pub logs: Vec<String>,
    pub notifications: Vec<String>,
    pub dashboard: DashboardView,
    pub agents: AgentsView,
    pub models: ModelsView,
    pub tasks: TasksView,
    pub settings: SettingsView,
}

impl TuiApp {
    pub fn new(config: RuntimeConfig) -> anyhow::Result<Self> {
        let theme_catalog = ThemeCatalog::default();
        let active_theme = theme_catalog.resolve(&config.theme);
        Ok(Self {
            config,
            theme_catalog,
            active_theme,
            view: ViewId::Dashboard,
            last_tick: Instant::now(),
            input: String::new(),
            logs: vec!["SuperAgentCLI ready".into()],
            notifications: vec![],
            dashboard: DashboardView::default(),
            agents: AgentsView::default(),
            models: ModelsView::default(),
            tasks: TasksView::default(),
            settings: SettingsView::default(),
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let stdout = io::stdout();
        let _raw = RawModeGuard::enable()?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        loop {
            self.draw(&mut terminal)?;
            let timeout = Duration::from_millis(TICK_RATE_MS);
            if crossterm::event::poll(timeout)? {
                if let CEvent::Key(key) = event::read()? {
                    if self.handle_key(key)? {
                        break;
                    }
                }
            }
            if self.last_tick.elapsed() >= Duration::from_millis(TICK_RATE_MS) {
                self.tick();
                self.last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn draw(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
        terminal.draw(|f| {
            let size = f.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ])
                .split(size);

            self.render_header(f, layout[0]);
            self.render_body(f, layout[1]);
            self.render_footer(f, layout[2]);
        })?;
        Ok(())
    }

    fn render_header<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let title = format!(" SuperAgentCLI • {} ", self.view.title());
        let subtitle = format!(
            "Mode: {} | Theme: {}",
            self.view.name(),
            self.active_theme.name
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(title, self.active_theme.title_style))
            .title_alignment(ratatui::layout::Alignment::Left);
        let paragraph = Paragraph::new(Spans::from(vec![
            Span::styled(subtitle, Style::default().fg(self.active_theme.muted_text)),
        ]))
        .block(block)
        .alignment(ratatui::layout::Alignment::Left);
        f.render_widget(paragraph, area);
    }

    fn render_body<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(area);

        self.render_nav(f, sections[0]);
        self.render_active_view(f, sections[1]);
        self.render_sidebar(f, sections[2]);
    }

    fn render_nav<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let items: Vec<ListItem> = ViewId::all().iter().map(|view| {
            let label = if *view == self.view {
                format!("▶ {}", view.title())
            } else {
                format!("  {}", view.title())
            };
            ListItem::new(label)
        }).collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Workspace"))
            .highlight_style(Style::default().fg(self.active_theme.accent).add_modifier(Modifier::BOLD));
        f.render_widget(list, area);
    }

    fn render_active_view<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        match self.view {
            ViewId::Dashboard => self.dashboard.render(f, area, &self.active_theme),
            ViewId::Agents => self.agents.render(f, area, &self.active_theme),
            ViewId::Models => self.models.render(f, area, &self.active_theme),
            ViewId::Tasks => self.tasks.render(f, area, &self.active_theme),
            ViewId::Settings => self.settings.render(f, area, &self.active_theme, &self.config),
        }
    }

    fn render_sidebar<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        let logs: Vec<ListItem> = self
            .logs
            .iter()
            .rev()
            .take(8)
            .map(|line| ListItem::new(line.clone()))
            .collect();
        let log_list = List::new(logs)
            .block(Block::default().borders(Borders::ALL).title("Activity"));
        f.render_widget(log_list, sections[0]);

        let notes = Paragraph::new(self.notifications.join("\n"))
            .block(Block::default().borders(Borders::ALL).title("Notifications"))
            .style(Style::default().fg(self.active_theme.muted_text))
            .wrap(Wrap { trim: true });
        f.render_widget(notes, sections[1]);
    }

    fn render_footer<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let hint = "[Tab] Switch view  [/] Command  [Ctrl+S] Save  [Q] Quit";
        let input = Paragraph::new(self.input.as_str())
            .block(Block::default().borders(Borders::ALL).title("Command"))
            .style(Style::default().fg(self.active_theme.text));
        let overlay = Paragraph::new(Span::styled(hint, Style::default().fg(Color::Gray)))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(input, area);
        f.render_widget(overlay, area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
        match key.code {
            KeyCode::Char('q') if key.modifiers == KeyModifiers::NONE => return Ok(true),
            KeyCode::Tab => {
                self.view = self.view.next();
                self.logs.push(format!("Switched to {}", self.view.title()));
            }
            KeyCode::Char('/') => {
                self.input.clear();
                self.logs.push("Command palette opened".into());
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.config.save()?;
                self.logs.push("Configuration saved".into());
            }
            KeyCode::Enter => {
                if !self.input.trim().is_empty() {
                    self.logs.push(format!("Command: {}", self.input.trim()));
                    self.input.clear();
                }
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            _ => {}
        }
        Ok(false)
    }

    fn tick(&mut self) {
        self.dashboard.tick();
        self.agents.tick();
        self.models.tick();
        self.tasks.tick();
    }

    pub fn set_theme(&mut self, name: ThemeName) {
        self.config.theme = name;
        self.active_theme = self.theme_catalog.resolve(&self.config.theme);
    }
}

struct RawModeGuard {
    enabled: bool,
}

impl RawModeGuard {
    fn enable() -> anyhow::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(Self { enabled: true })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.enabled {
            let _ = crossterm::terminal::disable_raw_mode();
        }
    }
}
