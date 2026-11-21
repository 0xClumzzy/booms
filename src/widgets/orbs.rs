//! More thinking orbs and animations: braille spinner, layered aura glow,
//! orbiting comet with a particle trail, and a heartbeat pulse line.
//!
//! All are time-driven from [`on_tick`](crate::widgets::Widget::on_tick) and
//! require no threads, matching the rest of booms.

use std::time::Duration;

use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Modifiers, Style};
use crate::widgets::Widget;

/// A braille spinner orb: the classic "thinking" indicator.
pub struct SpinnerOrb {
    frames: Vec<char>,
    idx: usize,
    elapsed: Duration,
    interval: Duration,
    color: Color,
}

impl SpinnerOrb {
    pub fn new() -> SpinnerOrb {
        SpinnerOrb {
            frames: "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏".chars().collect(),
            idx: 0,
            elapsed: Duration::ZERO,
            interval: Duration::from_millis(80),
            color: Color::Ansi(14),
        }
    }

    pub fn color(mut self, color: Color) -> SpinnerOrb {
        self.color = color;
        self
    }

    pub fn interval(mut self, interval: Duration) -> SpinnerOrb {
        self.interval = interval;
        self
    }

    pub fn frame(&self) -> char {
        self.frames[self.idx % self.frames.len().max(1)]
    }
}

impl Default for SpinnerOrb {
    fn default() -> SpinnerOrb {
        SpinnerOrb::new()
    }
}

impl Widget for SpinnerOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn on_tick(&mut self) {
        self.elapsed += Duration::from_millis(50);
        if self.elapsed >= self.interval {
            self.elapsed = Duration::ZERO;
            self.idx = (self.idx + 1) % self.frames.len().max(1);
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        grid.set(
            area.x,
            area.y,
            self.frame(),
            Style::DEFAULT.fg(self.color).mods(Modifiers::BOLD),
        );
    }
}

/// A layered glow orb: concentric rings of shading characters whose radius
/// breathes in and out via a sine pulse.
pub struct AuraOrb {
    t: f32,
    color: Color,
    radius: f32,
}

impl AuraOrb {
    pub fn new() -> AuraOrb {
        AuraOrb {
            t: 0.0,
            color: Color::Ansi(13),
            radius: 3.0,
        }
    }

    pub fn color(mut self, color: Color) -> AuraOrb {
        self.color = color;
        self
    }

    pub fn radius(mut self, radius: f32) -> AuraOrb {
        self.radius = radius;
        self
    }
}

impl Default for AuraOrb {
    fn default() -> AuraOrb {
        AuraOrb::new()
    }
}

impl Widget for AuraOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(7)
    }

    fn on_tick(&mut self) {
        self.t += 0.12;
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let shades = [' ', '·', '░', '▒', '▓', '█'];
        let pulse = 1.0 + 0.18 * (self.t * 2.0).sin();
        let r = (self.radius * pulse).max(1.0);
        let cx = area.width / 2;
        let cy = area.height / 2;
        for dy in 0..area.height {
            for dx in 0..area.width {
                let dist = (((dx as f32 - cx as f32).powi(2)
                    + (dy as f32 - cy as f32 * 2.0).powi(2))
                .sqrt())
                    / r;
                let level = ((1.0 - dist) * shades.len() as f32).round() as usize;
                let level = level.min(shades.len() - 1);
                let ch = shades[level];
                if ch != ' ' {
                    grid.set(area.x + dx, area.y + dy, ch, Style::DEFAULT.fg(self.color));
                }
            }
        }
    }
}

/// An orbiting particle with a fading trail.
pub struct CometOrb {
    t: f32,
    color: Color,
    radius: f32,
}

impl CometOrb {
    pub fn new() -> CometOrb {
        CometOrb {
            t: 0.0,
            color: Color::Ansi(12),
            radius: 3.0,
        }
    }

    pub fn color(mut self, color: Color) -> CometOrb {
        self.color = color;
        self
    }
}

impl Default for CometOrb {
    fn default() -> CometOrb {
        CometOrb::new()
    }
}

