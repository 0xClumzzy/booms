//! Animation and transition primitives.
//!
//! Ported from thinking-orbs' smoothstep easing and time-driven frames:
//! [`Anim`] interpolates a value from `from` to `to` over a duration,
//! driven by [`Anim::update`]. Widgets advance animations from
//! [`on_tick`](crate::widgets::Widget::on_tick), so transitions run at the
//! app's tick rate without extra threads.

use std::time::{Duration, Instant};

/// Easing curves for [`Anim`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Easing {
    /// `t`: the default, unchanged.
    #[default]
    Linear,
    /// The thinking-orbs smoothstep: `t * t * (3 - 2 * t)`.
    SmoothStep,
    /// `1 - cos(t * PI / 2)`: starts slow.
    SineIn,
    /// `sin(t * PI / 2)`: ends slow.
    SineOut,
    /// `(1 - cos(t * PI)) / 2`: slow at both ends.
    SineInOut,
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::SmoothStep => t * t * (3.0 - 2.0 * t),
            Easing::SineIn => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
            Easing::SineOut => (t * std::f32::consts::FRAC_PI_2).sin(),
            Easing::SineInOut => (1.0 - (t * std::f32::consts::PI).cos()) / 2.0,
        }
    }
}

/// Interpolatable values for [`Anim`]: colors, styles, rects, numbers.
pub trait Interpolate: Clone {
    /// The value at `t == 0`.
    fn lerp(&self, to: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn lerp(&self, to: &f32, t: f32) -> f32 {
        self + (to - self) * t
    }
}

impl Interpolate for f64 {
    fn lerp(&self, to: &f64, t: f32) -> f64 {
        self + (to - self) * t as f64
    }
}

impl Interpolate for u8 {
    fn lerp(&self, to: &u8, t: f32) -> u8 {
        let v = (*self as f32 + (*to as f32 - *self as f32) * t).round();
        v.clamp(0.0, u8::MAX as f32) as u8
    }
}

fn lerp_num(a: u16, b: u16, t: f32) -> u16 {
    if a == b {
        return a;
    }
    let v = (a as f32 + (b as f32 - a as f32) * t).round();
    v.clamp(0.0, u16::MAX as f32) as u16
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    if a == b {
        return a;
    }
    let v = (a as f32 + (b as f32 - a as f32) * t).round();
    v.clamp(0.0, u8::MAX as f32) as u8
}

impl Interpolate for crate::style::Color {
    fn lerp(&self, to: &crate::style::Color, t: f32) -> crate::style::Color {
        use crate::style::Color;
        match (self, to) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => Color::Rgb(
                lerp_u8(*r1, *r2, t),
                lerp_u8(*g1, *g2, t),
                lerp_u8(*b1, *b2, t),
            ),
            _ if t < 0.5 => *self,
            _ => *to,
        }
    }
}

impl Interpolate for crate::style::Style {
    fn lerp(&self, to: &crate::style::Style, t: f32) -> crate::style::Style {
        crate::style::Style {
            fg: self.fg.lerp(&to.fg, t),
            bg: self.bg.lerp(&to.bg, t),
            mods: if self.mods == to.mods || t < 0.5 {
                self.mods
            } else {
                to.mods
            },
            opacity: lerp_num(self.opacity as u16, to.opacity as u16, t) as u8,
        }
    }
}

impl Interpolate for crate::layout::Rect {
    fn lerp(&self, to: &crate::layout::Rect, t: f32) -> crate::layout::Rect {
        crate::layout::Rect::new(
            lerp_num(self.x, to.x, t),
            lerp_num(self.y, to.y, t),
            lerp_num(self.width, to.width, t),
            lerp_num(self.height, to.height, t),
        )
    }
}

/// A one-shot transition from `from` to `to` over `duration`.
///
/// Advance it from [`on_tick`](crate::widgets::Widget::on_tick) with
/// [`Anim::update`] and read [`Anim::value`] to render:
///
/// ```
/// use std::time::Duration;
/// use booms::anim::{Anim, Easing};
/// use booms::style::Color;
///
/// # let mut fade =
/// Anim::new(Color::Rgb(30, 30, 30), Color::Rgb(210, 210, 210), Duration::from_millis(700))
///     .easing(Easing::SineInOut);
/// fade.update();
/// # assert!(*fade.value() != Color::Default);
/// ```
#[derive(Clone, Debug)]
pub struct Anim<T> {
    from: T,
    to: T,
    duration: Duration,
    start: Instant,
    easing: Easing,
    value: T,
    finished: bool,
}

