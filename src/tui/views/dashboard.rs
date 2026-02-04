use crate::tui::theme::AppTheme;
use ratatui::{backend::Backend, layout::{Constraint, Direction, Layout, Rect}, style::{Style, Modifier}, text::{Span, Spans}, widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}, Frame};

#[derive(Default)]
pub struct DashboardView {
    highlights: Vec<String>,
}

impl DashboardView {
    pub fn tick(&mut self) {
        if self.highlights.len() > 6 {
            self.highlights.truncate(6);
        }
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, theme: &AppTheme) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let hero = Paragraph::new(vec![
            Spans::from(vec![
                Span::styled("SuperAgentCLI", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw(" — Mission Control for multi-agent workflows"),
            ]),
            Spans::from(Span::raw("")),
            Spans::from(Span::raw("• Orchestrate planner/executor/critic pipelines")),
            Spans::from(Span::raw("• Manage models (llama.cpp, Ollama, LM Studio)")),
            Spans::from(Span::raw("• Track tasks, subtasks, micro-agents")),
        ])
        .block(Block::default().borders(Borders::ALL).title("Overview"))
        .wrap(Wrap { trim: true });
        f.render_widget(hero, layout[0]);

        let items: Vec<ListItem> = if self.highlights.is_empty() {
            vec![ListItem::new("No recent highlights. Run a goal to populate this feed.")]
        } else {
            self.highlights.iter().map(|h| ListItem::new(h.clone())).collect()
        };
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Highlights"));
        f.render_widget(list, layout[1]);
    }
}
