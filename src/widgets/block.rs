//! Block: a bordered box with an optional title.

use crate::grid::{Grid, char_width};
use crate::layout::{Constraint, Rect};
use crate::style::Style;
use crate::widgets::Widget;

/// Border flags.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Borders(u8);

impl Borders {
    pub const NONE: Borders = Borders(0);
    pub const LEFT: Borders = Borders(1);
    pub const RIGHT: Borders = Borders(2);
    pub const TOP: Borders = Borders(4);
    pub const BOTTOM: Borders = Borders(8);
    pub const ALL: Borders = Borders(15);

    /// True if every border in `other` is also in `self`.
    pub const fn contains(self, other: Borders) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn union(self, other: Borders) -> Borders {
        Borders(self.0 | other.0)
    }
}

impl std::ops::BitOr for Borders {
    type Output = Borders;

    fn bitor(self, rhs: Borders) -> Borders {
        self.union(rhs)
    }
}

/// A bordered box with an optional title on the top border.
#[derive(Clone, Debug, Default)]
pub struct Block {
    title: Option<String>,
    title_style: Style,
    borders: Borders,
    border_style: Style,
}

impl Block {
    pub fn new() -> Block {
        Block::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Block {
        self.title = Some(title.into());
        self
    }

    pub fn borders(mut self, borders: Borders) -> Block {
        self.borders = borders;
        self
    }

    pub fn border_style(mut self, style: Style) -> Block {
        self.border_style = style;
        self
    }

    pub fn title_style(mut self, style: Style) -> Block {
        self.title_style = style;
        self
    }

    /// The rectangle inside the border.
    pub fn inner(&self, area: Rect) -> Rect {
        let b = self.borders;
        let left = b.contains(Borders::LEFT) as u16;
        let top = b.contains(Borders::TOP) as u16;
        let x = area.x + left;
        let y = area.y + top;
        let right_x = area
            .right()
            .saturating_sub(b.contains(Borders::RIGHT) as u16);
        let bottom_y = area
            .bottom()
            .saturating_sub(b.contains(Borders::BOTTOM) as u16);
        Rect::new(x, y, right_x.saturating_sub(x), bottom_y.saturating_sub(y))
    }
}

impl Widget for Block {
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.is_empty() {
            return;
        }
        let b = self.borders;
        let style = self.border_style;
        let has_top = b.contains(Borders::TOP);
        let has_bottom = b.contains(Borders::BOTTOM);
        let has_left = b.contains(Borders::LEFT);
        let has_right = b.contains(Borders::RIGHT);
        let top = area.y;
        let bottom = area.bottom().saturating_sub(1);
        let left = area.x;
        let right = area.right().saturating_sub(1);

        if has_top {
            for x in left..=right {
                let ch = if has_left && x == left {
                    '┌'
                } else if has_right && x == right {
                    '┐'
                } else {
                    '─'
                };
                grid.set(x, top, ch, style);
            }
            if let Some(title) = &self.title {
                for (x, ch) in (left + 1..).zip(title.chars()) {
                    if char_width(ch) == 0 || x >= right {
                        break;
                    }
                    grid.set(x, top, ch, self.title_style);
                }
            }
        }

        if has_bottom && bottom != top {
            for x in left..=right {
                let ch = if has_left && x == left {
                    '└'
                } else if has_right && x == right {
                    '┘'
                } else {
                    '─'
                };
                grid.set(x, bottom, ch, style);
            }
        }

        for y in (top + 1)..bottom {
            if has_left {
                grid.set(left, y, '│', style);
            }
            if has_right {
                grid.set(right, y, '│', style);
            }
        }
    }
}