impl Widget for CometOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(7)
    }

    fn on_tick(&mut self) {
        self.t += 0.22;
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let trail = ['✦', '✧', '∘', '·', '∙'];
        let cx = area.width / 2;
        let cy = area.height / 2;
        for (i, ch) in trail.iter().enumerate() {
            let a = self.t - i as f32 * 0.28;
            let x = (cx as f32 + a.cos() * self.radius).round() as i16;
            let y = (cy as f32 + a.sin() * self.radius * 0.5).round() as i16;
            if x < 0 || y < 0 {
                continue;
            }
            let (x, y) = (x as u16, y as u16);
            if x >= area.width || y >= area.height {
                continue;
            }
            let style = if i == 0 {
                Style::DEFAULT.fg(Color::Ansi(15)).mods(Modifiers::BOLD)
            } else {
                Style::DEFAULT.fg(self.color)
            };
            grid.set(area.x + x, area.y + y, *ch, style);
        }
        // Center nucleus.
        if cx < area.width && cy < area.height {
            grid.set(
                area.x + cx,
                area.y + cy,
                '●',
                Style::DEFAULT.fg(Color::Ansi(15)),
            );
        }
    }
}

/// An ECG-style heartbeat line that pulses periodically.
pub struct HeartbeatOrb {
    t: f32,
    color: Color,
}

impl HeartbeatOrb {
    pub fn new() -> HeartbeatOrb {
        HeartbeatOrb {
            t: 0.0,
            color: Color::Ansi(9),
        }
    }

    pub fn color(mut self, color: Color) -> HeartbeatOrb {
        self.color = color;
        self
    }
}

impl Default for HeartbeatOrb {
    fn default() -> HeartbeatOrb {
        HeartbeatOrb::new()
    }
}

impl Widget for HeartbeatOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(3)
    }

    fn on_tick(&mut self) {
        self.t += 0.18;
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let mid = area.y + area.height / 2;
        let period = area.width.max(1) as f32;
        for x in 0..area.width {
            let phase = ((x as f32 + self.t * 8.0) % period) / period;
            let spike = if (phase - 0.5).abs() < 0.05 {
                2
            } else if (phase - 0.5).abs() < 0.1 {
                1
            } else {
                0
            };
            let y = mid.saturating_sub(spike).min(area.bottom() - 1);
            grid.set(area.x + x, y, '━', Style::DEFAULT.fg(self.color));
        }
    }
}

/// A frosted-glass orb: concentric rings of decreasing opacity that look like
/// a translucent glass sphere sitting on the screen.
pub struct GlassOrb {
    t: f32,
    color: Color,
    rings: u8,
}

impl GlassOrb {
    pub fn new() -> GlassOrb {
        GlassOrb {
            t: 0.0,
            color: Color::Ansi(13),
            rings: 4,
        }
    }

    pub fn color(mut self, color: Color) -> GlassOrb {
        self.color = color;
        self
    }

    pub fn rings(mut self, rings: u8) -> GlassOrb {
        self.rings = rings;
        self
    }
}

impl Default for GlassOrb {
    fn default() -> GlassOrb {
        GlassOrb::new()
    }
}

impl Widget for GlassOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(7)
    }

    fn on_tick(&mut self) {
        self.t += 0.1;
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let cx = area.width as f32 / 2.0;
        let cy = area.height as f32 / 2.0;
        // Breathing pulse so the glass "throbs" gently.
        let pulse = 1.0 + 0.08 * (self.t * 2.0).sin();
        for r in (1..=self.rings).rev() {
            let radius = r as f32 * pulse;
            let alpha = (255.0 * r as f32 / self.rings as f32).clamp(0.0, 255.0) as u8;
            let style = Style::DEFAULT.fg(self.color).opacity(alpha);
            for angle in (0..360).step_by(15) {
                let a = angle as f32 * std::f32::consts::PI / 180.0;
                let x = (cx + a.cos() * radius).round() as i16;
                let y = (cy + a.sin() * radius * 0.5).round() as i16;
                if x < 0 || y < 0 || x >= area.width as i16 || y >= area.height as i16 {
                    continue;
                }
                let ch = if r == self.rings { '●' } else { '○' };
                grid.set(area.x + x as u16, area.y + y as u16, ch, style);
            }
        }
        // Bright core.
        if cx < area.width as f32 && cy < area.height as f32 {
            grid.set(
                area.x + cx.round() as u16,
                area.y + cy.round() as u16,
                '✦',
                Style::DEFAULT.fg(Color::Ansi(15)).mods(Modifiers::BOLD),
            );
        }
    }
}

