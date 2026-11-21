//! Asciify widget: an animated ASCII scene filling its area.

use crate::asciify::AsciifyEffect;
use crate::grid::{Cell, Grid};
use crate::layout::{Constraint, Rect};
use crate::widgets::Widget;

/// Seconds of scene time advanced per tick at the default 50 ms tick rate.
const DEFAULT_SPEED: f64 = 0.05;

/// A widget that renders an animated [`AsciifyEffect`] scene, filling its
/// whole area.
///
/// ```
/// use booms::asciify::AsciifyEffect;
/// use booms::widgets::Asciify;
///
/// let scene = Asciify::new(AsciifyEffect::Wave);
/// ```
pub struct Asciify {
    /// The scene kind.
    effect: AsciifyEffect,
    /// Elapsed scene time in seconds.
    t: f64,
    /// Scene seconds advanced per tick.
    speed: f64,
}

impl Asciify {
    /// Create a scene widget for `effect`.
    pub fn new(effect: AsciifyEffect) -> Asciify {
        Asciify {
            effect,
            t: 0.0,
            speed: DEFAULT_SPEED,
        }
    }

    /// Set the scene kind.
    pub fn set_effect(&mut self, effect: AsciifyEffect) {
        self.effect = effect;
    }

    /// The scene kind.
    pub fn effect(&self) -> AsciifyEffect {
        self.effect
    }

    /// Advance to the next scene kind, wrapping around.
    pub fn cycle_effect(&mut self) {
        self.effect = self.effect.next();
    }

    /// Set the scene seconds advanced per tick.
    pub fn with_speed(mut self, speed: f64) -> Asciify {
        self.speed = speed;
        self
    }

    /// The elapsed scene time in seconds.
    pub fn time(&self) -> f64 {
        self.t
    }

    /// Jump the scene to an absolute time.
    pub fn set_time(&mut self, t: f64) {
        self.t = t;
    }
}

impl Widget for Asciify {
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    fn on_tick(&mut self) {
        self.t += self.speed;
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let (w, h) = (area.width, area.height);
        if w == 0 || h == 0 {
            return;
        }
        // Render the scene into a sub-grid, then copy it into place.
        let mut scene = Grid::new(w, h, Cell::blank());
        self.effect.tick(self.t, &mut scene);
        for y in 0..h {
            for x in 0..w {
                let cell = scene.get(x, y);
                grid.set(area.x + x, area.y + y, cell.ch, cell.style);
            }
        }
    }
}
