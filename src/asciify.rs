//! Animated ASCII scenes: a terminal port of the
//! [asciify-engine](https://github.com/ayangabryl/asciify-engine) idea.
//!
//! The engine converts pixels to character grids with a density ramp by
//! brightness; a terminal has no pixels, so this module draws procedural
//! animated scenes straight onto a [`Grid`], dithered with an ordered
//! Bayer pattern so smooth fields move through neighbouring ramp steps
//! instead of flickering. Scenes are pure functions of an elapsed time
//! `t`: the same `t` always produces the same frame.
//!
//! The engine itself is an npm/browser library with no Rust crate of the
//! same name, so this is a concept port: the density ramp
//! ` .:-=+*#%@`, the ordered dithering, and hue-swept color, applied to
//! procedural scenes.

use crate::grid::Grid;
use crate::style::{Color, Style};

/// The density ramp, from empty to dense (the engine's dark-background
/// `FILL` ramp).
pub const FILL: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

/// Bayer ordered-dither thresholds (4x4 grid), 0..15.
const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// Pick a cell's character from a scene density value (0..=1) at `(x, y)`,
/// dithered with the Bayer pattern so smooth fields animate smoothly.
fn dithered(x: u16, y: u16, value: f64) -> char {
    let dither = (BAYER[(y % 4) as usize][(x % 4) as usize] as f64 + 0.5) / 16.0;
    let idx = ((value - dither) * FILL.len() as f64)
        .floor()
        .clamp(0.0, FILL.len() as f64 - 1.0) as usize;
    FILL[idx]
}

/// Classic HSV to RGB, used for the hue-swept palettes.
fn hsv_rgb(h: f64, s: f64, v: f64) -> Color {
    let c = v * s;
    let hp = (((h % 1.0) + 1.0) % 1.0) * 6.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r, g, b) = match hp as i32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    Color::Rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// A hue-swept color cycling violet -> cyan -> green, one sweep per 3.0 of `t`.
fn sweep(t: f64) -> Color {
    let h = ((t * 0.3) % 3.0) / 3.0;
    hsv_rgb(h, 0.55, 0.9)
}

/// An animated ASCII scene kind.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AsciifyEffect {
    /// Plasma-shape sine field: a drifting main wave over a vertical ripple.
    Wave,
    /// Low-density drifting lissajous fields: the fog/mist look.
    Fog,
    /// Particles on orbits around the center, with glows and rings.
    Orbits,
    /// Pulsing bars: beats sweeping across, each phase-shifted.
    Pulses,
}

impl AsciifyEffect {
    /// Every effect kind, for cycling or selection UIs.
    pub const ALL: &[AsciifyEffect] = &[
        AsciifyEffect::Wave,
        AsciifyEffect::Fog,
        AsciifyEffect::Orbits,
        AsciifyEffect::Pulses,
    ];

    /// The effect's name, for displays and pickers.
    pub fn name(&self) -> &'static str {
        match self {
            AsciifyEffect::Wave => "wave",
            AsciifyEffect::Fog => "fog",
            AsciifyEffect::Orbits => "orbits",
            AsciifyEffect::Pulses => "pulses",
        }
    }

    /// The next effect in [`AsciifyEffect::ALL`], wrapping around.
    pub fn next(&self) -> AsciifyEffect {
        let i = AsciifyEffect::ALL
            .iter()
            .position(|e| e == self)
            .unwrap_or(0);
        AsciifyEffect::ALL[(i + 1) % AsciifyEffect::ALL.len()]
    }

    /// Render one animated frame: fill `grid` with characters and colors
    /// from the [`FILL`] ramp, at elapsed time `t` (seconds). Pure and
    /// deterministic: the same `t` always gives the same frame.
    pub fn tick(&self, t: f64, grid: &mut Grid) {
        let (gw, gh) = (grid.width(), grid.height());
        if gw == 0 || gh == 0 {
            return;
        }
        let (w, h) = (gw as f64, gh as f64);
        let (cx, cy) = (w / 2.0, h / 2.0);
        let color = match self {
            AsciifyEffect::Wave => sweep(t * 0.4),
            AsciifyEffect::Fog => sweep(t * 0.4 + 0.3),
            AsciifyEffect::Orbits => Color::Rgb(90, 60, 240),
            AsciifyEffect::Pulses => sweep(t * 0.2 + 0.5),
        };
        let style = Style::new().fg(color);
        for y in 0..gh {
            for x in 0..gw {
                let (ux, uy) = (x as f64 / w, y as f64 / h);
                let v = match self {
                    AsciifyEffect::Wave => wave(ux, uy, t),
                    AsciifyEffect::Fog => fog(ux, uy, t),
                    AsciifyEffect::Orbits => orbits(x as f64, y as f64, cx, cy, t),
                    AsciifyEffect::Pulses => pulses(ux, uy, t),
                };
                grid.set(x, y, dithered(x, y, v.clamp(0.0, 1.0)), style);
            }
        }
    }
}

