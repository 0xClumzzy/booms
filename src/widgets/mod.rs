//! Widgets: things that paint into the [`Grid`].

use crossterm::event::{KeyEvent, MouseEvent};

use crate::grid::Grid;
use crate::layout::{Constraint, Rect};

/// A widget that paints into the back-buffer [`Grid`].
///
/// Widgets are added to an [`App`](crate::app::App), laid out with a
/// [`Constraint`], and told about keys and ticks. Widgets with animations
/// can drive them from [`on_tick`](Widget::on_tick) with
/// [`Anim`](crate::anim::Anim).
pub trait Widget {
    /// Paint into `grid`, inside `area`.
    fn render(&self, area: Rect, grid: &mut Grid);

    /// Handle a key press while focused. Returns true if the widget used it.
    fn on_key(&mut self, key: KeyEvent) -> bool {
        let _ = key;
        false
    }

    /// Handle a mouse event. `area` is the widget's own rectangle on the
    /// grid; the event's `column`/`row` are absolute (0-based) coordinates,
    /// so widgets should subtract `area.x`/`area.y` when hitting-testing.
    /// Returns true if the widget consumed the event.
    ///
    /// Mouse capture must be enabled by the terminal backend; booms'
    /// [`App`](crate::app::App) enables it automatically.
    fn on_mouse(&mut self, event: MouseEvent, area: Rect) -> bool {
        let _ = (event, area);
        false
    }

    /// Called every tick (at [`App::tick_interval`](crate::app::App::tick_interval)).
    fn on_tick(&mut self) {}

    /// Whether the widget is a tab stop.
    fn is_focusable(&self) -> bool {
        false
    }

    /// Called when the widget gains or loses focus.
    fn set_focus(&mut self, focused: bool) {
        let _ = focused;
    }

    /// The [`Constraint`] that reserves space for this widget in the
    /// column layout: its height.
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    /// Where the terminal cursor rests for this widget while focused.
    fn cursor_pos(&self, area: Rect) -> Option<(u16, u16)> {
        let _ = area;
        None
    }
}

pub mod asciify;
pub mod block;
pub mod crystal_orb;
pub mod gooey_orb;
pub mod input;
pub mod list;
pub mod markdown;
pub mod orbs;
pub mod panel;
pub mod pulse_orb;
pub mod ratatui;
#[cfg(feature = "ratatui")]
pub mod ratatui_native;
pub mod ring_orb;
pub mod scrollbar;
pub mod starship;
pub mod text;
pub mod thinking_orb;
pub mod wave_orb;

pub use asciify::Asciify;
pub use block::{Block, Borders};
pub use crystal_orb::CrystalOrb;
pub use gooey_orb::LiquidGooeyOrb;
pub use input::Input;
pub use list::List;
pub use markdown::MarkdownText;
pub use orbs::{
    AuraOrb, CometOrb, GlassOrb, HeartbeatOrb, MorphOrb, NeonOrb, RippleOrb, SpinnerOrb,
};
pub use panel::{BorderType, Panel, TitleAlignment, TitlePosition};
pub use pulse_orb::PulseOrb;
pub use ratatui::{BarChart, Gauge, LineGauge, Sparkline, Table, Tabs};
#[cfg(feature = "ratatui")]
pub use ratatui_native::RatatuiView;
pub use ring_orb::RingOrb;
pub use scrollbar::Scrollbar;
pub use starship::{StarshipInput, StarshipPrompt};
pub use text::{Text, wrap_line as wrap_text};
pub use thinking_orb::{OrbSize, OrbState, ThinkingOrb};
pub use wave_orb::WaveOrb;
