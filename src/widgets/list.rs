//! List: a vertical list of selectable strings.

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::style::{Modifiers, Style};
use crate::widgets::Widget;

/// A vertical list of strings, with a moving cursor.
#[derive(Clone, Debug, Default)]
pub struct List {
    items: Vec<String>,
    cursor: usize,
    style: Style,
    highlight_style: Style,
}

impl List {
    pub fn new(items: impl IntoIterator<Item = impl Into<String>>) -> List {
        List {
            items: items.into_iter().map(Into::into).collect(),
            cursor: 0,
            style: Style::DEFAULT,
            highlight_style: Style::DEFAULT.mods(Modifiers::REVERSE),
        }
    }

    pub fn style(mut self, style: Style) -> List {
        self.style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> List {
        self.highlight_style = style;
        self
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = impl Into<String>>) {
        self.items = items.into_iter().map(Into::into).collect();
        if let Some(last) = self.items.len().checked_sub(1) {
            self.cursor = self.cursor.min(last);
        }
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        if let Some(last) = self.items.len().checked_sub(1) {
            self.cursor = cursor.min(last);
        }
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn selected(&self) -> Option<&str> {
        self.items.get(self.cursor).map(String::as_str)
    }

    fn cursor_up(&mut self) {
        if !self.items.is_empty() {
            self.cursor = self.cursor.saturating_sub(1);
        }
    }

    fn cursor_down(&mut self) {
        if let Some(last) = self.items.len().checked_sub(1) {
            self.cursor = (self.cursor + 1).min(last);
        }
    }
}

impl Widget for List {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(self.items.len() as u16)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let h = area.height as usize;
        if h == 0 {
            return;
        }
        let cursor = self.cursor.min(self.items.len().saturating_sub(1));
        let start = if cursor >= h { cursor + 1 - h } else { 0 };
        for (i, item) in self.items.iter().skip(start).take(h).enumerate() {
            let y = area.y + i as u16;
            let style = if start + i == cursor {
                self.highlight_style
            } else {
                self.style
            };
            grid.set_str(area.x, y, item, style);
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                self.cursor_up();
                true
            }
            KeyCode::Down => {
                self.cursor_down();
                true
            }
            KeyCode::Home => {
                if !self.items.is_empty() {
                    self.cursor = 0;
                }
                true
            }
            KeyCode::End => {
                self.set_cursor(usize::MAX);
                true
            }
            _ => false,
        }
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn on_mouse(&mut self, event: MouseEvent, area: Rect) -> bool {
        match event.kind {
            MouseEventKind::ScrollUp => {
                self.cursor_up();
                true
            }
            MouseEventKind::ScrollDown => {
                self.cursor_down();
                true
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // Map the clicked row (area-relative, 0-based) to an item,
                // accounting for the visible scroll window.
                let h = area.height as usize;
                if h == 0 {
                    return true;
                }
                let cursor = self.cursor.min(self.items.len().saturating_sub(1));
                let start = if cursor >= h { cursor + 1 - h } else { 0 };
                let row = event.row.saturating_sub(area.y) as usize;
                if row < h {
                    let idx = start + row;
                    if idx < self.items.len() {
                        self.cursor = idx;
                    }
                }
                true
            }
            _ => false,
        }
    }
}
