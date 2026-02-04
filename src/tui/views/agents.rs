use crate::tui::theme::AppTheme;
use ratatui::{backend::Backend, layout::{Constraint, Direction, Layout, Rect}, style::Style, widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}, Frame};

#[derive(Default)]
pub struct AgentsView {
    agents: Vec<String>,
}

impl AgentsView {
    pub fn tick(&mut self) {
        if self.agents.is_empty() {
            self.agents = vec![
                "Planner • Strategy".into(),
                "Executor • Tool runner".into(),
                "Critic • QA review".into(),
                "MicroAgent • Summarizer".into(),
            ];
        }
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect, theme: &AppTheme) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(5)])
            .split(area);

        let intro = Paragraph::new("Define specialized agents, prompts, routing, and memory strategies.")
            .block(Block::default().borders(Borders::ALL).title("Agent Studio"))
            .style(Style::default().fg(theme.text))
            .wrap(Wrap { trim: true });
        f.render_widget(intro, layout[0]);

        let items: Vec<ListItem> = self.agents.iter().map(|a| ListItem::new(a.clone())).collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Registered Agents"));
        f.render_widget(list, layout[1]);
    }
}
