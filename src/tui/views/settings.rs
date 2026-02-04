use crate::config::{RuntimeConfig, ThemeName};
use crate::tui::theme::AppTheme;
use ratatui::{backend::Backend, layout::{Constraint, Direction, Layout, Rect}, style::{Modifier, Style}, text::{Span, Spans}, widgets::{Block, Borders, Paragraph, Wrap}, Frame};

#[derive(Default)]
pub struct SettingsView;

impl SettingsView {
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect, theme: &AppTheme, config: &RuntimeConfig) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(6),
                Constraint::Min(6),
            ])
            .split(area);

        let endpoints = Paragraph::new(vec![
            Spans::from(vec![Span::styled("LLM Endpoint: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&config.llm_endpoint)]),
            Spans::from(vec![Span::styled("Default Model: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&config.llm_model)]),
            Spans::from(vec![Span::styled("Model Dir: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(config.model_dir.display().to_string())]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Runtime"))
        .style(Style::default().fg(theme.text));
        f.render_widget(endpoints, layout[0]);

        let theme_info = Paragraph::new(vec![
            Spans::from(vec![Span::styled("Active Theme: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(theme.name)]),
            Spans::from(Span::raw("Use /theme <name> to switch themes.")),
            Spans::from(Span::raw("Available: DarkPlus, Light, Monokai, SolarizedDark, SolarizedLight")),
            Spans::from(Span::raw("Dracula, OneDark, Nord, Gruvbox, Peacocks")),
        ])
        .block(Block::default().borders(Borders::ALL).title("Appearance"))
        .style(Style::default().fg(theme.text))
        .wrap(Wrap { trim: true });
        f.render_widget(theme_info, layout[1]);

        let tips = Paragraph::new("Settings view is the control center for providers, storage, and governance policies. Configure global prompts, memory backends, and workspace sync here.")
            .block(Block::default().borders(Borders::ALL).title("Governance"))
            .style(Style::default().fg(theme.muted_text))
            .wrap(Wrap { trim: true });
        f.render_widget(tips, layout[2]);
    }
}

impl SettingsView {
    #[allow(dead_code)]
    pub fn parse_theme(input: &str) -> Option<ThemeName> {
        match input.to_lowercase().as_str() {
            "darkplus" => Some(ThemeName::DarkPlus),
            "light" => Some(ThemeName::Light),
            "monokai" => Some(ThemeName::Monokai),
            "solarizeddark" => Some(ThemeName::SolarizedDark),
            "solarizedlight" => Some(ThemeName::SolarizedLight),
            "dracula" => Some(ThemeName::Dracula),
            "onedark" => Some(ThemeName::OneDark),
            "nord" => Some(ThemeName::Nord),
            "gruvbox" => Some(ThemeName::Gruvbox),
            "peacocks" => Some(ThemeName::Peacocks),
            _ => None,
        }
    }
}
