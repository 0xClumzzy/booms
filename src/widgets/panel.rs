//! Panel: a bordered container whose title can wrap *around* the border
//! perimeter (across corners), plus squarish/rectangular block styling.
//!
//! This is booms' answer to ratatui's `Block`, with an extra trick: when a
//! title is longer than the edge it starts on, it keeps flowing around the
//! corners instead of being clipped.

use crate::grid::{Grid, char_width};
use crate::layout::{Constraint, Rect};
use crate::style::Style;
use crate::widgets::{Widget, wrap_text};

/// Border styles, mirroring ratatui's `BorderType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BorderType {
    /// `┌ ┐ └ ┘ ─ │`
    #[default]
    Plain,
    /// Rounded corners `╭ ╮ ╰ ╯`: the "circular" look.
    Rounded,
    /// Double lines `╔ ╗ ╚ ╝ ═ ║`.
    Double,
    /// Thick lines `┏ ┓ ┗ ┛ ━ ┃`.
    Thick,
}

impl BorderType {
    /// (top-left, top-right, bottom-left, bottom-right, horizontal, vertical)
    const fn glyphs(self) -> (char, char, char, char, char, char) {
        match self {
            BorderType::Plain => ('┌', '┐', '└', '┘', '─', '│'),
            BorderType::Rounded => ('╭', '╮', '╰', '╯', '─', '│'),
            BorderType::Double => ('╔', '╗', '╚', '╝', '═', '║'),
            BorderType::Thick => ('┏', '┓', '┗', '┛', '━', '┃'),
        }
    }
}

/// Which edge a panel title starts on.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TitlePosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

/// Title alignment along its starting edge.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TitleAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// A bordered panel with a wrap-around title and word-wrapped content.
///
/// ```
/// use booms::grid::Grid;
/// use booms::layout::Rect;
/// use booms::widgets::{BorderType, Panel, Widget};
///
/// let panel = Panel::new()
///     .title("a very long title that will wrap around the border corners")
///     .borders(BorderType::Rounded);
/// let mut grid = Grid::new(20, 6, booms::grid::Cell::blank());
/// panel.render(Rect::new(0, 0, 20, 6), &mut grid);
/// assert_eq!(grid.get(0, 0).ch, '╭');
/// ```
pub struct Panel {
    title: String,
    title_style: Style,
    border_type: BorderType,
    border_style: Style,
    title_position: TitlePosition,
    title_alignment: TitleAlignment,
    content: Vec<String>,
    content_style: Style,
    wrap_content: bool,
    padding: u16,
    /// Opacity of the whole panel (borders, title, content), `0..=255`.
    /// Values below `255` blend the panel over the canvas color, producing a
    /// frosted-glass / transparent look.
    opacity: u8,
}

