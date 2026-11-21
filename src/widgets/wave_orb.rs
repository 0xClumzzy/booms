//! Distorted wave orb: concentric rings whose edge ripples with
//! interfering sine waves, like water distorted by a drop. Colors pulse
//! along the wave phase.

use std::time::Duration;

use crate::anim::{Anim, Easing};
use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Style};
use crate::widgets::Widget;

/// Grid size of the wave orb (square).
const WAVE_SIZE: usize = 11;
/// Center of the orb grid.
const WAVE_CENTER: f32 = (WAVE_SIZE / 2) as f32;

/// A distorted wave orb: rings whose radius varies with sine waves,
/// creating interference patterns. The core pulses and colors cycle.
pub struct WaveOrb {
    /// Primary wave phase, 0.0 → 1.0.
    phase: Anim<f32>,
    /// Secondary wave phase, different frequency, 0.0 → 1.0.
    phase2: Anim<f32>,
    /// Color pulse phase, 0.0 → 1.0.
    pulse: Anim<f32>,
}

impl WaveOrb {
    /// Create a distorted wave orb, initially at rest phase.
    pub fn new() -> WaveOrb {
        WaveOrb {
            phase: Anim::new(0.0, 1.0, Duration::from_millis(2000)).easing(Easing::Linear),
            phase2: Anim::new(0.0, 1.0, Duration::from_millis(3700)).easing(Easing::SmoothStep),
            pulse: Anim::new(0.0, 1.0, Duration::from_millis(1500)).easing(Easing::SineInOut),
        }
    }
}

impl Default for WaveOrb {
    fn default() -> WaveOrb {
        WaveOrb::new()
    }
}

impl Widget for WaveOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(WAVE_SIZE as u16)
    }

    fn on_tick(&mut self) {
        if self.phase.update() >= 1.0 {
            self.phase.restart();
        }
        if self.phase2.update() >= 1.0 {
            self.phase2.restart();
        }
        if self.pulse.update() >= 1.0 {
            self.pulse.restart();
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let phase = self.phase.value();
        let phase2 = self.phase2.value();
        let pulse = self.pulse.value();

        let cx = area.x + WAVE_SIZE as u16 / 2;
        let cy = area.y + WAVE_SIZE as u16 / 2;
        let half = WAVE_SIZE as u16 / 2;

        for dy in 0..WAVE_SIZE {
            for dx in 0..WAVE_SIZE {
                let x = cx.saturating_sub(half) + dx as u16;
                let y = cy.saturating_sub(half) + dy as u16;
                if x >= area.right() || y >= area.bottom() {
                    continue;
                }

                let dx_f = dx as f32 - WAVE_CENTER;
                let dy_f = dy as f32 - WAVE_CENTER;
                let dist = (dx_f * dx_f + dy_f * dy_f).sqrt();

                if dist == 0.0 {
                    // Pulsing core.
                    let ch = if *pulse > 0.5 { '●' } else { '○' };
                    grid.set(
                        x,
                        y,
                        ch,
                        Style::DEFAULT
                            .fg(Color::Ansi(15))
                            .mods(crate::style::Modifiers::BOLD),
                    );
                    continue;
                }

                let angle = dy_f.atan2(dx_f);

                // Two interfering sine waves distort the ring radius.
                let wave1 = (angle * 6.0 - phase * std::f32::consts::TAU * 2.0).sin() * 0.9;
                let wave2 = (angle * 11.0 + phase2 * std::f32::consts::TAU * 3.0).cos() * 0.55;
                let radius = 3.5 + wave1 + wave2;

                // Ring band: cells near the distorted radius.
                let band = (dist - radius).abs();
                if band <= 0.75 {
                    let ch = if band <= 0.25 {
                        '●'
                    } else if band <= 0.5 {
                        '◍'
                    } else {
                        '○'
                    };
                    // Color pulses along the wave angle.
                    let idx = ((pulse * 4.0 + angle * 0.5) as isize).rem_euclid(4) as usize;
                    let color = match idx {
                        0 => Color::Ansi(6),  // cyan
                        1 => Color::Ansi(12), // blue
                        2 => Color::Ansi(14), // bright yellow
                        _ => Color::Ansi(15), // white
                    };
                    let style = if band <= 0.3 {
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
