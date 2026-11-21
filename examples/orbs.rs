//! A showcase of all booms orb widgets: nano-style pulsing indicators,
//! a faceted crystal gem, concentric rotating rings, a liquid gooey blob,
//! and distorted wave rings.
//!
//! Run with: `cargo run --example orbs`

use booms::app::App;
use booms::grid::Grid;
use booms::layout::{Constraint, Rect};
use booms::style::{Color, Style};
use booms::widgets::{CrystalOrb, Input, LiquidGooeyOrb, List, PulseOrb, RingOrb, WaveOrb, Widget};

/// A title banner.
struct Title {
    lines: Vec<String>,
}

impl Title {
    fn new(lines: Vec<String>) -> Title {
        Title { lines }
    }
}

impl Widget for Title {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(self.lines.len() as u16)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let style = Style::DEFAULT.fg(Color::Ansi(14));
        for (i, line) in self.lines.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.bottom() {
                return;
            }
            grid.set_str(area.x, y, line, style);
        }
    }
}

/// A small legend line describing an orb.
struct Legend {
    text: String,
    style: Style,
}

impl Legend {
    fn new(text: String, style: Style) -> Legend {
        Legend { text, style }
    }
}

impl Widget for Legend {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let y = area.y;
        if y >= area.bottom() {
            return;
        }
        grid.set_str(area.x, y, &self.text, self.style);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;
    app.add(Title::new(vec![
        "booms".to_string(),
        "orb animation showcase".to_string(),
    ]));

    // Nano-style pulsing status indicators.
    app.add(PulseOrb::new(Color::Ansi(10))); // green: ok
    app.add(PulseOrb::new(Color::Ansi(11))); // blue: sync
    app.add(PulseOrb::new(Color::Ansi(9))); // red: alert
    app.add(PulseOrb::new(Color::Ansi(13))); // magenta: pulse

    // The faceted crystal gem with a sweeping shine.
    app.add(CrystalOrb::new());

    // Concentric rotating rings around a pulsing core.
    app.add(RingOrb::new().center(4, 3));

    // Liquid gooey blob with sine-wave edge distortion and spring wobble.
    app.add(LiquidGooeyOrb::new());

    // Distorted wave orb with interfering sine-wave rings.
    app.add(WaveOrb::new());

    app.add(Legend::new(
        "green=ok  blue=sync  red=alert  magenta=pulse".to_string(),
        Style::DEFAULT.fg(Color::Ansi(7)),
    ));
    app.add(
        List::new([
            "orchestra",
            "consortium",
            "syndicate",
            "commune",
            "kibbutz",
            "utopia",
            "mutualism",
        ])
        .style(Style::DEFAULT.fg(Color::Ansi(7))),
    );
    app.add(Input::new().placeholder("type here"));
    app.run()?;
    Ok(())
}
