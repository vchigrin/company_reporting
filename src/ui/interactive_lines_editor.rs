use crate::model::{GenericKeys, ParsedLineInfo};
use crossterm::event::KeyCode;
use eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use super::line_edit_dialog::{EditDecision, LineEditDialog};

pub struct InteractiveLinesEditor<Keys: GenericKeys> {
    parsed_lines: Vec<ParsedLineInfo<Keys>>,
    selected_index: usize,
    list_state: ListState,
    mode: Mode<Keys>,
    status: Option<String>,
}

enum Mode<Keys: GenericKeys> {
    Navigate,
    Editing(Box<LineEditDialog<Keys>>),
}

#[derive(PartialEq)]
pub enum EditResult {
    SaveAndClose,
    CloseWithoutSaving,
}

impl<Keys: GenericKeys> InteractiveLinesEditor<Keys> {
    pub fn new(parsed_lines: Vec<ParsedLineInfo<Keys>>) -> Self {
        Self {
            parsed_lines,
            selected_index: 0,
            list_state: ListState::default(),
            mode: Mode::Navigate,
            status: None,
        }
    }

    pub fn result_lines(&self) -> &Vec<ParsedLineInfo<Keys>> {
        &self.parsed_lines
    }

    pub fn run_editor(&mut self) -> Result<EditResult> {
        let mut terminal = ratatui::init();
        let result = self.main_loop(&mut terminal);
        ratatui::restore();
        result
    }

    pub fn set_error_line(&mut self, error: String) {
        self.status = Some(error);
    }

    fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<EditResult> {
        loop {
            terminal.draw(|f| self.draw(f))?;
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match self.handle_key(key)? {
                    None => {}
                    Some(edit_result) => return Ok(edit_result),
                }
            }
        }
    }

    // If returns Some, then returned index is guaranted to be valid in
    // |parsed_lines|.
    fn selected(&self) -> Option<usize> {
        if self.parsed_lines.is_empty() {
            None
        } else {
            Some(self.selected_index.min(self.parsed_lines.len() - 1))
        }
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<EditResult>> {
        // Take from mode, to avoid borrowing problems of editor.
        // We'll replace editor in |mode| with the same editor
        // if we decide that editing must continue.
        let prev = std::mem::replace(&mut self.mode, Mode::Navigate);
        match prev {
            Mode::Navigate => self.handle_navigation_key(key),
            Mode::Editing(editor) => {
                self.handle_editing_key(key, editor)?;
                Ok(None)
            }
        }
    }

    fn handle_editing_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        mut editor: Box<LineEditDialog<Keys>>,
    ) -> Result<()> {
        let decision = editor.handle_key(key)?;
        match decision {
            EditDecision::Continue => {
                self.mode = Mode::Editing(editor);
            }
            EditDecision::Cancel => {
                self.mode = Mode::Navigate;
            }
            EditDecision::CommitOk(line) => {
                if editor.is_insert() {
                    let idx = self.selected().unwrap_or(self.parsed_lines.len());
                    self.parsed_lines.insert(idx, line);
                } else if let Some(i) = self.selected() {
                    self.parsed_lines[i] = line;
                } else {
                    panic!("Should not be reached; selected indec changed");
                }
                self.mode = Mode::Navigate;
                self.status = Some("Edits applied".to_owned());
            }
        }
        Ok(())
    }

    fn handle_navigation_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<Option<EditResult>> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(Some(EditResult::CloseWithoutSaving)),
            KeyCode::Enter => Ok(Some(EditResult::SaveAndClose)),
            KeyCode::Up => {
                let lines_count = self.parsed_lines.len();
                if lines_count > 0 {
                    self.selected_index = (self.selected_index + lines_count - 1) % lines_count;
                }
                Ok(None)
            }
            KeyCode::Down => {
                let lines_count = self.parsed_lines.len();
                if lines_count > 0 {
                    self.selected_index = (self.selected_index + 1) % lines_count;
                }
                Ok(None)
            }
            KeyCode::Char('i') => {
                self.mode = Mode::Editing(Box::new(LineEditDialog::new_insert()));
                self.status = None;
                Ok(None)
            }
            KeyCode::Char('e') => {
                if let Some(i) = self.selected() {
                    let line = self.parsed_lines[i].clone();
                    self.mode = Mode::Editing(Box::new(LineEditDialog::new_edit(line)));
                    self.status = None;
                }
                Ok(None)
            }
            KeyCode::Char('d') => {
                if let Some(i) = self.selected() {
                    self.parsed_lines.remove(i);
                    let len = self.parsed_lines.len();
                    self.selected_index = if len == 0 { 0 } else { i.min(len - 1) };
                    self.status = Some("Line deleted".to_owned());
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>) {
        self.list_state.select(self.selected());
        let list = self.list_widget();
        f.render_stateful_widget(list, f.area(), &mut self.list_state);
        if let Mode::Editing(editor) = &mut self.mode {
            editor.draw_editor_popup(f);
        } else {
            self.draw_status(f);
        }
    }

    fn list_widget(&self) -> List<'static> {
        let items: Vec<ListItem> = self
            .parsed_lines
            .iter()
            .enumerate()
            .map(|(i, l)| ListItem::new(self.render_line(i, l)))
            .collect();
        List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Interactive Lines Editor ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ")
    }

    fn render_line<K: GenericKeys>(&self, index: usize, line: &ParsedLineInfo<K>) -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("{:>3} ", index), Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:<25}", format!("{:?}", line.key)),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:>16} ", line.value.to_string()),
                Style::default().fg(Color::Red),
            ),
            Span::raw(line.original_line.clone()),
        ])
    }

    fn draw_status(&self, f: &mut Frame<'_>) {
        let status = self.status.clone().unwrap_or_else(|| {
            "i: insert  e: edit  d: delete  q/Esc: close without saving Enter: save and exit"
                .to_owned()
        });
        f.render_widget(
            Paragraph::new(status).style(Style::default().fg(Color::Yellow)),
            Rect::new(
                f.area().x,
                f.area().height.saturating_sub(1),
                f.area().width,
                1,
            ),
        );
    }
}
