use crate::tui::theme::AppTheme;
use ratatui::{backend::Backend, layout::{Constraint, Direction, Layout, Rect}, style::{Modifier, Style}, widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}, Frame};

#[derive(Default)]
pub struct TasksView {
    queues: Vec<String>,
    active: Vec<String>,
}

impl TasksView {
    pub fn tick(&mut self) {
        if self.queues.is_empty() {
            self.queues = vec![
                "Backlog • 12".into(),
                "Today • 4".into(),
                "Waiting • 2".into(),
            ];
        }
        if self.active.is_empty() {
            self.active = vec![
                "Design agent workflow".into(),
                "Implement vector memory".into(),
                "Model registry sync".into(),
            ];
        }
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect, theme: &AppTheme) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        let queues: Vec<ListItem> = self.queues.iter().map(|q| ListItem::new(q.clone())).collect();
        let queue_list = List::new(queues)
            .block(Block::default().borders(Borders::ALL).title("Queues"))
            .highlight_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));
        f.render_widget(queue_list, layout[0]);

        let active: Vec<ListItem> = self.active.iter().map(|t| ListItem::new(t.clone())).collect();
        let list = List::new(active)
            .block(Block::default().borders(Borders::ALL).title("Active Tasks"));
        f.render_widget(list, layout[1]);

        let footer = Paragraph::new("Use this workspace to break down goals into tasks, assign to agents, and monitor progress.")
            .style(Style::default().fg(theme.muted_text))
            .wrap(Wrap { trim: true });
        let footer_area = Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(3),
            width: area.width,
            height: 3,
        };
        f.render_widget(footer, footer_area);
    }
}
