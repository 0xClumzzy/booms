//! Markdown text widget: glow-style rendering of a markdown document
//! into the grid, wrapping to the widget width.

use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::markdown::{paint_md_line, render_markdown};
use crate::style::Color;
use crate::widgets::Widget;

/// A widget that renders markdown source with glow-style styling.
///
/// ```
/// use booms::style::Color;
/// use booms::widgets::MarkdownText;
///
/// let md = MarkdownText::new("# Title\n\nSome **bold** text.", Color::Ansi(14));
/// ```
pub struct MarkdownText {
    /// The markdown source.
    source: String,
    /// The color for list bullets and numbers.
    role_color: Color,
}

impl MarkdownText {
    /// Create a markdown widget from `text`, tinting list markers with
    /// `role_color`.
    pub fn new(text: impl Into<String>, role_color: Color) -> MarkdownText {
        MarkdownText {
            source: text.into(),
            role_color,
        }
    }

    /// Replace the markdown source.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.source = text.into();
    }

    /// The markdown source.
    pub fn text(&self) -> &str {
        &self.source
    }

    /// Set the list-marker color.
    pub fn role_color(mut self, color: Color) -> MarkdownText {
        self.role_color = color;
        self
    }

    /// Render the markdown into styled lines of `width` columns.
    pub fn lines(&self, width: usize) -> Vec<crate::markdown::MdLine> {
        render_markdown(&self.source, width.max(1), self.role_color)
    }
}

impl Widget for MarkdownText {
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let lines = self.lines(area.width as usize);
        for (i, line) in lines.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.bottom() {
                break;
            }
            paint_md_line(grid, area.x, y, line);
        }
    }
}
