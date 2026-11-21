use super::centered_popup_rect;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyHandlingResult {
    Close,
    Continue,
}

pub struct HelpItem {
    pub key: String,
    pub description: String,
}

pub struct HelpDialog {
    items: Vec<HelpItem>,
}

impl HelpDialog {
    pub fn new(items: Vec<HelpItem>) -> Self {
        Self { items }
    }

    pub fn handle_key(&self, key: KeyEvent) -> KeyHandlingResult {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => KeyHandlingResult::Close,
            _ => KeyHandlingResult::Continue,
        }
    }

    pub fn draw(&self, f: &mut Frame<'_>) {
        let area = f.area();
        let key_width = self
            .items
            .iter()
            .map(|item| item.key.len())
            .max()
            .unwrap_or(0);
        let content_width = self
            .items
            .iter()
            .map(|item| key_width + 2 + item.description.len())
            .max()
            .unwrap_or(0);
        let width = (content_width as u16).saturating_add(2).min(area.width);
        let height = (self.items.len() as u16).saturating_add(2).min(area.height);
        let popup_rect = centered_popup_rect(area, width, height);
        if popup_rect.is_empty() {
            return;
        }
        let block = Block::default().borders(Borders::ALL).title(" Help ");
        let inner = block.inner(popup_rect);
        f.render_widget(Clear, inner);
        f.render_widget(block, popup_rect);
        let lines: Vec<Line<'_>> = self
            .items
            .iter()
            .map(|item| {
                Line::from(vec![
                    Span::styled(
                        format!("{:<width$}", item.key, width = key_width),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw("  "),
                    Span::raw(item.description.clone()),
                ])
            })
            .collect();
        f.render_widget(Paragraph::new(lines), inner);
    }
}
