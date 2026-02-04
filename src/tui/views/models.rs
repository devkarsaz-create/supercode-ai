use crate::tui::theme::AppTheme;
use ratatui::{backend::Backend, layout::{Constraint, Direction, Layout, Rect}, style::Style, widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}, Frame};

#[derive(Default)]
pub struct ModelsView {
    providers: Vec<String>,
    models: Vec<String>,
}

impl ModelsView {
    pub fn tick(&mut self) {
        if self.providers.is_empty() {
            self.providers = vec![
                "llama.cpp • localhost".into(),
                "Ollama • local".into(),
                "LM Studio • local".into(),
            ];
        }
        if self.models.is_empty() {
            self.models = vec![
                "local.gguf • offline".into(),
                "mixtral-8x7b • remote".into(),
                "codellama-13b • staged".into(),
            ];
        }
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect, theme: &AppTheme) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(layout[0]);

        let providers: Vec<ListItem> = self.providers.iter().map(|p| ListItem::new(p.clone())).collect();
        let provider_list = List::new(providers)
            .block(Block::default().borders(Borders::ALL).title("Providers"));
        f.render_widget(provider_list, top[0]);

        let summary = Paragraph::new("Configure endpoints, credentials, and connection health checks.")
            .block(Block::default().borders(Borders::ALL).title("Connection"))
            .style(Style::default().fg(theme.text))
            .wrap(Wrap { trim: true });
        f.render_widget(summary, top[1]);

        let models: Vec<ListItem> = self.models.iter().map(|m| ListItem::new(m.clone())).collect();
        let model_list = List::new(models)
            .block(Block::default().borders(Borders::ALL).title("Model Catalog"));
        f.render_widget(model_list, layout[1]);
    }
}