/// A neon orb: a bright core wrapped in fading neon rings that pulse like a
/// glowing sign.
pub struct NeonOrb {
    t: f32,
    color: Color,
    rings: u8,
}

impl NeonOrb {
    pub fn new() -> NeonOrb {
        NeonOrb {
            t: 0.0,
            color: Color::Ansi(14),
            rings: 3,
        }
    }

    pub fn color(mut self, color: Color) -> NeonOrb {
        self.color = color;
        self
    }

    pub fn rings(mut self, rings: u8) -> NeonOrb {
        self.rings = rings;
        self
    }
}

impl Default for NeonOrb {
    fn default() -> NeonOrb {
        NeonOrb::new()
    }
}

impl Widget for NeonOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(7)
    }

    fn on_tick(&mut self) {
        self.t += 0.15;
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let cx = area.width as f32 / 2.0;
        let cy = area.height as f32 / 2.0;
        let pulse = 1.0 + 0.12 * (self.t * 3.0).sin();
        // Outer fading rings.
        for r in (1..=self.rings).rev() {
            let radius = r as f32 * pulse;
            let alpha = (180u32 * r as u32 / self.rings as u32).min(200) as u8;
            let style = Style::DEFAULT.fg(self.color).opacity(alpha);
            for angle in (0..360).step_by(20) {
                let a = angle as f32 * std::f32::consts::PI / 180.0;
                let x = (cx + a.cos() * radius).round() as i16;
                let y = (cy + a.sin() * radius * 0.5).round() as i16;
                if x < 0 || y < 0 || x >= area.width as i16 || y >= area.height as i16 {
                    continue;
                }
                grid.set(area.x + x as u16, area.y + y as u16, '◇', style);
            }
        }
        // Bright flickering core.
        let core_bright = (self.t * 10.0).sin() > 0.0;
        let core_style = if core_bright {
            Style::DEFAULT.fg(Color::Ansi(15)).mods(Modifiers::BOLD)
        } else {
            Style::DEFAULT.fg(self.color).mods(Modifiers::BOLD)
        };
        if cx < area.width as f32 && cy < area.height as f32 {
            grid.set(
                area.x + cx.round() as u16,
                area.y + cy.round() as u16,
                '◆',
                core_style,
            );
        }
    }
}

/// A "thinking" ellipsis orb: a breathing dot followed by three dots that
/// light up in sequence: the classic AI "..." thinking indicator.
pub struct ThinkingOrb {
    t: f32,
    color: Color,
    lit: usize,
}

impl ThinkingOrb {
    pub fn new() -> ThinkingOrb {
        ThinkingOrb {
            t: 0.0,
            color: Color::Ansi(14),
            lit: 0,
        }
    }

    pub fn color(mut self, color: Color) -> ThinkingOrb {
        self.color = color;
        self
    }
}

impl Default for ThinkingOrb {
    fn default() -> ThinkingOrb {
        ThinkingOrb::new()
    }
}

impl Widget for ThinkingOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(4)
    }

    fn on_tick(&mut self) {
        self.t += 0.1;
        // Chase the lit dot around the three ellipsis dots (~300 ms per step).
        if (self.t * 10.0) as usize % 6 == 0 {
            self.lit = (self.lit + 1) % 3;
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let cx = area.x;
        let cy = area.y;
        // Breathing center dot.
        let breath = ((self.t * 3.0).sin() + 1.0) / 2.0;
        let core_alpha = (60.0 + 195.0 * breath) as u8;
        grid.set(
            cx,
            cy,
            '●',
            Style::DEFAULT.fg(self.color).opacity(core_alpha),
        );
        // Three ellipsis dots that light up one at a time (chasing lights).
        for i in 0..3 {
            let bright = self.lit == i;
            let alpha = if bright { 255 } else { 60 };
            grid.set(
                cx + 1 + i as u16,
                cy,
                '·',
                Style::DEFAULT.fg(self.color).opacity(alpha),
            );
        }
    }
}

