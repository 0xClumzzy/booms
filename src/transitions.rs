//! View transitions: fade, slide, and cube wipes between full-screen
//! views, driven by eased [`Anim`](crate::anim::Anim) progress.
//!
//! A transition composites two grids: the outgoing view and the incoming
//! view, based on the current progress, producing cross-fades, directional
//! slides, and cube-style wipes in the terminal.

use std::time::Duration;

use crate::anim::{Anim, Easing};
use crate::grid::Grid;

/// Direction for slide transitions.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SlideDirection {
    /// The incoming view slides in from the left.
    #[default]
    Left,
    /// The incoming view slides in from the right.
    Right,
    /// The incoming view slides in from the top.
    Up,
    /// The incoming view slides in from the bottom.
    Down,
}

/// A transition between two views.
#[derive(Clone, Debug)]
pub enum Transition {
    /// Cross-fade: outgoing fades out as incoming fades in, with a
    /// speckled dissolve pattern.
    Fade { progress: Anim<f32> },
    /// Slide: incoming view slides in from a direction, revealing over
    /// the outgoing view.
    Slide {
        progress: Anim<f32>,
        direction: SlideDirection,
    },
    /// Cube: vertical curtain strips reveal the incoming view
    /// sequentially, like a rotating cube face.
    Cube { progress: Anim<f32> },
}

impl Transition {
    /// Create a fade transition over `duration`.
    pub fn fade(duration: Duration) -> Transition {
        Transition::Fade {
            progress: Anim::new(0.0, 1.0, duration).easing(Easing::SineInOut),
        }
    }

    /// Create a slide transition over `duration`.
    pub fn slide(duration: Duration, direction: SlideDirection) -> Transition {
        Transition::Slide {
            progress: Anim::new(0.0, 1.0, duration).easing(Easing::SmoothStep),
            direction,
        }
    }

    /// Create a cube-wipe transition over `duration`.
    pub fn cube(duration: Duration) -> Transition {
        Transition::Cube {
            progress: Anim::new(0.0, 1.0, duration).easing(Easing::SineInOut),
        }
    }

    /// Advance the transition by one tick. Returns `true` when the
    /// transition has completed.
    pub fn tick(&mut self) -> bool {
        let finished = match self {
            Transition::Fade { progress } => progress.update() >= 1.0,
            Transition::Slide { progress, .. } => progress.update() >= 1.0,
            Transition::Cube { progress } => progress.update() >= 1.0,
        };
        if finished {
            match self {
                Transition::Fade { progress } => progress.restart(),
                Transition::Slide { progress, .. } => progress.restart(),
                Transition::Cube { progress } => progress.restart(),
            }
        }
        finished
    }

    /// The current eased progress (`0.0` = start, `1.0` = end).
    pub fn progress(&self) -> f32 {
        match self {
            Transition::Fade { progress } => *progress.value(),
            Transition::Slide { progress, .. } => *progress.value(),
            Transition::Cube { progress } => *progress.value(),
        }
    }

    /// Whether the transition has reached its end.
    pub fn is_finished(&self) -> bool {
        match self {
            Transition::Fade { progress } => progress.is_finished(),
            Transition::Slide { progress, .. } => progress.is_finished(),
            Transition::Cube { progress } => progress.is_finished(),
        }
    }
}

/// Composite `from` (outgoing) and `to` (incoming) grids into `out` using
/// a cross-fade dissolve. At each cell, `to` is shown when progress
/// exceeds a per-cell threshold: producing a speckled fade.
pub fn composite_fade(from: &Grid, to: &Grid, progress: f32, out: &mut Grid) {
    let w = from.width().min(to.width()).min(out.width());
    let h = from.height().min(to.height()).min(out.height());
    for y in 0..h {
        for x in 0..w {
            let threshold =
                ((x as f32 + y as f32) / (w.max(1) as f32 + h.max(1) as f32)) * 0.6 + 0.2;
            let cell = if progress >= threshold {
                to.get(x, y)
            } else {
                from.get(x, y)
            };
            out.set(x, y, cell.ch, cell.style);
        }
    }
}

