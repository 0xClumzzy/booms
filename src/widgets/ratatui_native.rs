//! Native ratatui support: render real [`ratatui`] widgets inside booms.
//!
//! The [`RatatuiView`] adapter wraps any [`ratatui::widgets::Widget`] (must be
//! [`Clone`]) and paints it into booms' [`Grid`](crate::grid::Grid) every
//! frame, translating ratatui colors, modifiers and buffer cells into booms'
//! styling. Because the wrapped widget is cloned and re-rendered each tick, it
//! can produce dynamic output (clocks, animations, live data, ...).
//!
//! Most built-in ratatui widgets: [`ratatui::widgets::Paragraph`],
//! [`ratatui::widgets::Block`], [`ratatui::widgets::Table`],
//! [`ratatui::widgets::Gauge`], [`ratatui::widgets::Sparkline`],
//! [`ratatui::widgets::BarChart`], [`ratatui::widgets::List`],
//! [`ratatui::widgets::Tabs`] are [`Clone`] and work out of the box.
//!
//! This module is only compiled with the `ratatui` feature flag, because
//! ratatui pulls a large dependency tree.
//!
//! ```rust,no_run
//! use booms::app::App;
//! use booms::widgets::{RatatuiView, Text};
//! use ratatui::widgets::{Block, Paragraph};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut app = App::new()?;
//!     app.add(Text::new(["native ratatui inside booms:"]));
//!     app.add(RatatuiView::new(
//!         Paragraph::new("Hello from ratatui!").block(Block::bordered()),
//!     ));
//!     app.run()?;
//!     Ok(())
//! }
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect as RatRect,
    style::{Color as RatColor, Modifier as RatModifier},
    widgets::Widget,
};

use crate::grid::{Grid, char_width};
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Modifiers, Style};
use crate::widgets::Widget as BoomsWidget;

/// A booms [`Widget`](crate::widgets::Widget) that renders a real ratatui
/// widget into the grid.
///
/// The wrapped ratatui widget is cloned and re-rendered every frame, so it can
/// produce dynamic output.
///
/// The widget's height is governed by booms' layout (its [`Constraint`]); it
/// fills the whole area booms assigns it.
pub struct RatatuiView<W> {
    widget: W,
}

impl<W> RatatuiView<W> {
    /// Wrap a ratatui widget for rendering inside booms.
    pub fn new(widget: W) -> Self {
        Self { widget }
    }
}

impl<W: Clone> Clone for RatatuiView<W> {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
        }
    }
}

impl<W: Widget + Clone> BoomsWidget for RatatuiView<W> {
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let mut buffer = Buffer::empty(RatRect::new(0, 0, area.width, area.height));
        self.widget.clone().render(buffer.area, &mut buffer);
        render_buffer_into_grid(&buffer, grid, area.x, area.y);
    }
}

/// Copy a ratatui [`Buffer`] into a booms [`Grid`] at the given offset.
///
/// Wide symbols (e.g. CJK characters) occupy multiple grid columns; the
/// cursor advances by the symbol's display width so continuation columns are
/// not re-painted.
fn render_buffer_into_grid(buffer: &Buffer, grid: &mut Grid, offset_x: u16, offset_y: u16) {
    let bw = buffer.area.width as usize;
    let bh = buffer.area.height as usize;
    if bw == 0 || bh == 0 {
        return;
    }
    let gw = grid.width();
    let gh = grid.height();
    for y in 0..bh {
        let gy = offset_y.saturating_add(y as u16);
        if gy >= gh {
            break;
        }
        let mut x = 0;
        while x < bw {
            let cell = buffer
                .cell((x as u16, y as u16))
                .expect("iterating within buffer bounds");
            let symbol = cell.symbol();
            let style = rat_style_to_booms(cell);
            let mut advanced = 0;
            let mut any = false;
            for ch in symbol.chars() {
                let w = char_width(ch);
                if w == 0 {
                    continue;
                }
                let gx = offset_x
                    .saturating_add(x as u16)
                    .saturating_add(advanced as u16);
                if gx >= gw {
                    break;
                }
                grid.set(gx, gy, ch, style);
                if w > 1 && (gx as usize) + 1 < gw as usize {
                    grid.set(gx + 1, gy, ' ', style);
                }
                advanced += w as usize;
                any = true;
            }
            if !any {
                x += 1;
            } else {
                x += advanced;
            }
        }
    }
}

