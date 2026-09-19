use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Widget},
};
use std::cell::Cell;

use crate::model::GenericKeys;

pub struct KeysComboBox<Keys: GenericKeys> {
    input: String,
    filtered_options: Vec<Keys>,
    list_state: Cell<ListState>,
    is_open: bool,
    selected_item: Keys,
    style: Style,
}

impl<Keys: GenericKeys> KeysComboBox<Keys> {
    pub fn new() -> Self {
        // Start with the first option highlighted
        let first_key = Keys::VARIANTS
            .first()
            .expect("Keys must include at least one option");
        Self::new_with_selection(*first_key)
    }

    pub fn new_with_selection(selected: Keys) -> Self {
        let mut s = Self {
            input: String::new(),
            filtered_options: Vec::new(),
            list_state: Cell::new(ListState::default()),
            is_open: false,
            selected_item: selected,
            style: Style::default(),
        };
        s.filter_options();
        let idx = s
            .filtered_options
            .iter()
            .position(|x| *x == selected)
            .expect("VARIANTS must include all keys");
        s.list_state.get_mut().select(Some(idx));
        s
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn selected_item(&self) -> Keys {
        self.selected_item
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    fn handle_char(&mut self, c: char) {
        self.input.push(c);
        self.filter_options();
    }

    fn handle_backspace(&mut self) {
        self.input.pop();
        self.filter_options();
    }

    fn filter_options(&mut self) {
        let mut new_filtered = Vec::new();
        let input_lower = self.input.to_lowercase();
        for key in Keys::VARIANTS {
            let key_str: &str = (*key).into();
            if key_str.to_lowercase().contains(&input_lower) {
                new_filtered.push(*key);
            }
        }
        self.filtered_options = new_filtered;

        // Reset list indexing safely
        let mut new_state = self.list_state.get();
        if self.filtered_options.is_empty() {
            new_state.select(None);
        } else {
            new_state.select(Some(0));
        }
        self.list_state.set(new_state);
    }

    fn select_next(&mut self) {
        if self.filtered_options.is_empty() {
            return;
        }
        let mut new_state = self.list_state.get();
        let i = match new_state.selected() {
            Some(i) => (i + 1) % self.filtered_options.len(),
            None => 0,
        };
        new_state.select(Some(i));
        self.list_state.set(new_state);
    }

    fn select_previous(&mut self) {
        if self.filtered_options.is_empty() {
            return;
        }
        let mut new_state = self.list_state.get();
        let i = match new_state.selected() {
            Some(i) => (i + self.filtered_options.len() - 1) % self.filtered_options.len(),
            None => 0,
        };
        new_state.select(Some(i));
        self.list_state.set(new_state);
    }

    fn submit_selection(&mut self) {
        self.is_open = false;
        if let Some(i) = self.list_state.get().selected()
            && let Some(choise) = self.filtered_options.get(i)
        {
            self.input = String::new();
            self.selected_item = *choise;
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.is_open = false;
            }
            KeyCode::Enter => {
                self.submit_selection();
            }
            KeyCode::Down => {
                self.select_next();
            }
            KeyCode::Up => {
                self.select_previous();
            }
            KeyCode::Backspace => {
                self.handle_backspace();
            }
            KeyCode::Char(c) => {
                self.is_open = true;
                self.handle_char(c);
            }
            _ => {}
        }
    }
}

impl<Keys: GenericKeys> Widget for &KeysComboBox<Keys> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input_block = Block::default().borders(Borders::NONE);

        let selected_str: &str = self.selected_item.into();
        let paragraph = Paragraph::new(selected_str)
            .block(input_block)
            .style(if self.is_open {
                Style::default().fg(Color::Yellow)
            } else {
                self.style
            });

        paragraph.render(area, buf);

        if self.is_open {
            // Define dropdown geometry sitting directly underneath the input box
            let popup_area = Rect {
                x: area.x,
                y: area.y + area.height, // Appears right under the bar
                width: area.width,
                height: (self.filtered_options.len() as u16 + 2).min(8), // Limit height max layout
            };

            let mut list_items: Vec<ListItem> = Vec::with_capacity(self.filtered_options.len());
            for key in &self.filtered_options {
                let key_str: &str = (*key).into();
                list_items.push(ListItem::new(key_str));
            }

            let dropdown_list = List::new(list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" Items for: {} ", self.input)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            // Clear underneath cells to prevent base UI bleeding through the menu popup
            Clear.render(popup_area, buf);
            let mut new_state = self.list_state.get();
            StatefulWidget::render(dropdown_list, popup_area, buf, &mut new_state);
            self.list_state.set(new_state);
        }
    }
}

impl<Keys: GenericKeys> Default for KeysComboBox<Keys> {
    fn default() -> Self {
        Self::new()
    }
}
