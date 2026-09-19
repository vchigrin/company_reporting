use super::keys_combo_box::KeysComboBox;
use crate::model::{GenericKeys, Money, ParsedLineInfo};
use crossterm::event::KeyCode;
use eyre::{Result, eyre};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Clear},
};
use ratatui_textarea::TextArea;
use strum::VariantArray;
use strum_macros::VariantArray;

#[derive(Clone, Copy, PartialEq, VariantArray)]
enum Field {
    Key,
    Value,
    Raw,
}

pub enum EditDecision<Keys: GenericKeys> {
    Continue,
    Cancel,
    CommitOk(ParsedLineInfo<Keys>),
}

impl Field {
    fn label(self) -> &'static str {
        match self {
            Field::Value => "Value (roubles)",
            Field::Key => "Key",
            Field::Raw => "Raw line",
        }
    }

    fn max_label_len() -> usize {
        let mut result = 0;
        for f in Self::VARIANTS {
            result = result.max(f.label().len());
        }
        result
    }

    fn next(self) -> Field {
        let idx = Self::VARIANTS.iter().position(|f| *f == self).unwrap() + 1;
        Self::VARIANTS[idx % Self::VARIANTS.len()]
    }

    fn prev(self) -> Field {
        let len = Self::VARIANTS.len();
        let idx = Self::VARIANTS.iter().position(|f| *f == self).unwrap();
        Self::VARIANTS[(idx + len - 1) % len]
    }
}

pub struct LineEditDialog<Keys: GenericKeys> {
    value: TextArea<'static>,
    key: KeysComboBox<Keys>,
    raw: TextArea<'static>,
    active: Field,
    is_insert: bool,
}

impl<Keys: GenericKeys> LineEditDialog<Keys> {
    pub fn new_insert() -> Self {
        Self {
            value: TextArea::new(vec!["0".to_owned()]),
            key: KeysComboBox::new(),
            raw: TextArea::default(),
            active: Field::Key,
            is_insert: true,
        }
    }

    pub fn new_edit(line: ParsedLineInfo<Keys>) -> Self {
        Self {
            value: TextArea::new(vec![line.value.in_roubles().to_string()]),
            key: KeysComboBox::new_with_selection(line.key),
            raw: TextArea::new(vec![line.original_line]),
            active: Field::Key,
            is_insert: false,
        }
    }

    pub fn is_insert(&self) -> bool {
        self.is_insert
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Result<EditDecision<Keys>> {
        match key.code {
            KeyCode::Esc => Ok(EditDecision::<Keys>::Cancel),
            KeyCode::Enter => {
                // HACK: First enter must be handled by combobox.
                if self.active == Field::Key && self.key.is_open() {
                    self.key.handle_key(key);
                    Ok(EditDecision::<Keys>::Continue)
                } else {
                    match self.build_line() {
                        Ok(line) => Ok(EditDecision::<Keys>::CommitOk(line)),
                        // Error is reported during rendering, as bottom
                        // title.
                        Err(_) => Ok(EditDecision::<Keys>::Continue),
                    }
                }
            }
            KeyCode::Tab => {
                self.active = self.active.next();
                Ok(EditDecision::<Keys>::Continue)
            }
            KeyCode::BackTab => {
                self.active = self.active.prev();
                Ok(EditDecision::<Keys>::Continue)
            }
            _ => {
                self.handle_keys_by_children(key);
                Ok(EditDecision::<Keys>::Continue)
            }
        }
    }

    fn handle_keys_by_children(&mut self, key: crossterm::event::KeyEvent) {
        match self.active {
            Field::Value => {
                if let KeyCode::Char(c) = &key.code {
                    let is_sign = *c == '+' || *c == '-';
                    if !(c.is_ascii_digit() || is_sign) {
                        // Value can contain only numbers.
                        return;
                    }
                }
                self.value.input(key);
            }
            Field::Key => {
                self.key.handle_key(key);
            }
            Field::Raw => {
                self.raw.input(key);
            }
        };
    }

    fn field_trimmed_string(area: &TextArea<'static>) -> String {
        let lines = area.lines();
        if lines.len() != 1 {
            // Should never hapen, since we don't pass Enter keypresses
            // to textboxes.
            panic!("Exactly one line expected");
        }
        lines[0].trim().to_owned()
    }

    /// Build a ParsedLineInfo from the editor contents, or Err(err) if the
    /// value or key do not parse.
    fn build_line(&self) -> Result<ParsedLineInfo<Keys>> {
        let value_str = Self::field_trimmed_string(&self.value);
        let roubles = value_str
            .parse::<i64>()
            .map_err(|_| eyre!("Failed to parse value {}", value_str))?;
        let key = self.key.selected_item();
        let original_line = Self::field_trimmed_string(&self.raw);
        Ok(ParsedLineInfo {
            key,
            value: Money::from_roubles(roubles),
            original_line,
        })
    }

    pub fn draw_editor_popup(&mut self, f: &mut Frame<'_>) {
        let area = Self::centered_popup_rect(f.area(), 60, 5);
        if area.is_empty() {
            return;
        }
        let title = if self.is_insert {
            " Insert line "
        } else {
            " Edit line "
        };
        let mut popup = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Magenta));
        if let Err(e) = self.build_line() {
            popup = popup
                .title_bottom(e.to_string())
                .border_style(Style::default().fg(Color::Red));
        }
        let inner = popup.inner(area);
        f.render_widget(popup, area);
        f.render_widget(Clear, inner);

        let column_rects = Layout::horizontal([
            // One column for ':' and one - for separator space.
            Constraint::Length((Field::max_label_len() + 2) as u16),
            Constraint::Fill(1),
        ])
        .split(inner);

        let mut row_contraints = Vec::new();
        for _ in 0..Field::VARIANTS.len() {
            row_contraints.push(Constraint::Length(1));
        }
        let rows_layout = Layout::vertical(&row_contraints);
        let label_rects = rows_layout.split(column_rects[0]);
        let textarea_rects = rows_layout.split(column_rects[1]);

        self.render_field_row(f, label_rects[2], textarea_rects[2], Field::Raw);
        self.render_field_row(f, label_rects[1], textarea_rects[1], Field::Value);
        // Draw key last, sunce it pop-up may overlap other widgets.
        self.render_field_row(f, label_rects[0], textarea_rects[0], Field::Key);
    }

    fn render_field_row(
        &mut self,
        f: &mut Frame<'_>,
        label_rect: Rect,
        textarea_rect: Rect,
        field: Field,
    ) {
        let active = self.active == field;
        let style = Style::default().fg(if active { Color::Yellow } else { Color::White });
        let label = Span::styled(format!("{}:", field.label()), style);

        f.render_widget(label, label_rect);
        match field {
            Field::Value => {
                self.value.set_style(style);
                f.render_widget(&self.value, textarea_rect);
            }
            Field::Key => {
                self.key.set_style(style);
                f.render_widget(&self.key, textarea_rect);
            }
            Field::Raw => {
                self.raw.set_style(style);
                f.render_widget(&self.raw, textarea_rect);
            }
        }
    }

    fn centered_popup_rect(area: Rect, h: u16, v: u16) -> Rect {
        let v_padding = area.height.saturating_sub(v) / 2;
        let h_padding = area.width.saturating_sub(h) / 2;
        if v_padding == 0 || h_padding == 0 {
            return Rect::default();
        }
        Rect::new(area.x + h_padding, area.y + v_padding, h, v)
    }
}
