//! Crystal orb: a faceted gem with a sweeping shine and a slow color
//! shift. Built from unicode geometric characters painted cell-by-cell,
//! animated with eased [`Anim`] transitions.

use std::time::Duration;

use crate::anim::{Anim, Easing};
use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Modifiers, Style};
use crate::widgets::Widget;

/// The crystal shape, one row per entry. Each row is a string of facet
/// characters (spaces are background). All rows share the same width so
/// the orb stays centered.
const CRYSTAL_ROWS: &[&str] = &[
    "   ✦   ",
    "  ◈◇◆  ",
    " ◇◆◆◇ ",
    " ◆◆●◆◆ ",
    " ◇◆◆◇ ",
    "  ◈◇◆  ",
    "   ✧   ",
];

/// Base color for each facet character position, mirrored across rows.
const FACET_COLORS: &[Color] = &[
    Color::Ansi(14), // bright yellow (shine)
    Color::Ansi(12), // blue
    Color::Ansi(14), // bright yellow
    Color::Ansi(15), // white
    Color::Ansi(12), // blue
    Color::Ansi(14), // bright yellow
    Color::Ansi(15), // white
];

/// A crystal orb widget.
///
/// A sweeping shine moves across the core row while the facet colors
/// slowly shift between cyan and blue. The orb bobs slightly via a
/// spring-physics offset.
pub struct CrystalOrb {
    /// Shine position across the orb width, 0.0 → 1.0.
    shine: Anim<f32>,
    /// Color-shift phase for the facets.
    hue: Anim<f32>,
    /// Vertical bob offset (sub-pixel, applied as a row shift when it
    /// crosses integer thresholds).
    bob: f32,
}

impl CrystalOrb {
    /// Create a crystal orb.
    pub fn new() -> CrystalOrb {
        CrystalOrb {
            shine: Anim::new(0.0, 1.0, Duration::from_millis(1600)).easing(Easing::SineInOut),
            hue: Anim::new(0.0, 1.0, Duration::from_millis(3000)).easing(Easing::Linear),
            bob: 0.0,
        }
    }
}

impl Default for CrystalOrb {
    fn default() -> CrystalOrb {
        CrystalOrb::new()
    }
}

impl Widget for CrystalOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(CRYSTAL_ROWS.len() as u16)
    }

    fn on_tick(&mut self) {
        if self.shine.update() >= 1.0 {
            self.shine.restart();
        }
        if self.hue.update() >= 1.0 {
            self.hue.restart();
        }
        // Slow vertical bob: drift up and down over time.
        self.bob += 0.04;
        if self.bob > 1.0 {
            self.bob = -1.0;
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let shine_x = self.shine.value() * (CRYSTAL_ROWS[0].len() - 1) as f32;
        let hue = self.hue.value();

        // Core row index (the row with the ● core).
        let core_row = 3;

        for (row_i, row) in CRYSTAL_ROWS.iter().enumerate() {
            // Apply the subtle bob: skip drawing this row when bob pushes
            // it off-screen, or draw it at a shifted row near the bob.
            let bob_offset = if self.bob >= 0.0 && row_i == core_row {
                // The core row bobs by one row when bob crosses a threshold.
                if self.bob > 0.5 { 1 } else { 0 }
            } else {
                0
            };
            let y = area.y + row_i as u16 + bob_offset;
            if y >= area.bottom() {
                continue;
            }

            for (col_i, ch) in row.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let x = area.x + col_i as u16;
                if x >= area.right() {
                    continue;
                }

                // Base facet color, shifted by hue.
                let base = FACET_COLORS[col_i % FACET_COLORS.len()];
                let color = facet_color(base, *hue);

                // Sweeping shine: facets near the shine position on the
                // core row glow bright white and bold.
                let mut style = Style::DEFAULT.fg(color);
                if row_i == core_row {
                    let dist = (col_i as f32 - shine_x).abs();
                    if dist <= 1.0 {
                        style = style.fg(Color::Ansi(15)).mods(Modifiers::BOLD);
                    } else if dist <= 2.0 {
                        style = style.fg(Color::Ansi(15));
                    }
                }

                grid.set(x, y, ch, style);
            }
        }
    }
}

/// Shift a base facet color toward a tint based on the hue phase.
fn facet_color(base: Color, hue: f32) -> Color {
    match base {
        Color::Ansi(_n) => {
            // Cycle between the base color and a cyan/blue tint.
            let tinted = if hue < 0.33 {
                Color::Ansi(14) // bright yellow
            } else if hue < 0.66 {
                Color::Ansi(12) // blue
            } else {
                Color::Ansi(6) // cyan
            };
            if hue.fract() < 0.5 { base } else { tinted }
        }
        c => c,
    }
}