/// Plasma field: drifting main sine over a vertical ripple with a drifting
/// diagonal term: the shape-heavy wave.
fn wave(ux: f64, uy: f64, t: f64) -> f64 {
    let main = (ux * std::f64::consts::PI * 2.4 * 1.6
        + t * 1.3
        + (uy * std::f64::consts::PI * 2.9 + t * 0.7) * (t * 0.35).sin() * 0.4)
        .sin();
    let ripple = (uy * std::f64::consts::PI * 1.7 * 1.6 - t * 0.9
        + (ux * std::f64::consts::PI * 0.8 + t * 0.5) * 0.6)
        .sin();
    0.5 * (main + ripple) + 0.45
}

/// Low-density drifting lissajous fields; `t` shifts each, sweeping the fog.
fn fog(ux: f64, uy: f64, t: f64) -> f64 {
    let mut v = 0.0;
    let offsets = [(0.0, 0.0), (2.3, 1.7), (4.1, 2.9)];
    for (i, (ax, ay)) in offsets.iter().enumerate() {
        let vx = ux * std::f64::consts::PI * (1.1 + 0.4 * i as f64) + ax + t * 0.6;
        let vy = uy * std::f64::consts::PI * (0.8 + 0.3 * i as f64) + ay + t * 0.4;
        v += 0.5 * (vx + vy).sin();
    }
    ((v / 3.0 + 0.5).clamp(0.0, 1.0)) * 0.85
}

/// Particles orbiting around the center, each with a dense core, a soft
/// glow, and ring contours.
fn orbits(x: f64, y: f64, cx: f64, cy: f64, t: f64) -> f64 {
    let mut v = 0.0;
    for i in 0..6usize {
        let f = i as f64;
        let ang = t * (0.55 + 0.1 * f) + f * 1.9;
        let r = cx * 0.55 * (0.25 + 0.15 * f);
        let dx = x - (cx + r * ang.cos());
        let dy = y - (cy + r * ang.sin() * 0.7);
        let d = (dx * dx + dy * dy).sqrt() / cx;
        v += 2.0 / (1.0 + d * d * 12.0); // dense core
        v += 0.4 * ((d - 0.55) / 0.12 + 1.0).clamp(0.0, 1.0); // glow edge
    }
    (v / 12.0).clamp(0.0, 1.0) * 0.9 + 0.05
}

/// Pulsing bars: beats with a heart-beat rhythm sweeping across the row,
/// each phase-shifted, plus a scanline ripple in the column direction.
fn pulses(ux: f64, uy: f64, t: f64) -> f64 {
    let beat = 0.5 + 0.5 * (t * 1.7).sin();
    let bar = ((ux + 0.5) * 7.0).fract() < 0.7;
    let amp = if bar { beat } else { beat * 0.25 };
    let v = amp + 0.3 * (uy * std::f64::consts::PI * 1.2 + t * 0.8).sin() + 0.05;
    v / 1.35
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Cell;

    fn grid(w: u16, h: u16) -> Grid {
        Grid::new(w, h, Cell::blank())
    }

    #[test]
    fn every_effect_fills_its_grid() {
        for &effect in AsciifyEffect::ALL {
            let mut g = grid(48, 14);
            effect.tick(0.0, &mut g);
            let mut nonblank = 0;
            for y in 0..g.height() {
                for x in 0..g.width() {
                    let c = g.get(x, y);
                    assert!(FILL.contains(&c.ch), "char {:?} outside the ramp", c.ch);
                    if c.ch != ' ' {
                        nonblank += 1;
                    }
                }
            }
            assert!(
                nonblank > g.width() as usize * 2,
                "{} left the grid too empty",
                effect.name()
            );
        }
    }

    #[test]
    fn scene_is_deterministic() {
        for &effect in AsciifyEffect::ALL {
            let mut a = grid(40, 10);
            let mut b = grid(40, 10);
            effect.tick(0.37, &mut a);
            effect.tick(0.37, &mut b);
            for y in 0..a.height() {
                for x in 0..a.width() {
                    assert_eq!(
                        a.get(x, y),
                        b.get(x, y),
                        "{} not deterministic",
                        effect.name()
                    );
                }
            }
        }
    }

    #[test]
    fn scene_animates() {
        for &effect in AsciifyEffect::ALL {
            let mut a = grid(40, 10);
            let mut b = grid(40, 10);
            effect.tick(0.0, &mut a);
            effect.tick(1.5, &mut b);
            let diff: usize = (0..a.width())
                .flat_map(|x| (0..a.height()).map(move |y| (x, y)))
                .filter(|&(x, y)| a.get(x, y) != b.get(x, y))
                .count();
            assert!(diff > 5, "{} does not animate", effect.name());
        }
    }

    #[test]
    fn empty_grid_is_a_no_op() {
        let mut g = grid(0, 0);
        AsciifyEffect::Wave.tick(1.0, &mut g);
    }

    #[test]
    fn next_effect_cycles() {
        let mut e = AsciifyEffect::Wave;
        for _ in 0..AsciifyEffect::ALL.len() {
            e = e.next();
        }
        assert_eq!(e, AsciifyEffect::Wave);
    }
}
