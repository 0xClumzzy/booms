//! Aesthetic showcase: booms' beauty features in one view:
//! crisp transparency (glass panels), orb-like interface (glass + neon +
//! pulse + aura orbs), and rounded/double borders with wrap-around titles.
//!
//! Run with: `cargo run --example aesthetic`
//!
//! Click panels to focus them (mouse support). Press Escape to quit.

use booms::app::App;
use booms::style::{Color, Modifiers, Style};
use booms::widgets::{
    AuraOrb, BorderType, CometOrb, GlassOrb, NeonOrb, OrbSize, OrbState, Panel, PulseOrb,
    SpinnerOrb, ThinkingOrb,
};

// ── Noctalia palette (Catppuccin Mocha) ───────────────────────────────────

const BLUE: Color = Color::Rgb(137, 180, 250);
const GREEN: Color = Color::Rgb(166, 227, 161);
const CYAN: Color = Color::Rgb(148, 226, 213);
const MAGENTA: Color = Color::Rgb(245, 194, 231);
const PEACH: Color = Color::Rgb(250, 179, 135);
const SKY: Color = Color::Rgb(137, 220, 235);
const YELLOW: Color = Color::Rgb(249, 226, 175);
const TEXT: Color = Color::Rgb(205, 214, 244);
const BASE: Color = Color::Rgb(30, 30, 46);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;
    app.set_canvas_colors(TEXT, BASE);

    // Top row: a row of glass panels, each with a different orb inside.
    app.add(
        Panel::new()
            .title("orb interface")
            .borders(BorderType::Rounded)
            .border_style(Style::DEFAULT.fg(CYAN).opacity(200))
            .content_style(Style::DEFAULT.fg(TEXT).opacity(210))
            .title_style(Style::DEFAULT.fg(CYAN).mods(Modifiers::BOLD))
            .opacity(180)
            .padding(1),
    );
    app.add(GlassOrb::new().color(CYAN));
    app.add(NeonOrb::new().color(MAGENTA));
    app.add(PulseOrb::new(GREEN));

    // Second row: aura + comet + spinner orbs in a double-bordered panel.
    app.add(
        Panel::new()
            .title("thinking orbs")
            .borders(BorderType::Double)
            .border_style(Style::DEFAULT.fg(BLUE).opacity(190))
            .content_style(Style::DEFAULT.fg(TEXT).opacity(210))
            .title_style(Style::DEFAULT.fg(BLUE).mods(Modifiers::BOLD))
            .opacity(170)
            .padding(1),
    );
    app.add(AuraOrb::new().color(PEACH));
    app.add(CometOrb::new().color(SKY));
    app.add(SpinnerOrb::new().color(YELLOW));
    app.add(
        ThinkingOrb::new()
            .state(OrbState::Searching)
            .size(OrbSize::Size20),
    );

    // Third row: a tall glass panel with wrap-around title and content.
    app.add(
        Panel::new()
            .title("crisp transparency: this long title wraps around the rounded border corners")
            .borders(BorderType::Rounded)
            .border_style(Style::DEFAULT.fg(MAGENTA).opacity(210))
            .content_style(Style::DEFAULT.fg(TEXT).opacity(200))
            .title_style(Style::DEFAULT.fg(MAGENTA).mods(Modifiers::BOLD))
            .opacity(160)
            .padding(1)
            .content([
                "Semi-transparent panels blend over the canvas background.",
                "The canvas is set to mocha base (#1e1e2e) via",
                "App::set_canvas_colors, so glass looks crisp on dark",
                "terminals. Adjust it to match your theme.",
                "",
                "Orbs pulse, spin, comet-trail and glow: an orb-like",
                "interface for AI agent harnesses.",
            ]),
    );

    app.run()?;
    Ok(())
}