impl<T: Interpolate> Anim<T> {
    pub fn new(from: T, to: T, duration: Duration) -> Anim<T> {
        Anim {
            value: from.clone(),
            from,
            to,
            duration,
            start: Instant::now(),
            easing: Easing::default(),
            finished: false,
        }
    }

    pub fn easing(mut self, easing: Easing) -> Anim<T> {
        self.easing = easing;
        self
    }

    /// The value the animation starts from.
    pub fn from(&self) -> &T {
        &self.from
    }

    /// The value the animation ends at.
    pub fn to(&self) -> &T {
        &self.to
    }

    /// The current interpolated value.
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Restart the animation from the beginning.
    pub fn restart(&mut self) {
        self.start = Instant::now();
        self.finished = false;
        self.value = self.from.clone();
    }

    /// Advance the animation by wall clock time; returns the eased
    /// progress in `0..=1`.
    pub fn update(&mut self) -> f32 {
        if self.finished {
            return 1.0;
        }
        let t = if self.duration.is_zero() {
            1.0
        } else {
            (self.start.elapsed().as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
        };
        self.value = self.from.lerp(&self.to, t);
        let progress = self.easing.apply(t);
        if t >= 1.0 {
            self.finished = true;
        }
        progress
    }
}

/// A spring-physics transition between two states: a port of the
/// [`liquid-gooey`] spring engine.
///
/// Unlike [`Anim`], which follows a fixed duration, a spring follows
/// physical constants: it overshoots and settles with a physical feel.
/// Advance it from [`on_tick`](crate::widgets::Widget::on_tick) with
/// [`Spring::tick`].
///
/// [`liquid-gooey`]: Libraries.dev/packages/liquid-gooey: used as a
/// reference in this framework's workspace.
///
/// ```
/// use booms::anim::Spring;
///
/// let mut spring = Spring::new(0.0, 1.0, 500.0, 110.0, 1.0);
/// while !spring.is_settled() {
///     spring.tick();
/// }
/// assert!((spring.value() - 1.0).abs() < 0.05);
/// ```
#[derive(Clone, Debug)]
pub struct Spring {
    x: f32,
    v: f32,
    target: f32,
    settled: bool,
    settled_since: Option<f32>,
    t: f32,
    stiffness: f32,
    damping: f32,
    mass: f32,
    dt: f32,
}

impl Spring {
    const DT: f32 = 1.0 / 120.0;

    /// Create a spring, simulating to completion so it starts near
    /// `target` with zero velocity.
    pub fn new(x: f32, target: f32, stiffness: f32, damping: f32, mass: f32) -> Spring {
        let mut spring = Spring {
            x,
            v: 0.0,
            target,
            settled: false,
            settled_since: None,
            t: 0.0,
            stiffness,
            damping,
            mass,
            dt: Spring::DT,
        };
        for _ in 0..10_000 {
            if spring.settled {
                break;
            }
            spring.tick();
        }
        spring
    }

    /// The spring's current position.
    pub fn value(&self) -> f32 {
        self.x
    }

    /// The spring's target.
    pub fn target(&self) -> f32 {
        self.target
    }

    /// Move the spring toward `target`; if it was settled it snaps to it.
    pub fn set_target(&mut self, target: f32) {
        if self.settled {
            self.x = target;
            self.v = 0.0;
        }
        self.target = target;
        self.settled = false;
        self.settled_since = None;
    }

    /// True once the spring is at its target with (nearly) zero velocity
    /// and has stayed there for a short settling grace period.
    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// Integrate one tick (`1/120` s of simulated time).
    pub fn tick(&mut self) {
        let a = (-self.stiffness * (self.x - self.target) - self.damping * self.v) / self.mass;
        self.v += a * self.dt;
        self.x += self.v * self.dt;
        self.t += self.dt;
        if (self.x - self.target).abs() < 0.001 && self.v.abs() < 0.02 {
            match self.settled_since {
                None => self.settled_since = Some(self.t),
                Some(since) if self.t - since >= 0.064 => self.settled = true,
                Some(_) => {}
            }
        } else {
            self.settled_since = None;
        }
    }
}
