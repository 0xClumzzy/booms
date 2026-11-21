//! Thinking orbs: the terminal port of Libraries.dev `thinking-orbs`.
//!
//! Nine animated states for AI & agent "thinking" indicators, ported from the
//! React library `thinking-orbs` (Jakub Antalik, MIT). Each state is a
//! distinct dot-field animation rendered on the terminal grid: no threads,
//! no WebGL, just `on_tick`-driven math like the rest of booms.
//!
//! ## States (mirroring the React props)
//!
//! ```text
//! ThinkingOrb::new().state(OrbState::Working)     // particles on tilted orbits
//! ThinkingOrb::new().state(OrbState::Searching)   // scan meridian sweeps a dotted globe
//! ThinkingOrb::new().state(OrbState::Solving)     // bands scramble, then click back solved
//! ThinkingOrb::new().state(OrbState::Listening)   // a waveform rolls through the rings
//! ThinkingOrb::new().state(OrbState::Connecting)  // a constellation wires itself
//! ThinkingOrb::new().state(OrbState::Weaving)     // three strands plait around the sphere
//! ThinkingOrb::new().state(OrbState::Composing)   // an undulating multi-band sash
//! ThinkingOrb::new().state(OrbState::Breathing)   // a ring slowly morphing
//! ThinkingOrb::new().state(OrbState::Shaping)     // dotted outline: circle → triangle → square
//! ```
//!
//! ## Props (mirroring the React API)
//!
//! - `state`: one of the nine animated states (default `Working`)
//! - `size`: `Size64` (chat-avatar scale) or `Size20` (inline-text scale)
//! - `speed`: multiplies the animation clock, default 1
//! - `dark`: picks light or dark ink tuning (default `true`)
//! - `paused`: freezes the animation on the current frame
//!
//! ## Example
//!
//! ```no_run
//! use booms::app::App;
//! use booms::widgets::{OrbState, OrbSize, ThinkingOrb};
//!
//! let mut app = App::new()?;
//! app.add(
//!     ThinkingOrb::new()
//!         .state(OrbState::Searching)
//!         .size(OrbSize::Size64)
//!         .speed(1.5)
//!         .dark(true),
//! );
//! app.run()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::grid::Grid;
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Style};
use crate::widgets::Widget;

/// One animated state from thinking-orbs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OrbState {
    #[default]
    Working,
    Searching,
    Solving,
    Listening,
    Connecting,
    Weaving,
    Composing,
    Breathing,
    Shaping,
}

impl OrbState {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrbState::Working => "working",
            OrbState::Searching => "searching",
            OrbState::Solving => "solving",
            OrbState::Listening => "listening",
            OrbState::Connecting => "connecting",
            OrbState::Weaving => "weaving",
            OrbState::Composing => "composing",
            OrbState::Breathing => "breathing",
            OrbState::Shaping => "shaping",
        }
    }

    /// The per-state aria-label used by the React component.
    pub fn label(&self) -> &'static str {
        match self {
            OrbState::Working => "Working",
            OrbState::Searching => "Searching",
            OrbState::Solving => "Solving",
            OrbState::Listening => "Listening",
            OrbState::Connecting => "Connecting",
            OrbState::Weaving => "Weaving",
            OrbState::Composing => "Composing",
            OrbState::Breathing => "Breathing",
            OrbState::Shaping => "Shaping",
        }
    }
}

/// Size presets from thinking-orbs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OrbSize {
    #[default]
    Size64, // chat-avatar scale: more dots, larger radii
    Size20, // inline-text scale: fewer dots, smaller radii
}

/// A thinking orb widget. Time-driven from [`Widget::on_tick`].
pub struct ThinkingOrb {
    state: OrbState,
    size: OrbSize,
    speed: f32,
    dark: bool,
    paused: bool,
    t: f32,
}

impl ThinkingOrb {
    pub fn new() -> Self {
        Self {
            state: OrbState::default(),
            size: OrbSize::default(),
            speed: 1.0,
            dark: true,
            paused: false,
            t: 0.0,
        }
    }

    pub fn state(mut self, state: OrbState) -> Self {
        self.state = state;
        self
    }

    pub fn size(mut self, size: OrbSize) -> Self {
        self.size = size;
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn dark(mut self, dark: bool) -> Self {
        self.dark = dark;
        self
    }

    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }

    /// Current animation clock (advanced every tick unless paused).
    pub fn time(&self) -> f32 {
        self.t
    }
}

impl Default for ThinkingOrb {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ThinkingOrb {
    fn constraint(&self) -> Constraint {
        // Square area: 9×9 cells for avatar scale, 5×5 for inline scale.
        Constraint::Exact(if self.size == OrbSize::Size64 { 9 } else { 5 })
    }

