//! Liquid gooey orb: a blobby, organic shape whose edge ripples with
//! sine waves and settles with spring-physics (ported from liquid-gooey).
//! The blob wobbles, morphs, and cycles through a liquid color palette.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::anim::{Anim, Easing, Spring};
use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Style};
use crate::widgets::Widget;

/// Grid size of the gooey orb (square).
const GOOEY_SIZE: usize = 9;
/// Center of the orb grid.
const GOOEY_CENTER: f32 = (GOOEY_SIZE / 2) as f32;

/// Liquid color palette: cyan → blue → teal → white.
const GOOEY_COLORS: [Color; 4] = [
    Color::Ansi(6),  // cyan
    Color::Ansi(12), // blue
    Color::Ansi(6),  // cyan again (teal-ish)
    Color::Ansi(15), // white
];

/// A liquid gooey orb: an organic blob whose edge ripples with interfering
/// sine waves, driven by a spring for natural wobble and settling.
pub struct LiquidGooeyOrb {
    /// Spring-driven wobble amplitude; when settled the blob is still.
    wobble: Spring,
    /// Edge-morph phase, 0.0 → 1.0 over the morph cycle.
    morph: Anim<f32>,
    /// Color-cycle phase through the liquid palette.
    hue: Anim<f32>,
}

impl LiquidGooeyOrb {
    /// Create a liquid gooey orb, initially settled.
    pub fn new() -> LiquidGooeyOrb {
        LiquidGooeyOrb {
            wobble: Spring::new(0.0, 0.0, 180.0, 55.0, 1.0),
            morph: Anim::new(0.0, 1.0, Duration::from_millis(3200)).easing(Easing::Linear),
            hue: Anim::new(0.0, 1.0, Duration::from_millis(4800)).easing(Easing::SineInOut),
        }
    }

    /// Nudge the gooey to start wobbling (call from outside to "poke" it).
    pub fn nudge(&mut self) {
        self.wobble.set_target(0.45);
    }
}

impl Default for LiquidGooeyOrb {
    fn default() -> LiquidGooeyOrb {
        LiquidGooeyOrb::new()
    }
}

impl Widget for LiquidGooeyOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(GOOEY_SIZE as u16)
    }

    fn on_tick(&mut self) {
        // Integrate the spring for organic wobble.
        self.wobble.tick();
        // Periodically re-nudge so the blob never fully goes still.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();
        if self.wobble.is_settled() && now.sin() > 0.98 {
            self.wobble.set_target(0.35 + 0.15 * now.cos());
        }
        if self.morph.update() >= 1.0 {
            self.morph.restart();
        }
        if self.hue.update() >= 1.0 {
            self.hue.restart();
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let morph = self.morph.value();
        let hue = self.hue.value();
        let wobble = self.wobble.value().clamp(0.0, 1.0);

        let cx = area.x + GOOEY_SIZE as u16 / 2;
        let cy = area.y + GOOEY_SIZE as u16 / 2;
        let half = GOOEY_SIZE as u16 / 2;

        let color = GOOEY_COLORS[((hue * GOOEY_COLORS.len() as f32) as usize) % GOOEY_COLORS.len()];

        for dy in 0..GOOEY_SIZE {
            for dx in 0..GOOEY_SIZE {
                let x = cx.saturating_sub(half) + dx as u16;
                let y = cy.saturating_sub(half) + dy as u16;
                if x >= area.right() || y >= area.bottom() {
                    continue;
                }

                let dx_f = dx as f32 - GOOEY_CENTER;
                let dy_f = dy as f32 - GOOEY_CENTER;
                let dist = (dx_f * dx_f + dy_f * dy_f).sqrt();

                // Interfering sine waves distort the blob edge.
                let angle = dy_f.atan2(dx_f);
                let ripple1 = (angle * 3.0 + morph * std::f32::consts::TAU).sin() * 0.6;
                let ripple2 = (angle * 7.0 - morph * 4.0 + 1.0).cos() * 0.4;
                let ripple3 = (angle * 2.0 + morph * 2.0).sin() * 0.25;
                let radius = 3.0 + ripple1 + ripple2 + ripple3 + wobble * 0.6;

                if dist <= radius {
                    // Character by depth: core → mid → edge.
                    let ch = if dist <= radius * 0.35 {
                        '●'
                    } else if dist <= radius * 0.65 {
                        '◍'
                    } else {
                        '○'
                    };
                    let style = if dist <= radius * 0.35 {
                        Style::DEFAULT.fg(color).mods(crate::style::Modifiers::BOLD)
                    } else {
                        Style::DEFAULT.fg(color)
                    };
                    grid.set(x, y, ch, style);
                }
            }
        }
    }
}
