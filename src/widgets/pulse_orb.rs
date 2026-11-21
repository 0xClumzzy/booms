//! Nano-style pulsing orb: a tiny status indicator whose brightness
//! breathes in and out (dim → bright → dim) on an eased curve, driven
//! from [`on_tick`](crate::widgets::Widget::on_tick) like nano's
//! modified/recording dot.

use std::time::Duration;

use crate::anim::{Anim, Easing};
use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Modifiers, Style};
use crate::widgets::Widget;

/// A tiny pulsing orb, like nano's modified/recording indicator.
///
/// The orb breathes between a dim and a bright color. The character
/// switches between `○` (dim) and `●` (bright) as it pulses.
pub struct PulseOrb {
    /// Interpolates between the dim and bright colors.
    pulse: Anim<Color>,
    /// Character shown when dim.
    dim_ch: char,
    /// Character shown when bright.
    bright_ch: char,
}

impl PulseOrb {
    /// Create a pulsing orb with the given bright color.
    pub fn new(color: Color) -> PulseOrb {
        let dim = match color {
            Color::Ansi(n) => Color::Ansi(n.saturating_sub(6)),
            Color::Rgb(r, g, b) => Color::Rgb(
                r.saturating_div(3),
                g.saturating_div(3),
                b.saturating_div(3),
            ),
            _ => Color::Default,
        };
        PulseOrb {
            pulse: Anim::new(dim, color, Duration::from_millis(1000)).easing(Easing::SineInOut),
            dim_ch: '○',
            bright_ch: '●',
        }
    }

    /// Create a pulsing orb with default bright green.
    pub fn default_green() -> PulseOrb {
        PulseOrb::new(Color::Ansi(10))
    }

    /// Set the dim character.
    pub fn dim_char(mut self, ch: char) -> PulseOrb {
        self.dim_ch = ch;
        self
    }

    /// Set the bright character.
    pub fn bright_char(mut self, ch: char) -> PulseOrb {
        self.bright_ch = ch;
        self
    }
}

impl Default for PulseOrb {
    fn default() -> PulseOrb {
        PulseOrb::default_green()
    }
}

impl Widget for PulseOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn on_tick(&mut self) {
        if self.pulse.update() >= 1.0 {
            self.pulse.restart();
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let color = self.pulse.value();
        let bright = match (color, self.pulse.from()) {
            (Color::Ansi(n), Color::Ansi(base)) => *n > *base,
            _ => true,
        };
        let ch = if bright { self.bright_ch } else { self.dim_ch };
        let style = if bright {
            Style::DEFAULT.fg(*color).mods(Modifiers::BOLD)
        } else {
            Style::DEFAULT.fg(*color)
        };
        grid.set(area.x, area.y, ch, style);
    }
}