    fn on_tick(&mut self) {
        if self.paused {
            return;
        }
        self.t += 0.05 * self.speed;
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let opts = resolve_preset(self.state, self.size);
        let clock = self.t * opts.speed;
        let frame = mode_frame(self.state.mode(), area.width as f32, clock, &opts.opts);

        let size_px = match self.size {
            OrbSize::Size64 => 64.0,
            OrbSize::Size20 => 20.0,
        };
        render_frame(area, grid, &frame, self.dark, size_px);
    }
}

/// Resolved draw options for one (state, size) pair: ported from thinking-orbs
/// `presets.ts`.
#[derive(Clone)]
struct Preset {
    speed: f32,
    opts: ModeOpts,
}

#[derive(Clone, Default)]
struct ModeOpts {
    // Sphere lattice (globe / rubik / wave)
    lat_rings: Option<f32>,
    lon_density: Option<f32>,
    rings: Option<f32>,
    // Orbits
    orbit_n: Option<f32>,
    ghost_n: Option<f32>,
    particles: Option<f32>,
    ghost_r: Option<f32>,
    ghost_a: Option<f32>,
    part_r: Option<f32>,
    part_r_depth: Option<f32>,
    // Rubik
    move_count: Option<f32>,
    r_active: Option<f32>,
    // Web
    node_n: Option<f32>,
    thr: Option<f32>,
    signals: Option<f32>,
    node_r: Option<f32>,
    node_r_depth: Option<f32>,
    // Braid / ribbon
    strand_n: Option<f32>,
    turns: Option<f32>,
    lanes: Option<f32>,
    segs: Option<f32>,
    band_mul: Option<f32>,
    wob_mul: Option<f32>,
    face_on: Option<f32>,
    spin: Option<f32>,
    // Morph
    icon_d: Option<f32>,
    r_dot: Option<f32>,
    spread: Option<f32>,
    // Globe extra
    scan_mul: Option<f32>,
    dim_base: Option<f32>,
    // Shared
    r_base: Option<f32>,
    r_depth: Option<f32>,
    r_boost: Option<f32>,
    ink_far: Option<f32>,
    ink_span: Option<f32>,
    rs_pow: Option<f32>,
    r_min: Option<f32>,
}

/// One projected dot.
#[derive(Clone, Copy, Debug)]
struct Dot {
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    white: f32,
    a: f32,
}

/// One stroked edge (connecting state).
#[derive(Clone, Copy, Debug)]
struct Line {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    white: f32,
    a: f32,
}

/// A finished frame: dots z-sorted far→near, lines ready to draw.
struct Frame {
    dots: Vec<Dot>,
    lines: Vec<Line>,
}

/// Map state → mode key (ported from `presets.ts` STATE_TO_MODE).
impl OrbState {
    fn mode(&self) -> ModeKey {
        match self {
            OrbState::Working => ModeKey::Orbits,
            OrbState::Searching => ModeKey::Globe,
            OrbState::Solving => ModeKey::Rubik,
            OrbState::Listening => ModeKey::Wave,
            OrbState::Connecting => ModeKey::Web,
            OrbState::Weaving => ModeKey::Braid,
            OrbState::Composing => ModeKey::Ribbon,
            OrbState::Breathing => ModeKey::Ring,
            OrbState::Shaping => ModeKey::Morph,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModeKey {
    Orbits,
    Globe,
    Rubik,
    Wave,
    Web,
    Braid,
    Ribbon,
    Ring,
    Morph,
}

/// Resolve a (state, size) pair to its mode + fully-scaled draw options.
fn resolve_preset(state: OrbState, size: OrbSize) -> Preset {
    let mode = state.mode();
    let (speed, opts, _extra_speed_mul) = match (mode, size) {
        // working → orbits
        (ModeKey::Orbits, OrbSize::Size64) => (1.885, base_orbits(), 1.0),
        (ModeKey::Orbits, OrbSize::Size20) => (3.9, scale_counts(base_orbits(), 0.238), 1.0),
        // searching → globe
        (ModeKey::Globe, OrbSize::Size64) => {
            let mut o = base_globe();
            o.scan_mul = Some(4.08);
            o.dim_base = Some(0.45);
            (2.015, scale_counts(o, 0.42), 1.0)
        }
        (ModeKey::Globe, OrbSize::Size20) => {
            let mut o = base_globe();
            o.scan_mul = Some(4.335);
            o.dim_base = Some(0.45);
            (2.665, scale_counts(o, 0.105), 1.0)
        }
        // solving → rubik
        (ModeKey::Rubik, OrbSize::Size64) => (1.82, scale_counts(base_rubik(), 0.35), 1.0),
        (ModeKey::Rubik, OrbSize::Size20) => (1.95, scale_counts(base_rubik(), 0.088), 1.0),
        // listening → wave
        (ModeKey::Wave, OrbSize::Size64) => (4.388, scale_counts(base_wave(), 0.341), 1.0),
        (ModeKey::Wave, OrbSize::Size20) => (3.998, scale_counts(base_wave(), 0.105), 1.0),
        // connecting → web
        (ModeKey::Web, OrbSize::Size64) => (3.315, scale_counts(base_web(), 1.35), 1.0),
        (ModeKey::Web, OrbSize::Size20) => (6.63, scale_counts(base_web(), 0.25), 1.0),
        // weaving → braid
        (ModeKey::Braid, OrbSize::Size64) => (1.625, scale_counts(base_braid(), 0.5), 1.0),
        (ModeKey::Braid, OrbSize::Size20) => (2.75, scale_counts(base_braid(), 0.1125), 1.0),
        // composing → ribbon
        (ModeKey::Ribbon, OrbSize::Size64) => {
            let mut o = base_ribbon();
            o.spin = Some(0.0);
            o.band_mul = Some(3.9);
            o.wob_mul = Some(1.0);
            (2.34, scale_counts(o, 0.25), 1.0)
        }
        (ModeKey::Ribbon, OrbSize::Size20) => {
            let mut o = base_ribbon();
            o.spin = Some(0.0);
            o.band_mul = Some(4.94);
            o.wob_mul = Some(1.0);
            (3.12, scale_counts(o, 0.051), 1.0)
        }
        // breathing → ring (face-on ribbon)
        (ModeKey::Ring, OrbSize::Size64) => {
            let mut o = base_ribbon();
            o.ghost_n = Some(0.0);
            o.face_on = Some(1.0);
            o.band_mul = Some(3.627);
            o.wob_mul = Some(0.368);
            (3.24, scale_counts(o, 0.25), 1.0)
        }
        (ModeKey::Ring, OrbSize::Size20) => {
            let mut o = base_ribbon();
            o.ghost_n = Some(0.0);
            o.face_on = Some(1.0);
            o.band_mul = Some(3.968);
            o.wob_mul = Some(0.565);
            (3.78, scale_counts(o, 0.028), 1.0)
        }
        // shaping → morph
        (ModeKey::Morph, OrbSize::Size64) => (2.405, base_morph(), 1.0),
        (ModeKey::Morph, OrbSize::Size20) => (2.08, base_morph(), 1.0),
    };

    // Apply radius scaling for the size multiplier.
    let size_mul = match (mode, size) {
        (_, OrbSize::Size64) => 1.0,
        (ModeKey::Orbits, OrbSize::Size20) => 2.4,
        (ModeKey::Globe, OrbSize::Size20) => 1.75,
        (ModeKey::Rubik, OrbSize::Size20) => 1.9,
        (ModeKey::Wave, OrbSize::Size20) => 1.6,
        (ModeKey::Web, OrbSize::Size20) => 1.52,
        (ModeKey::Braid, OrbSize::Size20) => 1.36,
        (ModeKey::Ribbon, OrbSize::Size20) => 1.073,
        (ModeKey::Ring, OrbSize::Size20) => 1.622,
        (ModeKey::Morph, OrbSize::Size20) => 1.011,
    };
    let opts = scale_radii(opts, size_mul);

    Preset { speed, opts }
}

// --- Base profiles (ported from `profiles.ts` BASE_PROFILES) ------------

fn base_orbits() -> ModeOpts {
    ModeOpts {
        orbit_n: Some(12.0),
        ghost_n: Some(40.0),
        particles: Some(3.0),
        ghost_r: Some(0.9),
        ghost_a: Some(0.5),
        part_r: Some(1.2),
        part_r_depth: Some(1.6),
        rs_pow: Some(0.6),
        r_min: Some(0.3),
        ..ModeOpts::default()
    }
}

fn base_globe() -> ModeOpts {
    ModeOpts {
        lat_rings: Some(17.0),
        lon_density: Some(44.0),
        r_base: Some(0.6),
        r_depth: Some(1.7),
        r_boost: Some(1.0),
        ink_far: Some(0.62),
        ink_span: Some(0.54),
        rs_pow: Some(0.6),
        r_min: Some(0.3),
        ..ModeOpts::default()
    }
}

fn base_rubik() -> ModeOpts {
    ModeOpts {
        lat_rings: Some(15.0),
        lon_density: Some(40.0),
        move_count: Some(14.0),
        r_base: Some(0.6),
        r_depth: Some(1.7),
        r_active: Some(0.3),
        ink_far: Some(0.62),
        ink_span: Some(0.54),
        rs_pow: Some(0.6),
        r_min: Some(0.3),
        ..ModeOpts::default()
    }
}

fn base_wave() -> ModeOpts {
    ModeOpts {
        rings: Some(15.0),
        lon_density: Some(40.0),
        r_base: Some(0.6),
        r_depth: Some(1.7),
        rs_pow: Some(0.6),
        r_min: Some(0.3),
        ..ModeOpts::default()
    }
}

fn base_web() -> ModeOpts {
    ModeOpts {
        node_n: Some(30.0),
        thr: Some(0.72),
        signals: Some(5.0),
        node_r: Some(1.4),
        node_r_depth: Some(1.8),
        rs_pow: Some(0.6),
        r_min: Some(0.3),
        ..ModeOpts::default()
    }
}

fn base_braid() -> ModeOpts {
    ModeOpts {
        strand_n: Some(52.0),
        turns: Some(3.0),
        ghost_n: Some(150.0),
        r_base: Some(1.2),
        r_depth: Some(1.8),
        rs_pow: Some(0.6),
        r_min: Some(0.3),
        ..ModeOpts::default()
    }
}

fn base_ribbon() -> ModeOpts {
    ModeOpts {
        lanes: Some(5.0),
        segs: Some(88.0),
        ghost_n: Some(150.0),
        r_base: Some(1.1),
        r_depth: Some(1.7),
        rs_pow: Some(0.6),
        r_min: Some(0.3),
        ..ModeOpts::default()
    }
}

fn base_morph() -> ModeOpts {
    ModeOpts {
        icon_d: Some(1.0),
        r_dot: Some(0.021),
        r_min: Some(0.25),
        ..ModeOpts::default()
    }
}

/// Scale count knobs by `scale` (ported from `profiles.ts` scaleCounts).
fn scale_counts(mut o: ModeOpts, scale: f32) -> ModeOpts {
    let rt = scale.sqrt();
    // Lat/lon pairs scale by √scale each (total scales by `scale`).
    if let (Some(a), Some(b)) = (o.lat_rings, o.lon_density) {
        o.lat_rings = Some((a * rt).max(2.0).round());
        o.lon_density = Some((b * rt).max(2.0).round());
    }
    if let (Some(a), Some(b)) = (o.rings, o.lon_density) {
        o.rings = Some((a * rt).max(2.0).round());
        o.lon_density = Some((b * rt).max(2.0).round());
    }
    if let (Some(a), Some(b)) = (o.lanes, o.segs) {
        o.lanes = Some((a * rt).max(2.0).round());
        o.segs = Some((b * rt).max(2.0).round());
    }
    // Standalone counts scale linearly.
    for v in [
        &mut o.orbit_n,
        &mut o.ghost_n,
        &mut o.particles,
        &mut o.move_count,
        &mut o.node_n,
        &mut o.signals,
        &mut o.strand_n,
    ]
    .into_iter()
    .flatten()
    {
        if *v != 0.0 {
            *v = (*v * scale).max(1.0).round();
        }
    }
    o
}

/// Scale radius knobs by `scale` (ported from `profiles.ts` scaleRadii).
fn scale_radii(mut o: ModeOpts, scale: f32) -> ModeOpts {
    for v in [
        &mut o.r_base,
        &mut o.r_depth,
        &mut o.r_boost,
        &mut o.ghost_r,
        &mut o.part_r,
        &mut o.part_r_depth,
        &mut o.r_active,
        &mut o.node_r,
        &mut o.node_r_depth,
    ]
    .into_iter()
    .flatten()
    {
        *v *= scale;
    }
    o
}

// --- Shared math (ported from `core.ts`) --------------------------------

fn fib_dir(i: usize, n: usize) -> (f32, f32, f32) {
    let golden = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let y = 1.0 - (2.0 * (i as f32 + 0.5)) / n as f32;
    let rad = (1.0 - y * y).max(0.0).sqrt();
    let a = i as f32 * golden;
    (rad * a.cos(), y, rad * a.sin())
}

// TS: `Math.sin(a * 12.9898 + b * 78.233) * 43758.5453`: TS parity.
#[allow(clippy::excessive_precision)]
fn hash_d(a: f32, b: f32) -> f32 {
    let h = (a * 12.9898 + b * 78.233).sin() * 43758.5453;
    h - h.floor()
}

fn vnoise(x: f32, y: f32) -> f32 {
    let xi = x.floor();
    let yi = y.floor();
    let mut fx = x - xi;
    let mut fy = y - yi;
    fx = fx * fx * (3.0 - 2.0 * fx);
    fy = fy * fy * (3.0 - 2.0 * fy);
    let a = hash_d(xi, yi);
    let b = hash_d(xi + 1.0, yi);
    let c = hash_d(xi, yi + 1.0);
    let d = hash_d(xi + 1.0, yi + 1.0);
    a + (b - a) * fx + (c - a) * fy + (a - b - c + d) * fx * fy
}

fn make_proj(
    yaw: f32,
    tilt: f32,
    cx: f32,
    cy: f32,
    scale: f32,
) -> impl Fn((f32, f32, f32)) -> (f32, f32, f32) {
    let st = tilt.sin();
    let ct = tilt.cos();
    let sy = yaw.sin();
    let cyw = yaw.cos();
    move |(x, y, z)| {
        let x1 = x * cyw + z * sy;
        let z1 = -x * sy + z * cyw;
        let y1 = y * ct - z1 * st;
        let z2 = y * st + z1 * ct;
        (cx + x1 * scale, cy - y1 * scale, z2)
    }
}

fn angle_delta(a: f32, b: f32) -> f32 {
    (a - b).sin().atan2((a - b).cos())
}

fn radius_scale(size: f32, pow: f32) -> f32 {
    (size / 300.0).powf(pow)
}

/// Ink color for a dot on dark/light substrate (ported from `core.ts` inkColor).
/// Alpha is applied via the style's opacity, so we return just the gray ink.
fn ink_color(white: f32, dark: bool) -> Color {
    let g = if dark {
        ((1.0 - white).clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        (white.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    Color::Rgb(g, g, g)
}

fn opacity_for(alpha: f32) -> u8 {
    (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Z-sort dots far→near, clamp radii, cull invisible. Ported from
/// `core.ts` finalizeFrame.
fn finalize_frame(mut dots: Vec<Dot>, mut lines: Vec<Line>, r_min: f32) -> Frame {
    dots.retain(|d| d.a >= 0.02);
    for d in &mut dots {
        d.r = d.r.max(r_min);
    }
    dots.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
    lines.retain(|l| l.a >= 0.02);
    Frame { dots, lines }
}

/// Render a finished frame into the terminal grid. Dot coordinates are in the
/// orb's "pixel space" of `size_px` (the TS canvas size, 64 or 20); they are
/// mapped onto the grid area via `area.width / size_px`.
fn render_frame(area: Rect, grid: &mut Grid, frame: &Frame, dark: bool, size_px: f32) {
    let gx =
        |x: f32| ((x * area.width as f32 / size_px).round() as i32).clamp(0, area.width as i32 - 1);
    let gy = |y: f32| {
        ((y * area.height as f32 / size_px).round() as i32).clamp(0, area.height as i32 - 1)
    };

    // Draw lines first (connecting state edges), so dots sit on top.
    for l in &frame.lines {
        let steps = ((l.x2 - l.x1).abs().max((l.y2 - l.y1).abs()) * 2.0) as usize + 1;
        for s in 0..=steps {
            let f = s as f32 / steps.max(1) as f32;
            let x = l.x1 + (l.x2 - l.x1) * f;
            let y = l.y1 + (l.y2 - l.y1) * f;
            let (gx, gy) = (gx(x), gy(y));
            let style = Style::DEFAULT
                .fg(ink_color(l.white, dark))
                .opacity(opacity_for(l.a));
            grid.set(area.x + gx as u16, area.y + gy as u16, '│', style);
        }
    }

    // Draw dots far→near (z-sorted).
    for d in &frame.dots {
        let (gx, gy) = (gx(d.x), gy(d.y));
        let ch = if d.r >= 1.2 {
            '●'
        } else if d.r >= 0.6 {
            '○'
        } else if d.a > 0.5 {
            '·'
        } else {
            '.'
        };
        let style = Style::DEFAULT
            .fg(ink_color(d.white, dark))
            .opacity(opacity_for(d.a));
        grid.set(area.x + gx as u16, area.y + gy as u16, ch, style);
    }
}

// --- Mode painters (ported from `engine/*.ts`) --------------------------

fn mode_frame(state: ModeKey, size: f32, t: f32, o: &ModeOpts) -> Frame {
    match state {
        ModeKey::Orbits => frame_orbits(size, t, o),
        ModeKey::Globe => frame_globe(size, t, o),
        ModeKey::Rubik => frame_rubik(size, t, o),
        ModeKey::Wave => frame_wave(size, t, o),
        ModeKey::Web => frame_web(size, t, o),
        ModeKey::Braid => frame_braid(size, t, o),
        ModeKey::Ribbon => frame_ribbon(size, t, o),
        ModeKey::Ring => frame_ring(size, t, o),
        ModeKey::Morph => frame_morph(size, t, o),
    }
}

/// Orbits: particles on tilted orbits: the "working" state.
fn frame_orbits(size: f32, t: f32, o: &ModeOpts) -> Frame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.82;
    let pt = make_proj(t * 0.12, 0.3, cx, cy, 1.0);
    let rs = radius_scale(size, o.rs_pow.unwrap_or(0.6));

    let mut dots = Vec::new();
    let orbit_n = o.orbit_n.unwrap_or(12.0) as usize;
    let ghost_n = o.ghost_n.unwrap_or(40.0) as usize;
    let particles = o.particles.unwrap_or(3.0) as usize;

    for orb in 0..orbit_n {
        let h1 = hash_d(orb as f32, 1.7);
        let h2 = hash_d(orb as f32, 5.2);
        let h3 = hash_d(orb as f32, 8.9);
        let ro = r * (0.45 + 0.52 * h1);
        let th = h1 * 2.0 * std::f32::consts::PI;
        let phi = (2.0 * h2 - 1.0).clamp(-1.0, 1.0).acos();
        let nx = phi.sin() * th.cos();
        let ny = phi.cos();
        let nz = phi.sin() * th.sin();
        let mut ux = -ny;
        let mut uy = nx;
        let uz = 0.0;
        let ul = (ux * ux + uy * uy).max(1e-6).sqrt();
        ux /= ul;
        uy /= ul;
        let vx = ny * uz - nz * uy;
        let vy = nz * ux - nx * uz;
        let vz = nx * uy - ny * ux;
        let speed = (0.25 + 0.55 * h3) * if h3 > 0.5 { 1.0 } else { -1.0 };

        for k in 0..ghost_n {
            let a = (k as f32 / ghost_n as f32) * 2.0 * std::f32::consts::PI;
            let (px, py, z) = pt((
                (ux * a.cos() + vx * a.sin()) * ro,
                (uy * a.cos() + vy * a.sin()) * ro,
                (uz * a.cos() + vz * a.sin()) * ro,
            ));
            let depth = (z / ro + 1.0) / 2.0;
            dots.push(Dot {
                x: px,
                y: py,
                z,
                r: o.ghost_r.unwrap_or(0.9) * rs,
                white: 0.72,
                a: o.ghost_a.unwrap_or(0.5) * (0.4 + 0.6 * depth),
            });
        }
        for m in 0..particles {
            let a =
                t * speed + (m as f32 / particles as f32) * 2.0 * std::f32::consts::PI + h2 * 6.0;
            let (px, py, z) = pt((
                (ux * a.cos() + vx * a.sin()) * ro,
                (uy * a.cos() + vy * a.sin()) * ro,
                (uz * a.cos() + vz * a.sin()) * ro,
            ));
            let depth = (z / ro + 1.0) / 2.0;
            dots.push(Dot {
                x: px,
                y: py,
                z,
                r: (o.part_r.unwrap_or(1.2) + o.part_r_depth.unwrap_or(1.6) * depth) * rs,
                white: 0.3 - 0.22 * depth,
                a: 1.0,
            });
        }
    }
    finalize_frame(dots, Vec::new(), o.r_min.unwrap_or(0.3))
}

/// Globe: lat/long field, a scan meridian sweeps: "searching".
fn frame_globe(size: f32, t: f32, o: &ModeOpts) -> Frame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let radius = (size / 2.0) * 0.82;
    let tilt = 0.4 + 0.06 * (t * 0.35).sin();
    let pt = make_proj(t * 0.5, tilt, cx, cy, radius);
    let scan_mul = o.scan_mul.unwrap_or(1.0);
    let scan = t * (0.5 + (1.7 - 0.5) * scan_mul);
    let rs = radius_scale(size, o.rs_pow.unwrap_or(0.6));
    let dim_base = o.dim_base.unwrap_or(1.0);

    let mut dots = Vec::new();
    let lat_rings = o.lat_rings.unwrap_or(17.0) as usize;
    let lon_density = o.lon_density.unwrap_or(44.0) as usize;

    for li in 0..=lat_rings {
        let lat =
            -std::f32::consts::PI / 2.0 + (li as f32 / lat_rings as f32) * std::f32::consts::PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();
        let lon_count = (cos_lat.abs() * lon_density as f32).round().max(1.0) as usize;
        for lj in 0..lon_count {
            let lon = (lj as f32 / lon_count as f32) * 2.0 * std::f32::consts::PI;
            let (px, py, z) = pt((cos_lat * lon.cos(), sin_lat, cos_lat * lon.sin()));
            let depth = (z + 1.0) / 2.0;
            let d = angle_delta(lon + t * 0.5, scan);
            let boost = (-(d * d) / 0.18).exp() * z.max(0.0);
            dots.push(Dot {
                x: px,
                y: py,
                z,
                r: (o.r_base.unwrap_or(0.6)
                    + o.r_depth.unwrap_or(1.7) * depth
                    + o.r_boost.unwrap_or(1.0) * boost)
                    * rs,
                white: o.ink_far.unwrap_or(0.62) - o.ink_span.unwrap_or(0.54) * depth,
                a: dim_base + (1.0 - dim_base) * boost.min(1.0),
            });
        }
    }
    finalize_frame(dots, Vec::new(), o.r_min.unwrap_or(0.3))
}

/// Rubik: bands twist in quarter turns, scramble → solve: "solving".
fn frame_rubik(size: f32, t: f32, o: &ModeOpts) -> Frame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.82;
    let tilt = 0.35 + 0.1 * (t * 0.9).sin();
    let pt = make_proj(t * 0.55, tilt, cx, cy, r);
    let rs = radius_scale(size, o.rs_pow.unwrap_or(0.6));
    let move_count = o.move_count.unwrap_or(14.0) as usize;
    let moves = make_moves(move_count);
    let sc = solve_cycle(t, move_count, 0.42, 1.2);

    let mut dots = Vec::new();
    let lat_rings = o.lat_rings.unwrap_or(15.0) as usize;
    let lon_density = o.lon_density.unwrap_or(40.0) as usize;

    for li in 0..=lat_rings {
        let lat =
            -std::f32::consts::PI / 2.0 + (li as f32 / lat_rings as f32) * std::f32::consts::PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();
        let lon_count = (cos_lat.abs() * lon_density as f32).round().max(1.0) as usize;
        for lj in 0..lon_count {
            let lon = (lj as f32 / lon_count as f32) * 2.0 * std::f32::consts::PI;
            let pt3 = (cos_lat * lon.cos(), sin_lat, cos_lat * lon.sin());
            let (x, y, z, in_active) = apply_moves(pt3, &moves, &sc);
            let (px, py, zr) = pt((x, y, z));
            let depth = (zr + 1.0) / 2.0;
            dots.push(Dot {
                x: px,
                y: py,
                z: zr,
                r: (o.r_base.unwrap_or(0.6)
                    + o.r_depth.unwrap_or(1.7) * depth
                    + if in_active {
                        o.r_active.unwrap_or(0.3)
                    } else {
                        0.0
                    })
                    * rs,
                white: o.ink_far.unwrap_or(0.62)
                    - o.ink_span.unwrap_or(0.54) * depth
                    - if in_active { 0.14 } else { 0.0 },
                a: 1.0,
            });
        }
    }
    finalize_frame(dots, Vec::new(), o.r_min.unwrap_or(0.3))
}

/// Wave: a waveform rolls through the rings: "listening".
fn frame_wave(size: f32, t: f32, o: &ModeOpts) -> Frame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let rr = (size / 2.0) * 0.874;
    let pt = make_proj(t * 0.18, 0.38, cx, cy, 1.0);
    let rs = radius_scale(size, o.rs_pow.unwrap_or(0.6));

    let mut dots = Vec::new();
    let rings = o.rings.unwrap_or(15.0) as usize;
    let lon_density = o.lon_density.unwrap_or(40.0) as usize;

    for ri in 0..=rings {
        let lat = -std::f32::consts::PI / 2.0 + (ri as f32 / rings as f32) * std::f32::consts::PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();
        let w =
            0.62 * (t * 2.1 - ri as f32 * 0.52).sin() + 0.38 * (t * 1.27 + ri as f32 * 0.83).sin();
        let rad = rr * (0.88 + 0.105 * w);
        let lon_count = (cos_lat.abs() * lon_density as f32).round().max(1.0) as usize;
        for lj in 0..lon_count {
            let lon = (lj as f32 / lon_count as f32) * 2.0 * std::f32::consts::PI;
            let (px, py, z) = pt((
                cos_lat * lon.cos() * rad,
                sin_lat * rad,
                cos_lat * lon.sin() * rad,
            ));
            let depth = (z / rr + 1.0) / 2.0;
            let crest = w.max(0.0);
            dots.push(Dot {
                x: px,
                y: py,
                z,
                r: (o.r_base.unwrap_or(0.6) + o.r_depth.unwrap_or(1.7) * depth)
                    * (1.0 + 0.4 * crest)
                    * rs,
                white: 0.66 - 0.56 * depth - 0.1 * crest,
                a: 1.0,
            });
        }
    }
    finalize_frame(dots, Vec::new(), o.r_min.unwrap_or(0.3))
}

/// Web: a constellation wires itself: "connecting".
fn frame_web(size: f32, t: f32, o: &ModeOpts) -> Frame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.8 * o.spread.unwrap_or(1.0);
    let pt = make_proj(t * 0.12, 0.32, cx, cy, r);
    let rs = radius_scale(size, o.rs_pow.unwrap_or(0.6));

    let node_n = o.node_n.unwrap_or(30.0) as usize;
    let thr = o.thr.unwrap_or(0.72);
    let node_r = o.node_r.unwrap_or(1.4);
    let node_r_depth = o.node_r_depth.unwrap_or(1.8);

    let mut nodes: Vec<(f32, f32, f32)> = Vec::new();
    for i in 0..node_n {
        let d = fib_dir(i, node_n);
        let x = d.0 + 0.3 * (vnoise(i as f32 * 0.31 + 9.0, t * 0.24) - 0.5) * 2.0;
        let y = d.1 + 0.3 * (vnoise(i as f32 * 0.53 + 27.0, t * 0.21) - 0.5) * 2.0;
        let z = d.2 + 0.3 * (vnoise(i as f32 * 0.77 + 55.0, t * 0.27) - 0.5) * 2.0;
        let l = (x * x + y * y + z * z).sqrt();
        nodes.push((x / l, y / l, z / l));
    }

    let mut lines = Vec::new();
    let mut dots = Vec::new();

    for (i, node_i) in nodes.iter().enumerate() {
        for (_j, node_j) in nodes.iter().enumerate().filter(|(j, _)| *j > i) {
            let dx = node_i.0 - node_j.0;
            let dy = node_i.1 - node_j.1;
            let dz = node_i.2 - node_j.2;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist >= thr {
                continue;
            }
            let (x1, y1, z1) = pt(*node_i);
            let (x2, y2, z2) = pt(*node_j);
            let depth = ((z1 + z2) / 2.0 + 1.0) / 2.0;
            lines.push(Line {
                x1,
                y1,
                x2,
                y2,
                white: 0.42,
                a: (1.0 - dist / thr) * (0.3 + 0.55 * depth),
            });
        }
    }

    for (i, node) in nodes.iter().enumerate() {
        let (px, py, z) = pt(*node);
        let depth = (z + 1.0) / 2.0;
        let pulse = 1.0 + 0.25 * (t * 1.4 + i as f32 * 2.7).sin();
        dots.push(Dot {
            x: px,
            y: py,
            z,
            r: (node_r + node_r_depth * depth) * pulse * rs,
            white: 0.55 - 0.45 * depth,
            a: 1.0,
        });
    }

    let signals = o.signals.unwrap_or(5.0) as usize;
    for s in 0..signals {
        let seg = (t * 0.55 + s as f32 * 7.31).floor() as usize;
        let a = hash_d(seg as f32, s as f32 * 3.1 + 1.7) * node_n as f32;
        let b = hash_d(seg as f32, s as f32 * 5.7 + 4.2) * node_n as f32;
        let a = a as usize % node_n;
        let b = b as usize % node_n;
        if a == b {
            continue;
        }
        let f = (t * 0.55 + s as f32 * 7.31) - (t * 0.55 + s as f32 * 7.31).floor();
        let x = nodes[a].0 + (nodes[b].0 - nodes[a].0) * f;
        let y = nodes[a].1 + (nodes[b].1 - nodes[a].1) * f;
        let z = nodes[a].2 + (nodes[b].2 - nodes[a].2) * f;
        let l = (x * x + y * y + z * z).max(1e-6).sqrt();
        let (px, py, zr) = pt((x / l, y / l, z / l));
        let depth = (zr + 1.0) / 2.0;
        dots.push(Dot {
            x: px,
            y: py,
            z: zr,
            r: (node_r * 1.5 + node_r_depth * depth) * rs,
            white: 0.05,
            a: 0.5 + 0.5 * depth,
        });
    }

    finalize_frame(dots, lines, o.r_min.unwrap_or(0.3))
}

/// Braid: three strands plait around the sphere: "weaving".
fn frame_braid(size: f32, t: f32, o: &ModeOpts) -> Frame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.76;
    let pt = make_proj(t * 0.4, 0.3, cx, cy, 1.0);
    let rs = radius_scale(size, o.rs_pow.unwrap_or(0.6));

    let mut dots = Vec::new();
    let ghost_n = o.ghost_n.unwrap_or(150.0) as usize;

    for i in 0..ghost_n {
        let d = fib_dir(i, ghost_n);
        let (px, py, z) = pt((d.0 * r, d.1 * r, d.2 * r));
        let depth = (z / r + 1.0) / 2.0;
        dots.push(Dot {
            x: px,
            y: py,
            z,
            r: 0.8 * rs,
            white: 0.78,
            a: 0.1 + 0.22 * depth,
        });
    }

    let strand_n = o.strand_n.unwrap_or(52.0) as usize;
    let turns = o.turns.unwrap_or(3.0);

    for s in 0..3 {
        let phase = (s as f32 / 3.0) * 2.0 * std::f32::consts::PI;
        for i in 0..strand_n {
            let u = ((i as f32 / strand_n as f32 + t * 0.045).fract() * 2.0 - 1.0) * 0.96;
            let surf = (1.0 - u * u).max(0.0).sqrt();
            let end_fade = ((1.0 - u.abs()) / 0.1).min(1.0);
            let a = u * std::f32::consts::PI * turns + phase;
            let weave = 1.0
                + 0.075 * (u * std::f32::consts::PI * turns * 2.0 + phase * 2.0 + t * 0.8).sin();
            let rr = surf * r * weave;
            let (px, py, zr) = pt((a.cos() * rr, u * r * weave, a.sin() * rr));
            let depth = (zr / r + 1.0) / 2.0;
            dots.push(Dot {
                x: px,
                y: py,
                z: zr,
                r: (o.r_base.unwrap_or(1.2) + o.r_depth.unwrap_or(1.8) * depth) * rs,
                white: 0.55 - 0.45 * depth,
                a: end_fade * (0.45 + 0.55 * depth),
            });
        }
    }
    finalize_frame(dots, Vec::new(), o.r_min.unwrap_or(0.3))
}

/// Ribbon: an undulating sash: "composing". Also drives "breathing" (ring)
/// via the `face_on` flag.
fn frame_ribbon(size: f32, t: f32, o: &ModeOpts) -> Frame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.78;
    let spin = o.spin.unwrap_or(1.0);
    let cam_tilt = 0.3;
    let pt = make_proj(t * 0.1 * spin, cam_tilt, cx, cy, 1.0);
    let rs = radius_scale(size, o.rs_pow.unwrap_or(0.6));

    let mut dots = Vec::new();
    let ghost_n = o.ghost_n.unwrap_or(150.0) as usize;
    for i in 0..ghost_n {
        let d = fib_dir(i, ghost_n);
        let (px, py, z) = pt((d.0 * r, d.1 * r, d.2 * r));
        let depth = (z / r + 1.0) / 2.0;
        dots.push(Dot {
            x: px,
            y: py,
            z,
            r: 0.8 * rs,
            white: 0.78,
            a: 0.1 + 0.22 * depth,
        });
    }

    let ya = t * 0.24 * spin;
    let ta = if o.face_on.unwrap_or(0.0) != 0.0 {
        -cam_tilt
    } else {
        0.55 + 0.3 * (t * 0.18).sin() * spin
    };
    let ux = ya.cos();
    let uy = 0.0;
    let uz = ya.sin();
    let vx = -uz * ta.sin();
    let vy = ta.cos();
    let vz = ux * ta.sin();
    let nx = uy * vz - uz * vy;
    let ny = uz * vx - ux * vz;
    let nz = ux * vy - uy * vx;

    let wob_amp = 0.23 * o.wob_mul.unwrap_or(1.0);
    let base_r = if o.face_on.unwrap_or(0.0) != 0.0 {
        r / (1.0 + 0.85 * wob_amp)
    } else {
        r
    };

    let base_lanes = o.lanes.unwrap_or(5.0) as usize;
    let segs = o.segs.unwrap_or(88.0) as usize;
    let lanes = (base_lanes as f32 * o.band_mul.unwrap_or(1.0))
        .round()
        .max(1.0) as usize;

    for w in 0..lanes {
        let lane_off = (w as f32 - (lanes - 1) as f32 / 2.0) * 0.075;
        let edge =
            (w as f32 - (lanes - 1) as f32 / 2.0).abs() / ((lanes - 1) as f32 / 2.0).max(1.0);
        for k in 0..segs {
            let a = (k as f32 / segs as f32) * 2.0 * std::f32::consts::PI;
            let wob = (0.16 * (a * 3.0 - t * 1.7 + w as f32 * 0.22).sin()
                + 0.07 * (a * 5.0 + t * 1.1).sin())
                * o.wob_mul.unwrap_or(1.0);
            let radial = if o.face_on.unwrap_or(0.0) != 0.0 {
                1.0 + wob
            } else {
                1.0
            };
            let off = if o.face_on.unwrap_or(0.0) != 0.0 {
                lane_off
            } else {
                lane_off + wob
            };
            let x = ux * a.cos() + vx * a.sin() + nx * off;
            let y = uy * a.cos() + vy * a.sin() + ny * off;
            let z = uz * a.cos() + vz * a.sin() + nz * off;
            let l = (x * x + y * y + z * z).sqrt();
            let rr = base_r * radial;
            let (px, py, zr) = pt(((x / l) * rr, (y / l) * rr, (z / l) * rr));
            let depth = (zr / r + 1.0) / 2.0;
            dots.push(Dot {
                x: px,
                y: py,
                z: zr,
                r: (o.r_base.unwrap_or(1.1) + o.r_depth.unwrap_or(1.7) * depth)
                    * (1.0 - 0.25 * edge)
                    * rs,
                white: 0.52 - 0.44 * depth + 0.18 * edge,
                a: 0.4 + 0.6 * depth,
            });
        }
    }
    finalize_frame(dots, Vec::new(), o.r_min.unwrap_or(0.3))
}

/// Ring: face-on ribbon: "breathing". Delegates to ribbon with face_on=1.
fn frame_ring(size: f32, t: f32, o: &ModeOpts) -> Frame {
    frame_ribbon(size, t, o)
}

/// Morph: dotted outline circle → triangle → square: "shaping".
fn frame_morph(size: f32, t: f32, o: &ModeOpts) -> Frame {
    fn smooth_e(x: f32) -> f32 {
        x * x * (3.0 - 2.0 * x)
    }

    fn poly_path(verts: Vec<(f32, f32)>) -> Box<dyn Fn(f32) -> (f32, f32)> {
        let v = verts.len();
        let mut l: Vec<f32> = Vec::with_capacity(v);
        let mut total = 0.0;
        for i in 0..v {
            let a = verts[i];
            let b = verts[(i + 1) % v];
            let seg_len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            l.push(seg_len);
            total += seg_len;
        }
        Box::new(move |f: f32| {
            let mut target = f * total;
            let mut i = 0;
            while target > l[i] && i < v - 1 {
                target -= l[i];
                i += 1;
            }
            let a = verts[i];
            let b = verts[(i + 1) % v];
            let ff = if l[i] > 0.0 {
                (target / l[i]).min(1.0)
            } else {
                0.0
            };
            (a.0 + (b.0 - a.0) * ff, a.1 + (b.1 - a.1) * ff)
        })
    }

    let circle: Box<dyn Fn(f32) -> (f32, f32)> = Box::new(|f| {
        let a = -std::f32::consts::PI / 2.0 + f * 2.0 * std::f32::consts::PI;
        (a.cos() * 0.24, a.sin() * 0.24)
    });
    let triangle = poly_path(vec![(0.0, -0.26), (0.24, 0.16), (-0.24, 0.16)]);
    let square = poly_path(vec![
        (0.0, -0.2),
        (0.2, -0.2),
        (0.2, 0.2),
        (-0.2, 0.2),
        (-0.2, -0.2),
    ]);
    let cycle = [circle, triangle, square];

    let k_cycle = cycle.len() as f32;
    let seg = 1.4 + 0.9;
    let tc = t % (seg * k_cycle);
    let k = ((tc / seg).floor() as usize).min(cycle.len() - 1);
    let local = tc - k as f32 * seg;
    let m = if local > 1.4 {
        smooth_e((local - 1.4) / 0.9)
    } else {
        0.0
    };
    let sprd = o.spread.unwrap_or(1.0);

    let p_a: &dyn Fn(f32) -> (f32, f32) = &*cycle[k];
    let p_b: &dyn Fn(f32) -> (f32, f32) = &*cycle[(k + 1) % cycle.len()];
    let m_pts = 160;
    let mut pts: Vec<(f32, f32)> = Vec::with_capacity(m_pts);
    for i in 0..m_pts {
        let f = i as f32 / m_pts as f32;
        let a = p_a(f);
        let b = p_b(f);
        pts.push((
            (a.0 + (b.0 - a.0) * m) * sprd,
            (a.1 + (b.1 - a.1) * m) * sprd,
        ));
    }
    let mut seg_lens: Vec<f32> = Vec::with_capacity(m_pts);
    let mut total = 0.0;
    for i in 0..m_pts {
        let a = pts[i];
        let b = pts[(i + 1) % m_pts];
        let seg_len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        seg_lens.push(seg_len);
        total += seg_len;
    }

    let n = ((34.0 * o.icon_d.unwrap_or(1.0)).round() as usize).max(6);
    let re = o.r_dot.unwrap_or(0.021) * 1.35 * sprd;
    let pulse = 1.0 + 0.02 * (local * 3.1).sin();

    let mut dots = Vec::new();
    let c2 = size / 2.0;
    let mut seg_idx = 0;
    let mut acc = 0.0;
    for k2 in 0..n {
        let target = (k2 as f32 / n as f32) * total;
        while acc + seg_lens[seg_idx] < target && seg_idx < m_pts - 1 {
            acc += seg_lens[seg_idx];
            seg_idx += 1;
        }
        let a = pts[seg_idx];
        let b = pts[(seg_idx + 1) % m_pts];
        let f = if seg_lens[seg_idx] > 0.0 {
            ((target - acc) / seg_lens[seg_idx]).min(1.0)
        } else {
            0.0
        };
        let x = (a.0 + (b.0 - a.0) * f) * pulse;
        let y = (a.1 + (b.1 - a.1) * f) * pulse;
        dots.push(Dot {
            x: c2 + x * size,
            y: c2 + y * size,
            z: 0.0,
            r: (re * size).max(0.35),
            white: 0.1,
            a: 1.0,
        });
    }
    finalize_frame(dots, Vec::new(), o.r_min.unwrap_or(0.25))
}

/// --- Rubik helpers (ported from lattice.ts) ---------------------------

#[derive(Clone, Copy)]
struct Move {
    axis: usize,
    lo: f32,
    hi: f32,
    ang: f32,
}

fn solve_cycle(time: f32, count: usize, slot_dur: f32, rest: f32) -> (Vec<f32>, isize) {
    let cyc = 2.0 * count as f32 * slot_dur + rest;
    let tc = time % cyc;
    let mut amount = vec![0.0; count];
    let mut active = -1isize;
    if tc < 2.0 * count as f32 * slot_dur {
        let slot = (tc / slot_dur).floor() as usize;
        let p = (tc - slot as f32 * slot_dur) / slot_dur;
        let cl = (1.0_f32).min(p / 0.7);
        let ep = 1.0 - (1.0 - cl).powi(3);
        if slot < count {
            for v in amount.iter_mut().take(slot) {
                *v = 1.0;
            }
            amount[slot] = ep;
            active = slot as isize;
        } else {
            let u = 2 * count - 1 - slot;
            for v in amount.iter_mut().take(u) {
                *v = 1.0;
            }
            amount[u] = 1.0 - ep;
            active = u as isize;
        }
    }
    (amount, active)
}

fn apply_moves(
    pt3: (f32, f32, f32),
    moves: &[Move],
    sc: &(Vec<f32>, isize),
) -> (f32, f32, f32, bool) {
    let (amount, active) = sc;
    let (mut x, mut y, mut z) = pt3;
    let mut in_active = false;
    for (i, mv) in moves.iter().enumerate() {
        if amount[i] <= 0.0 {
            continue;
        }
        let coord = match mv.axis {
            0 => x,
            1 => y,
            _ => z,
        };
        if coord < mv.lo || coord >= mv.hi {
            continue;
        }
        if i as isize == *active {
            in_active = true;
        }
        let a = mv.ang * amount[i];
        let ca = a.cos();
        let sa = a.sin();
        if mv.axis == 0 {
            let y2 = y * ca - z * sa;
            z = y * sa + z * ca;
            y = y2;
        } else if mv.axis == 1 {
            let x2 = x * ca + z * sa;
            z = -x * sa + z * ca;
            x = x2;
        } else {
            let x2 = x * ca - y * sa;
            y = x * sa + y * ca;
            x = x2;
        }
    }
    (x, y, z, in_active)
}

fn make_moves(count: usize) -> Vec<Move> {
    let mut moves = Vec::with_capacity(count);
    for i in 0..count {
        let axis = (hash_d(i as f32, 2.3) * 3.0).floor() as usize;
        let axis = axis.min(2);
        let lo = -1.0 + 0.5 * (hash_d(i as f32, 5.9) * 4.0).floor().min(3.0);
        let dir = if hash_d(i as f32, 7.7) < 0.5 {
            1.0
        } else {
            -1.0
        };
        moves.push(Move {
            axis,
            lo,
            hi: lo + 0.5,
            ang: dir * std::f32::consts::PI / 2.0,
        });
    }
    moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Cell;

    #[test]
    fn thinking_orb_defaults() {
        let o = ThinkingOrb::new();
        assert_eq!(o.state, OrbState::Working);
        assert_eq!(o.size, OrbSize::Size64);
        assert_eq!(o.speed, 1.0);
        assert!(o.dark);
        assert!(!o.paused);
    }

    #[test]
    fn thinking_orbs_states_cover_all_nine() {
        let states = [
            OrbState::Working,
            OrbState::Searching,
            OrbState::Solving,
            OrbState::Listening,
            OrbState::Connecting,
            OrbState::Weaving,
            OrbState::Composing,
            OrbState::Breathing,
            OrbState::Shaping,
        ];
        assert_eq!(states.len(), 9);
        for s in states {
            assert!(!s.as_str().is_empty());
            assert!(!s.label().is_empty());
        }
    }

    #[test]
    fn thinking_orb_renders_without_panic() {
        for state in [
            OrbState::Working,
            OrbState::Searching,
            OrbState::Solving,
            OrbState::Listening,
            OrbState::Connecting,
            OrbState::Weaving,
            OrbState::Composing,
            OrbState::Breathing,
            OrbState::Shaping,
        ] {
            let mut orb = ThinkingOrb::new().state(state).size(OrbSize::Size64);
            let mut grid = Grid::new(9, 9, Cell::blank());
            for _ in 0..10 {
                orb.on_tick();
                orb.render(Rect::new(0, 0, 9, 9), &mut grid);
            }
            // At least one non-blank cell should be painted.
            let painted = (0..9).any(|x| (0..9).any(|y| grid.get(x, y).ch != ' '));
            assert!(painted, "state {:?} painted nothing", state.as_str());
        }
    }

    #[test]
    fn thinking_orb_inline_size_renders() {
        let mut orb = ThinkingOrb::new()
            .size(OrbSize::Size20)
            .state(OrbState::Searching);
        let mut grid = Grid::new(5, 5, Cell::blank());
        for _ in 0..5 {
            orb.on_tick();
            orb.render(Rect::new(0, 0, 5, 5), &mut grid);
        }
        let painted = (0..5).any(|x| (0..5).any(|y| grid.get(x, y).ch != ' '));
        assert!(painted);
    }

    #[test]
    fn thinking_orb_paused_freezes() {
        let mut orb = ThinkingOrb::new().paused(true).state(OrbState::Working);
        let t0 = orb.time();
        for _ in 0..20 {
            orb.on_tick();
        }
        assert_eq!(orb.time(), t0);
    }

    #[test]
    fn thinking_orb_tick_advances_time() {
        let mut orb = ThinkingOrb::new();
        let t0 = orb.time();
        orb.on_tick();
        assert!(orb.time() > t0);
    }
}
