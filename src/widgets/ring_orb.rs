//! Ring orb: concentric rotating rings of geometric characters, like a
//! crystal's electron shells. The rings rotate at different speeds using
//! eased [`Anim`] transitions.

use std::time::Duration;

use crate::anim::{Anim, Easing};
use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Style};
use crate::widgets::Widget;

/// Characters used to draw a ring, in drawing order.
const RING_CHARS: &[char] = &['○', '◌', '◍', '◎', '●', '◇', '◆', '✦'];

/// Colors cycled around the ring.
const RING_COLORS: &[Color] = &[
    Color::Ansi(14), // bright yellow
    Color::Ansi(12), // blue
    Color::Ansi(6),  // cyan
    Color::Ansi(15), // white
    Color::Ansi(13), // magenta
    Color::Ansi(11), // bright blue
];

/// A ring orb widget: concentric rings that rotate at different speeds.
pub struct RingOrb {
    /// Outer ring rotation, 0.0 → 1.0 (a full cycle of RING_CHARS).
    outer: Anim<f32>,
    /// Inner ring rotation, 0.0 → 1.0, faster than the outer.
    inner: Anim<f32>,
    /// Center column of the orb (relative to area.x).
    cx: u16,
    /// Center row of the orb (relative to area.y).
    cy: u16,
}

impl RingOrb {
    /// Create a ring orb centered in its area.
    pub fn new() -> RingOrb {
        RingOrb {
            outer: Anim::new(0.0, 1.0, Duration::from_millis(2400)).easing(Easing::Linear),
            inner: Anim::new(0.0, 1.0, Duration::from_millis(1200)).easing(Easing::SmoothStep),
            cx: 4,
            cy: 3,
        }
    }

    /// Set the orb's center (relative to the widget area).
    pub fn center(mut self, cx: u16, cy: u16) -> RingOrb {
        self.cx = cx;
        self.cy = cy;
        self
    }
}

impl Default for RingOrb {
    fn default() -> RingOrb {
        RingOrb::new()
    }
}

impl Widget for RingOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(7)
    }

    fn on_tick(&mut self) {
        if self.outer.update() >= 1.0 {
            self.outer.restart();
        }
        if self.inner.update() >= 1.0 {
            self.inner.restart();
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let cx = area.x + self.cx;
        let cy = area.y + self.cy;

        let outer_phase = self.outer.value();
        let inner_phase = self.inner.value();

        // Draw outer ring (radius 3) and inner ring (radius 1).
        for (ring_i, (radius, phase, _thickness)) in [(3, outer_phase, 1), (1, inner_phase, 1)]
            .iter()
            .enumerate()
        {
            // Sample points around the circle.
            for step in 0..16 {
                let angle = (step as f32 / 16.0) * 2.0 * std::f32::consts::PI
                    + *phase * 2.0 * std::f32::consts::PI;
                let dx = (angle.cos() * *radius as f32).round() as i16;
                let dy = (angle.sin() * *radius as f32 * 0.5).round() as i16; // flatten vertically
                let x = cx as i16 + dx;
                let y = cy as i16 + dy;
                if x < 0 || y < 0 {
                    continue;
                }
                let x = x as u16;
                let y = y as u16;
                if x >= area.right() || y >= area.bottom() {
                    continue;
                }

                // Pick a character and color based on position around the ring.
                let ch = RING_CHARS[(step + ring_i * 2) % RING_CHARS.len()];
                let color = RING_COLORS[(step + ring_i) % RING_COLORS.len()];
                let style = if step == 0 {
                    Style::DEFAULT.fg(color).mods(crate::style::Modifiers::BOLD)
                } else {
                    Style::DEFAULT.fg(color)
                };
                grid.set(x, y, ch, style);
            }
        }

        // Core dot in the very center, pulsing between bright and dim.
        let core_bright = (inner_phase * 2.0).sin() > 0.0;
        let core_ch = if core_bright { '●' } else { '○' };
        let core_color = if core_bright {
            Color::Ansi(15)
        } else {
            Color::Ansi(12)
        };
        if cx < area.right() && cy < area.bottom() {
            grid.set(
                cx,
                cy,
                core_ch,
                Style::DEFAULT.fg(core_color).mods(Modifiers::BOLD),
            );
        }
    }
}

use crate::style::Modifiers;