/// Composite `from` and `to` grids into `out` using a directional slide:
/// `to` is revealed from `direction` over `out` while `from` remains in
/// the unrevealed area.
pub fn composite_slide(
    from: &Grid,
    to: &Grid,
    progress: f32,
    direction: SlideDirection,
    out: &mut Grid,
) {
    let w = from.width().min(to.width()).min(out.width());
    let h = from.height().min(to.height()).min(out.height());
    for y in 0..h {
        for x in 0..w {
            let (show_to, tx, ty) = match direction {
                SlideDirection::Left => {
                    let shift = (w as f32 * progress) as u16;
                    let show = x < shift;
                    (show, x, y)
                }
                SlideDirection::Right => {
                    let shift = (w as f32 * progress) as u16;
                    let show = x >= w.saturating_sub(shift);
                    let tx = if show { x.saturating_sub(shift) } else { x };
                    (show, tx, y)
                }
                SlideDirection::Up => {
                    let shift = (h as f32 * progress) as u16;
                    let show = y < shift;
                    (show, x, y)
                }
                SlideDirection::Down => {
                    let shift = (h as f32 * progress) as u16;
                    let show = y >= h.saturating_sub(shift);
                    let ty = if show { y.saturating_sub(shift) } else { y };
                    (show, x, ty)
                }
            };
            let cell = if show_to {
                to.get(tx, ty)
            } else {
                from.get(x, y)
            };
            out.set(x, y, cell.ch, cell.style);
        }
    }
}

/// Composite `from` and `to` grids into `out` using a cube-wipe:
/// vertical strips reveal `to` sequentially from left to right based on
/// progress.
pub fn composite_cube(from: &Grid, to: &Grid, progress: f32, out: &mut Grid) {
    let w = from.width().min(to.width()).min(out.width());
    let h = from.height().min(to.height()).min(out.height());
    let strips = w.max(1);
    let reveal_col = (progress * strips as f32) as u16;
    for y in 0..h {
        for x in 0..w {
            let cell = if x < reveal_col {
                to.get(x, y)
            } else {
                from.get(x, y)
            };
            out.set(x, y, cell.ch, cell.style);
        }
    }
}

/// Composite two grids into `out` according to the transition's kind and
/// progress.
pub fn composite(transition: &Transition, from: &Grid, to: &Grid, out: &mut Grid) {
    let progress = transition.progress();
    match transition {
        Transition::Fade { .. } => composite_fade(from, to, progress, out),
        Transition::Slide { direction, .. } => composite_slide(from, to, progress, *direction, out),
        Transition::Cube { .. } => composite_cube(from, to, progress, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Cell;
    use crate::style::{Color, Style};

    fn make_grid(w: u16, h: u16, ch: char, color: Color) -> Grid {
        let mut g = Grid::new(w, h, Cell::blank());
        for y in 0..h {
            for x in 0..w {
                g.set(x, y, ch, Style::DEFAULT.fg(color));
            }
        }
        g
    }

    #[test]
    fn fade_starts_as_from() {
        let from = make_grid(10, 5, 'A', Color::Ansi(1));
        let to = make_grid(10, 5, 'B', Color::Ansi(2));
        let mut out = Grid::new(10, 5, Cell::blank());
        let t = Transition::fade(std::time::Duration::from_millis(100));
        composite(&t, &from, &to, &mut out);
        // At progress 0, the threshold is >= 0.2, so nothing shows 'to'.
        for y in 0..5 {
            for x in 0..10 {
                assert_eq!(out.get(x, y).ch, 'A');
            }
        }
    }

    #[test]
    fn slide_reveals_from_at_zero() {
        let from = make_grid(10, 5, 'A', Color::Ansi(1));
        let to = make_grid(10, 5, 'B', Color::Ansi(2));
        let mut out = Grid::new(10, 5, Cell::blank());
        let t = Transition::slide(std::time::Duration::from_millis(100), SlideDirection::Left);
        // Progress is 0 at construction, so all 'A'.
        composite(&t, &from, &to, &mut out);
        for y in 0..5 {
            for x in 0..10 {
                assert_eq!(out.get(x, y).ch, 'A');
            }
        }
    }
}
