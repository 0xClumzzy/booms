//! # booms
//!
//! A small, safe TUI framework in Rust with crystal orb animations.
//!
//! The main pieces:
//!
//! - [`App`]: the event loop: polls input, ticks widgets, lays out widgets
//!   and redraws the screen. Supports a view stack with animated
//!   [`Transition`](crate::transitions::Transition)s.
//! - [`Grid`]: the back buffer widgets paint into, written to the terminal
//!   with diffing.
//! - [`Term`]: the raw-mode, alternate-screen terminal backend, built on
//!   [`crossterm`](https://crates.io/crates/crossterm).
//! - [`Text`], [`Block`], [`Input`], [`List`]: the built-in widgets.
//! - [`PulseOrb`], [`CrystalOrb`], [`RingOrb`], [`LiquidGooeyOrb`],
//!   [`WaveOrb`]: orb animations (nano-style pulse, faceted crystal,
//!   rotating rings, liquid gooey blob, distorted wave rings).
//! - [`SpinnerOrb`], [`AuraOrb`], [`CometOrb`], [`HeartbeatOrb`],
//!   [`GlassOrb`], [`NeonOrb`]: thinking orbs (braille spinner, layered glow,
//!   orbiting trail, heartbeat pulse, frosted glass, neon sign).
//! - [`ThinkingOrb`], [`RippleOrb`], [`MorphOrb`]: AI "thinking" indicators
//!   (chasing ellipsis, sonar brain waves, morphing shapes).
//! - [`Anim`], [`Easing`], [`Spring`]: eased and spring-physics transition
//!   primitives.
//! - [`Transition`]: fade, slide, and cube view transitions.
//! - [`render_markdown`] / [`MarkdownText`]: glow-style markdown rendering.
//! - [`AsciifyEffect`] / [`Asciify`]: animated ASCII scenes (a terminal
//!   port of the asciify-engine idea).
//! - [`StarshipPrompt`] / [`StarshipInput`]: a Rust port of a starship.toml
//!   "Noctalia" trailing-bubble capsule prompt, with right-side time.
//! - [`Panel`]: rounded / square / double / thick borders whose titles wrap
//!   around the border perimeter.
//! - [`RatatuiView`](crate::widgets::RatatuiView) (feature `ratatui`): render
//!   real [`ratatui`](https://crates.io/crates/ratatui) widgets
//!   (Paragraph, Table, Gauge, ...) inside booms.
//! - `GpuSurface` (feature `gpu`): headless wgpu rendering read back into
//!   terminal cells via the truecolor half-block trick.
//!
//! ```no_run
//! use booms::app::App;
//! use booms::widgets::{CrystalOrb, LiquidGooeyOrb, Text};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut app = App::new()?;
//!     app.add(Text::new(["hello".to_string()]));
//!     app.add(CrystalOrb::new());
//!     app.run()?;
//!     Ok(())
//! }
//! ```

#[cfg(feature = "gpu")]
pub mod gpu;
#[cfg(feature = "gpu")]
pub use gpu::GpuSurface;

pub mod anim;
pub mod app;
pub mod asciify;
pub mod grid;
pub mod layout;
pub mod markdown;
pub mod style;
pub mod term;
pub mod transitions;
pub mod widgets;

pub use anim::{Anim, Easing, Interpolate, Spring};
pub use app::App;
pub use asciify::AsciifyEffect;
pub use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
pub use grid::{Cell, Grid};
pub use layout::{Constraint, Rect};
pub use markdown::{MdLine, MdSpan, render_markdown};
pub use style::{Color, Modifiers, Style};
pub use transitions::{SlideDirection, Transition};
#[cfg(feature = "ratatui")]
pub use widgets::RatatuiView;
pub use widgets::{
    Asciify, AuraOrb, BarChart, Block, BorderType, Borders, CometOrb, CrystalOrb, Gauge,
    HeartbeatOrb, Input, LineGauge, LiquidGooeyOrb, List, MarkdownText, MorphOrb, OrbSize,
    OrbState, Panel, PulseOrb, RingOrb, RippleOrb, Scrollbar, Sparkline, SpinnerOrb, StarshipInput,
    StarshipPrompt, Table, Tabs, Text, ThinkingOrb, TitleAlignment, TitlePosition, WaveOrb,
};