/// Convert a ratatui buffer cell's style into a booms [`Style`].
fn rat_style_to_booms(cell: &ratatui::buffer::Cell) -> Style {
    let fg = rat_color_to_booms(cell.fg);
    let bg = rat_color_to_booms(cell.bg);
    let mut mods = Modifiers::NONE;
    let m = cell.modifier;
    if m.contains(RatModifier::BOLD) {
        mods = mods.union(Modifiers::BOLD);
    }
    if m.contains(RatModifier::DIM) {
        mods = mods.union(Modifiers::DIM);
    }
    if m.contains(RatModifier::ITALIC) {
        mods = mods.union(Modifiers::ITALIC);
    }
    if m.contains(RatModifier::UNDERLINED) {
        mods = mods.union(Modifiers::UNDERLINE);
    }
    if m.contains(RatModifier::SLOW_BLINK) || m.contains(RatModifier::RAPID_BLINK) {
        mods = mods.union(Modifiers::BLINK);
    }
    if m.contains(RatModifier::REVERSED) {
        mods = mods.union(Modifiers::REVERSE);
    }
    if m.contains(RatModifier::CROSSED_OUT) {
        mods = mods.union(Modifiers::STRIKETHROUGH);
    }
    Style::new().fg(fg).bg(bg).mods(mods)
}

/// Convert a ratatui [`Color`] into a booms [`Color`].
fn rat_color_to_booms(color: RatColor) -> Color {
    match color {
        RatColor::Reset => Color::Default,
        RatColor::Black => Color::Ansi(0),
        RatColor::Red => Color::Ansi(1),
        RatColor::Green => Color::Ansi(2),
        RatColor::Yellow => Color::Ansi(3),
        RatColor::Blue => Color::Ansi(4),
        RatColor::Magenta => Color::Ansi(5),
        RatColor::Cyan => Color::Ansi(6),
        RatColor::Gray => Color::Ansi(7),
        RatColor::DarkGray => Color::Ansi(8),
        RatColor::LightRed => Color::Ansi(9),
        RatColor::LightGreen => Color::Ansi(10),
        RatColor::LightYellow => Color::Ansi(11),
        RatColor::LightBlue => Color::Ansi(12),
        RatColor::LightMagenta => Color::Ansi(13),
        RatColor::LightCyan => Color::Ansi(14),
        RatColor::White => Color::Ansi(15),
        RatColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        RatColor::Indexed(n) => Color::Ansi256(n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Stylize;
    use ratatui::widgets::Paragraph;

    #[test]
    fn ratatui_view_renders_paragraph() {
        let widget = Paragraph::new("hello");
        let view = RatatuiView::new(widget);
        let mut grid = Grid::new(10, 3, crate::grid::Cell::blank());
        view.render(Rect::new(0, 0, 10, 3), &mut grid);
        assert_eq!(grid.get(0, 0).ch, 'h');
        assert_eq!(grid.get(4, 0).ch, 'o');
        // cells beyond the text are blank
        assert_eq!(grid.get(5, 0).ch, ' ');
    }

    #[test]
    fn ratatui_view_styles_bold_text() {
        let widget = Paragraph::new("bold").bold();
        let view = RatatuiView::new(widget);
        let mut grid = Grid::new(10, 1, crate::grid::Cell::blank());
        view.render(Rect::new(0, 0, 10, 1), &mut grid);
        assert!(grid.get(0, 0).style.mods.contains(Modifiers::BOLD));
    }

    #[test]
    fn ratatui_view_handles_wide_symbols() {
        let widget = Paragraph::new("あいう");
        let view = RatatuiView::new(widget);
        let mut grid = Grid::new(10, 1, crate::grid::Cell::blank());
        view.render(Rect::new(0, 0, 10, 1), &mut grid);
        // three wide chars, each 2 columns
        assert_eq!(grid.get(0, 0).ch, 'あ');
        assert_eq!(grid.get(2, 0).ch, 'い');
        assert_eq!(grid.get(4, 0).ch, 'う');
    }

    #[test]
    fn ratatui_view_is_cloneable() {
        let widget = Paragraph::new("x");
        let view = RatatuiView::new(widget);
        let _clone = view.clone();
    }

    #[test]
    fn rat_color_mapping() {
        assert_eq!(rat_color_to_booms(RatColor::Reset), Color::Default);
        assert_eq!(rat_color_to_booms(RatColor::Red), Color::Ansi(1));
        assert_eq!(
            rat_color_to_booms(RatColor::Rgb(1, 2, 3)),
            Color::Rgb(1, 2, 3)
        );
        assert_eq!(
            rat_color_to_booms(RatColor::Indexed(200)),
            Color::Ansi256(200)
        );
    }
}