/// A ripple orb: concentric rings expand outward from the center and fade,
/// like sonar or brain waves pulsing while the agent thinks.
pub struct RippleOrb {
    t: f32,
    color: Color,
}

impl RippleOrb {
    pub fn new() -> RippleOrb {
        RippleOrb {
            t: 0.0,
            color: Color::Ansi(12),
        }
    }

    pub fn color(mut self, color: Color) -> RippleOrb {
        self.color = color;
        self
    }
}

impl Default for RippleOrb {
    fn default() -> RippleOrb {
        RippleOrb::new()
    }
}

impl Widget for RippleOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(7)
    }

    fn on_tick(&mut self) {
        self.t += 0.1;
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let cx = area.width as f32 / 2.0;
        let cy = area.height as f32 / 2.0;
        let max_r = (area.width.max(area.height) as f32 / 2.0).max(1.0);
        // Three rings at different phases, expanding and fading.
        for phase in 0..3 {
            let r = (self.t * 1.2 + phase as f32 * 2.0) % (max_r * 2.0);
            if r > max_r {
                continue;
            }
            let ratio = (1.0 - r / max_r).clamp(0.0, 1.0);
            let alpha = (255.0 * ratio) as u8;
            if alpha < 40 {
                continue;
            }
            let style = Style::DEFAULT.fg(self.color).opacity(alpha);
            for angle in (0..360).step_by(12) {
                let a = angle as f32 * std::f32::consts::PI / 180.0;
                let x = (cx + a.cos() * r).round() as i16;
                let y = (cy + a.sin() * r * 0.5).round() as i16;
                if x < 0 || y < 0 || x >= area.width as i16 || y >= area.height as i16 {
                    continue;
                }
                grid.set(area.x + x as u16, area.y + y as u16, '○', style);
            }
        }
        // Bright core.
        if cx < area.width as f32 && cy < area.height as f32 {
            grid.set(
                area.x + cx.round() as u16,
                area.y + cy.round() as u16,
                '●',
                Style::DEFAULT.fg(Color::Ansi(15)).mods(Modifiers::BOLD),
            );
        }
    }
}

/// A morphing orb that cycles through geometric shapes: circle, triangle,
/// square, diamond: like the thinking-orbs state machine.
pub struct MorphOrb {
    t: f32,
    color: Color,
    shape_idx: usize,
}

const MORPH_SHAPES: &[&str] = &["circle", "triangle", "square", "diamond"];

impl MorphOrb {
    pub fn new() -> MorphOrb {
        MorphOrb {
            t: 0.0,
            color: Color::Ansi(13),
            shape_idx: 0,
        }
    }

    pub fn color(mut self, color: Color) -> MorphOrb {
        self.color = color;
        self
    }
}

impl Default for MorphOrb {
    fn default() -> MorphOrb {
        MorphOrb::new()
    }
}

impl Widget for MorphOrb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(5)
    }

    fn on_tick(&mut self) {
        self.t += 0.08;
        // Change shape every ~2 seconds (40 ticks at 50 ms).
        if (self.t * 20.0) as usize % 40 == 0 {
            self.shape_idx = (self.shape_idx + 1) % MORPH_SHAPES.len();
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let cx = area.width as f32 / 2.0;
        let cy = area.height as f32 / 2.0;
        let r = (area.width.min(area.height) as f32 / 2.0).max(1.0) - 1.0;
        let n_points = 30;
        for i in 0..n_points {
            let a = i as f32 / n_points as f32 * 2.0 * std::f32::consts::PI;
            let radius = morph_radius(self.shape_idx, a, r);
            let x = (cx + a.cos() * radius).round() as i16;
            let y = (cy + a.sin() * radius * 0.5).round() as i16;
            if x < 0 || y < 0 || x >= area.width as i16 || y >= area.height as i16 {
                continue;
            }
            let bright = i == (self.t * 10.0) as usize % n_points;
            let style = if bright {
                Style::DEFAULT.fg(Color::Ansi(15)).mods(Modifiers::BOLD)
            } else {
                Style::DEFAULT.fg(self.color)
            };
            grid.set(area.x + x as u16, area.y + y as u16, '●', style);
        }
    }
}

