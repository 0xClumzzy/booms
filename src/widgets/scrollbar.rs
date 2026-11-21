//! A vertical scrollbar: a track with a proportional thumb showing the
//! viewport's position within a longer list of lines.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Modifiers, Style};
use crate::widgets::Widget;

/// A vertical scrollbar widget.
///
/// Stateless: build one per frame with the content's total line count,
/// the visible viewport height, and the current scroll offset, then
/// [`render`](Widget::render) it into a one-column-wide [`Rect`]. When
/// the content fits the viewport it paints a plain track.
pub struct Scrollbar {
    /// Total content lines.
    total: usize,
    /// Lines visible in the viewport.
    visible: usize,
    /// First visible content line.
    offset: usize,
}

impl Scrollbar {
    /// Create a scrollbar for `total` lines of content, `visible` of
    /// which are shown, scrolled to `offset`.
    pub fn new(total: usize, visible: usize, offset: usize) -> Scrollbar {
        Scrollbar {
            total,
            visible,
            offset,
        }
    }

    /// Whether the content overflows the viewport, i.e. whether there is
    /// anything to scroll.
    pub fn is_scrollable(&self) -> bool {
        self.total > self.visible
    }

    /// The maximum scroll offset (0 when the content fits).
    pub fn max_offset(&self) -> usize {
        self.total.saturating_sub(self.visible)
    }

    /// Map a row within the scrollbar's area to a content offset. Useful for
    /// implementing click-to-jump / drag on a stateful parent widget.
    pub fn offset_for_row(&self, row: usize, area_height: usize) -> usize {
        if area_height == 0 || !self.is_scrollable() {
            return 0;
        }
        let h = area_height;
        let ratio = self.visible as f32 / self.total as f32;
        let thumb = (((h as f32) * ratio).round().max(1.0) as usize).min(h);
        let free = h.saturating_sub(thumb);
        let frac = (row.min(h.saturating_sub(1)) as f32 / free.max(1) as f32).clamp(0.0, 1.0);
        (frac * self.max_offset() as f32).round() as usize
    }
}

impl Widget for Scrollbar {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    /// Consume scroll-wheel events over the scrollbar. The scrollbar is
    /// stateless; a stateful parent should update its own offset in response
    /// (e.g. via [`offset_for_row`](Scrollbar::offset_for_row) on drag).
    fn on_mouse(&mut self, event: MouseEvent, _area: Rect) -> bool {
        matches!(
            event.kind,
            MouseEventKind::ScrollUp
                | MouseEventKind::ScrollDown
                | MouseEventKind::Down(MouseButton::Left)
                | MouseEventKind::Drag(MouseButton::Left)
        )
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let h = area.height as usize;
        if h == 0 {
            return;
        }
        let track_style = Style::DEFAULT.fg(Color::Ansi(8));
        let thumb_style = Style::DEFAULT.fg(Color::Ansi(7)).mods(Modifiers::BOLD);

        if !self.is_scrollable() {
            for i in 0..h {
                grid.set(area.x, area.y + i as u16, '│', track_style);
            }
            return;
        }

        // Thumb height is proportional to visible/total, at least 1.
        let ratio = self.visible as f32 / self.total as f32;
        let thumb = ((h as f32) * ratio).round().max(1.0) as usize;
        let thumb = thumb.min(h);
        // Thumb position: offset/max_offset across the free track space.
        let frac = self.offset.min(self.max_offset()) as f32 / self.max_offset() as f32;
        let thumb_start = ((h - thumb) as f32 * frac.clamp(0.0, 1.0)).round() as usize;

        for i in 0..h {
            let in_thumb = i >= thumb_start && i < thumb_start + thumb;
            let ch = if in_thumb { '█' } else { '│' };
            let style = if in_thumb { thumb_style } else { track_style };
            grid.set(area.x, area.y + i as u16, ch, style);
        }
    }
}
