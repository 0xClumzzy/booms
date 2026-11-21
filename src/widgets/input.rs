//! Input: a single-line text field.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::style::{Modifiers, Style};
use crate::widgets::Widget;

/// A single-line text input.
#[derive(Clone, Debug, Default)]
pub struct Input {
    buffer: Vec<char>,
    cursor: usize,
    focused: bool,
    style: Style,
    placeholder: String,
}

impl Input {
    pub fn new() -> Input {
        Input::default()
    }

    /// An input pre-filled with `value`; the cursor rests at the end.
    pub fn with_value(value: impl Into<String>) -> Input {
        let buffer: Vec<char> = value.into().chars().collect();
        Input {
            cursor: buffer.len(),
            buffer,
            ..Input::default()
        }
    }

    pub fn style(mut self, style: Style) -> Input {
        self.style = style;
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Input {
        self.placeholder = text.into();
        self
    }

    /// The current text.
    pub fn value(&self) -> String {
        self.buffer.iter().collect()
    }

    /// Remove all text and reset the cursor, keeping the placeholder and
    /// style. Useful for clearing a prompt after submission.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    /// The first column currently on screen, keeping the cursor visible.
    fn visible_start(&self, area: Rect) -> usize {
        let w = area.width as usize;
        if w == 0 {
            0
        } else {
            self.cursor.saturating_sub(w.saturating_sub(1))
        }
    }

    fn insert(&mut self, c: char) {
        let idx = self.cursor.min(self.buffer.len());
        self.buffer.insert(idx, c);
        self.cursor = idx + 1;
    }

    fn delete_back(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
        }
    }

    fn delete_forward(&mut self) {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
        }
    }
}

impl Widget for Input {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let w = area.width as usize;
        let start = self.visible_start(area);
        let visible: String = self.buffer[start..].iter().take(w).collect();
        let end = grid.set_str(area.x, area.y, &visible, self.style);
        // Clear the rest of the line.
        for x in end..area.right().min(grid.width()) {
            grid.set(x, area.y, ' ', self.style);
        }
        if self.buffer.is_empty() && !self.placeholder.is_empty() {
            let placeholder = self.style.mods(Modifiers::DIM);
            grid.set_str(area.x, area.y, &self.placeholder, placeholder);
        }
    }

    fn cursor_pos(&self, area: Rect) -> Option<(u16, u16)> {
        if !self.focused || area.width == 0 {
            return None;
        }
        let start = self.visible_start(area);
        Some((area.x + (self.cursor - start) as u16, area.y))
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Click to place the cursor at the clicked column.
    fn on_mouse(&mut self, event: MouseEvent, area: Rect) -> bool {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let start = self.visible_start(area);
                let offset = (event.column.saturating_sub(area.x)) as usize;
                self.cursor = (start + offset).min(self.buffer.len());
                true
            }
            _ => false,
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c)
                if c != '\r'
                    && c != '\n'
                    && c != '\t'
                    && !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.insert(c);
                true
            }
            KeyCode::Backspace => {
                self.delete_back();
                true
            }
            KeyCode::Delete => {
                self.delete_forward();
                true
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
                true
            }
            KeyCode::Right => {
                self.cursor = (self.cursor + 1).min(self.buffer.len());
                true
            }
            KeyCode::Home => {
                self.cursor = 0;
                true
            }
            KeyCode::End => {
                self.cursor = self.buffer.len();
                true
            }
            _ => false,
        }
    }
}