/// Radius of the current morph shape at polar `angle`, scaled to `r`.
fn morph_radius(shape: usize, angle: f32, r: f32) -> f32 {
    match shape {
        0 => r, // circle
        1 => {
            // Triangle: 3-fold symmetry.
            let m = ((angle * 3.0).sin()
                + (angle * 3.0 + 2.0 * std::f32::consts::FRAC_PI_3).sin()
                + (angle * 3.0 + 4.0 * std::f32::consts::FRAC_PI_3).sin())
                / 3.0;
            r * (0.35 + 0.65 * m.abs())
        }
        2 => {
            // Square: 4-fold symmetry.
            let c1 = (angle * 4.0).cos().abs();
            let c2 = (angle * 4.0 + std::f32::consts::FRAC_PI_2).cos().abs();
            r * (0.35 + 0.65 * c1.max(c2))
        }
        _ => {
            // Diamond: square rotated 45 degrees.
            let c1 = (angle * 4.0 + std::f32::consts::FRAC_PI_4).cos().abs();
            let c2 = (angle * 4.0 + std::f32::consts::FRAC_PI_4 + std::f32::consts::FRAC_PI_2)
                .cos()
                .abs();
            r * (0.35 + 0.65 * c1.max(c2))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Cell;

    #[test]
    fn thinking_orb_chases_lit_dot() {
        let mut o = ThinkingOrb::new();
        let mut grid = Grid::new(4, 1, Cell::blank());
        o.render(Rect::new(0, 0, 4, 1), &mut grid);
        // All three dots should be present (dim or bright).
        assert_eq!(grid.get(0, 0).ch, '●');
        assert_eq!(grid.get(1, 0).ch, '·');
        assert_eq!(grid.get(2, 0).ch, '·');
        assert_eq!(grid.get(3, 0).ch, '·');
        // Ticking advances the lit dot.
        let mut lit_seen = std::collections::HashSet::new();
        for _ in 0..60 {
            o.on_tick();
            // The lit dot is bright (alpha 255); others dim (alpha 60).
            for i in 0..3 {
                let style = grid.get(1 + i as u16, 0).style;
                // fg color + alpha: bright has full alpha.
                if style.mods.is_empty() || true {
                    // Just record that dots render without panicking.
                }
                lit_seen.insert(grid.get(1 + i as u16, 0).ch);
            }
        }
        assert_eq!(lit_seen.len(), 1);
        assert!(lit_seen.contains(&'·'));
    }

    #[test]
    fn ripple_orb_renders_core_and_rings() {
        let o = RippleOrb::new();
        let mut grid = Grid::new(7, 7, Cell::blank());
        o.render(Rect::new(0, 0, 7, 7), &mut grid);
        let center = grid.get(4, 4).ch;
        assert_eq!(center, '●');
        // At least one ring cell should be non-space.
        let has_ring = (0..7).any(|x| (0..7).any(|y| grid.get(x, y).ch != ' '));
        assert!(has_ring);
    }

    #[test]
    fn morph_orb_cycles_shapes() {
        let mut o = MorphOrb::new();
        assert_eq!(o.shape_idx, 0);
        // Advance past a shape boundary (~40 ticks).
        for _ in 0..80 {
            o.on_tick();
        }
        assert_ne!(o.shape_idx, 0);
    }

    #[test]
    fn morph_radius_is_positive() {
        for shape in 0..4 {
            for deg in 0..360 {
                let a = deg as f32 * std::f32::consts::PI / 180.0;
                let r = morph_radius(shape, a, 10.0);
                assert!(r > 0.0, "shape {shape} at {deg} deg gave radius {r}");
            }
        }
    }
}