impl Panel {
    pub fn new() -> Panel {
        Panel {
            title: String::new(),
            title_style: Style::DEFAULT,
            border_type: BorderType::Plain,
            border_style: Style::DEFAULT,
            title_position: TitlePosition::Top,
            title_alignment: TitleAlignment::Left,
            content: Vec::new(),
            content_style: Style::DEFAULT,
            wrap_content: true,
            // Content sits just inside the border by default. Callers can
            // opt into more breathing room with `.padding(1)`. A zero default
            // also keeps very short panels usable (padding would eat the
            // entire inner area).
            padding: 0,
            opacity: 255,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Panel {
        self.title = title.into();
        self
    }

    /// Set the opacity of the whole panel (borders, title, content),
    /// `0..=255`. Values below `255` make the panel semi-transparent by
    /// blending over the canvas background: a frosted-glass look.
    pub fn opacity(mut self, opacity: u8) -> Panel {
        self.opacity = opacity;
        self
    }

    pub fn title_style(mut self, style: Style) -> Panel {
        self.title_style = style;
        self
    }

    pub fn borders(mut self, border: BorderType) -> Panel {
        self.border_type = border;
        self
    }

    pub fn border_style(mut self, style: Style) -> Panel {
        self.border_style = style;
        self
    }

    pub fn title_position(mut self, position: TitlePosition) -> Panel {
        self.title_position = position;
        self
    }

    pub fn title_alignment(mut self, alignment: TitleAlignment) -> Panel {
        self.title_alignment = alignment;
        self
    }

    /// Lines of inner content. Each is word-wrapped to the panel's inner
    /// width when `wrap_content` is enabled.
    pub fn content<I, S>(mut self, content: I) -> Panel
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.content = content.into_iter().map(Into::into).collect();
        self
    }

    pub fn content_style(mut self, style: Style) -> Panel {
        self.content_style = style;
        self
    }

    pub fn wrap_content(mut self, wrap: bool) -> Panel {
        self.wrap_content = wrap;
        self
    }

    /// Blank columns/rows between the border and the content.
    pub fn padding(mut self, padding: u16) -> Panel {
        self.padding = padding;
        self
    }

    /// The area inside the border and padding.
    pub fn inner(&self, area: Rect) -> Rect {
        let p = self.padding;
        let x = area.x.saturating_add(1).saturating_add(p);
        let y = area.y.saturating_add(1).saturating_add(p);
        let w = area.width.saturating_sub(2).saturating_sub(p * 2);
        let h = area.height.saturating_sub(2).saturating_sub(p * 2);
        Rect::new(x, y, w, h)
    }

    fn draw_border(&self, area: Rect, grid: &mut Grid, s: Style) {
        if area.width < 2 || area.height < 2 {
            return;
        }
        let (tl, tr, bl, br, hz, vt) = self.border_type.glyphs();
        let last_x = area.right() - 1;
        let last_y = area.bottom() - 1;

        grid.set(area.x, area.y, tl, s);
        grid.set(last_x, area.y, tr, s);
        grid.set(area.x, last_y, bl, s);
        grid.set(last_x, last_y, br, s);
        for x in area.x + 1..last_x {
            grid.set(x, area.y, hz, s);
            grid.set(x, last_y, hz, s);
        }
        for y in area.y + 1..last_y {
            grid.set(area.x, y, vt, s);
            grid.set(last_x, y, vt, s);
        }
    }

    /// Clockwise path of *non-corner* border cells, starting just after the
    /// top-left corner: top edge, right edge, bottom edge (reversed), left
    /// edge (reversed). Corners are excluded so a title that wraps around a
    /// corner never overwrites the corner glyph; it jumps straight to the
    /// next edge instead. A title longer than one edge simply continues
    /// around the perimeter.
    fn perimeter(area: Rect) -> Vec<(u16, u16)> {
        let mut path = Vec::new();
        if area.width == 0 || area.height == 0 {
            return path;
        }
        if area.width < 3 || area.height < 3 {
            // Degenerate panel: there are no interior edge cells. Offer the
            // top edge so short titles still land somewhere.
            for x in area.x..area.right() {
                path.push((x, area.y));
            }
            return path;
        }
        let last_x = area.right() - 1;
        let last_y = area.bottom() - 1;
        for x in area.x + 1..last_x {
            path.push((x, area.y));
        }
        for y in area.y + 1..last_y {
            path.push((last_x, y));
        }
        for x in (area.x + 1..last_x).rev() {
            path.push((x, last_y));
        }
        for y in (area.y + 1..last_y).rev() {
            path.push((area.x, y));
        }
        path
    }

    fn draw_title(&self, area: Rect, grid: &mut Grid, title_style: Style) {
        if self.title.is_empty() || area.width < 2 || area.height < 2 {
            return;
        }
        let mut path = Self::perimeter(area);
        if path.is_empty() {
            return;
        }
        // The perimeter is clockwise (top L→R, right T→B, bottom R→L, left
        // B→T). Titles on the bottom or left edge must read in the natural
        // direction, so reverse the path for those positions.
        if matches!(
            self.title_position,
            TitlePosition::Bottom | TitlePosition::Left
        ) {
            path.reverse();
        }

        // Edge lengths (number of non-corner cells) and start indices into
        // the perimeter path built above (which excludes corners).
        let w = area.width as usize;
        let h = area.height as usize;
        let top_len = w.saturating_sub(2);
        let right_len = h.saturating_sub(2);
        let bottom_len = w.saturating_sub(2);
        let left_len = h.saturating_sub(2);
        // For Bottom/Left the path was reversed above, so the segment order
        // is [left, bottom, right, top] instead of [top, right, bottom, left].
        let (edge_start, edge_len) = match self.title_position {
            TitlePosition::Top => (0usize, top_len),
            TitlePosition::Right => (top_len, right_len),
            TitlePosition::Bottom => (left_len, bottom_len),
            TitlePosition::Left => (0usize, left_len),
        };
        let title_w = self.title.chars().count();
        let offset = match self.title_alignment {
            TitleAlignment::Left => 0,
            TitleAlignment::Center => edge_len.saturating_sub(title_w) / 2,
            TitleAlignment::Right => edge_len.saturating_sub(title_w),
        };
        let mut idx = edge_start.saturating_add(offset) % path.len();
        for ch in self.title.chars() {
            if ch == ' ' {
                idx = (idx + 1) % path.len();
                continue;
            }
            let cw = char_width(ch);
            if cw == 0 {
                continue;
            }
            let (x, y) = path[idx];
            grid.set(x, y, ch, title_style);
            if cw > 1 && idx + 1 < path.len() {
                let (x2, y2) = path[idx + 1];
                grid.set(x2, y2, ' ', title_style);
                idx = (idx + 2) % path.len();
            } else {
                idx = (idx + 1) % path.len();
            }
        }
    }

    fn draw_content(&self, area: Rect, grid: &mut Grid, style: Style) {
        let inner = self.inner(area);
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        let width = inner.width as usize;
        let mut y = inner.y;
        for line in &self.content {
            let pieces: Vec<String> = if self.wrap_content {
                wrap_text(line, width)
            } else {
                vec![line.clone()]
            };
            for piece in pieces {
                if y >= inner.bottom() {
                    return;
                }
                grid.set_str(inner.x, y, &piece, style);
                y += 1;
            }
        }
    }
}

impl Default for Panel {
    fn default() -> Panel {
        Panel::new()
    }
}

impl Widget for Panel {
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        // Apply panel opacity to every drawn style so a semi-transparent
        // panel blends over the canvas background (frosted glass).
        let border_style = if self.opacity < 255 {
            self.border_style.opacity(self.opacity)
        } else {
            self.border_style
        };
        let content_style = if self.opacity < 255 {
            self.content_style.opacity(self.opacity)
        } else {
            self.content_style
        };
        let title_style = if self.opacity < 255 {
            self.title_style.opacity(self.opacity)
        } else {
            self.title_style
        };
        self.draw_border(area, grid, border_style);
        self.draw_content(area, grid, content_style);
        self.draw_title(area, grid, title_style); // title overwrites border glyphs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Color;

    #[test]
    fn rounded_border_corners() {
        let p = Panel::new().borders(BorderType::Rounded);
        let mut g = Grid::new(6, 4, crate::grid::Cell::blank());
        p.render(Rect::new(0, 0, 6, 4), &mut g);
        assert_eq!(g.get(0, 0).ch, '╭');
        assert_eq!(g.get(5, 0).ch, '╮');
        assert_eq!(g.get(0, 3).ch, '╰');
        assert_eq!(g.get(5, 3).ch, '╯');
    }

    #[test]
    fn long_title_wraps_around_corner() {
        let p = Panel::new()
            .title("abcdefghijkl")
            .borders(BorderType::Plain);
        let mut g = Grid::new(8, 4, crate::grid::Cell::blank());
        p.render(Rect::new(0, 0, 8, 4), &mut g);
        // Top edge holds ~6 chars, then wrapping continues down the right.
        assert_eq!(g.get(1, 0).ch, 'a');
        assert_eq!(g.get(6, 0).ch, 'f');
        assert_eq!(g.get(7, 1).ch, 'g');
        assert_eq!(g.get(7, 2).ch, 'h');
    }

    #[test]
    fn bottom_and_left_titles_read_naturally() {
        let p = Panel::new()
            .title("HELLO")
            .title_position(TitlePosition::Bottom);
        let mut g = Grid::new(10, 4, crate::grid::Cell::blank());
        p.render(Rect::new(0, 0, 10, 4), &mut g);
        // bottom edge reads left→right (not mirrored by the clockwise path)
        assert_eq!(g.get(1, 3).ch, 'H');
        assert_eq!(g.get(5, 3).ch, 'O');

        let p2 = Panel::new().title("HI").title_position(TitlePosition::Left);
        let mut g2 = Grid::new(10, 6, crate::grid::Cell::blank());
        p2.render(Rect::new(0, 0, 10, 6), &mut g2);
        // left edge reads top→bottom
        assert_eq!(g2.get(0, 1).ch, 'H');
        assert_eq!(g2.get(0, 2).ch, 'I');
    }

    #[test]
    fn double_border_glyphs() {
        let p = Panel::new().borders(BorderType::Double);
        let mut g = Grid::new(5, 3, crate::grid::Cell::blank());
        p.render(Rect::new(0, 0, 5, 3), &mut g);
        assert_eq!(g.get(0, 0).ch, '╔');
        assert_eq!(g.get(1, 0).ch, '═');
        assert_eq!(g.get(0, 1).ch, '║');
    }

    #[test]
    fn content_is_wrapped() {
        let p = Panel::new().content(["one two three"]);
        let mut g = Grid::new(10, 5, crate::grid::Cell::blank());
        p.render(Rect::new(0, 0, 10, 5), &mut g);
        assert_eq!(g.get(1, 1).ch, 'o');
        assert_eq!(g.get(1, 2).ch, 't');
        let _ = Color::Ansi(1);
    }
}
