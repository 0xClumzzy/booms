//! Showcase: real ratatui widgets rendered inside booms via [`RatatuiView`].
//!
//! Run with `cargo run --features ratatui --example native_ratatui`.
//!
//! Press Escape to quit. The mouse works: click the list to focus it.

use booms::app::App;
use booms::widgets::{RatatuiView, Text};
use ratatui::widgets::{Block, Borders, Gauge, List, Paragraph, Tabs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;

    app.add(Text::new(["native ratatui widgets inside booms"]));
    app.add(RatatuiView::new(
        Paragraph::new("This paragraph is a real ratatui widget, styled with truecolor.")
            .block(Block::bordered().title("ratatui Paragraph")),
    ));
    app.add(RatatuiView::new(
        Gauge::default()
            .block(Block::bordered().title("ratatui Gauge"))
            .ratio(0.72)
            .gauge_style(ratatui::style::Style::default().fg(ratatui::style::Color::Magenta)),
    ));
    app.add(RatatuiView::new(
        Tabs::new(["overview", "metrics", "logs"])
            .block(Block::bordered().title("ratatui Tabs"))
            .select(1),
    ));
    app.add(RatatuiView::new(
        List::new([
            "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
        ])
        .block(Block::default().borders(Borders::ALL).title("ratatui List")),
    ));

    app.add(Text::new(["status: ready"]));

    app.run()?;
    Ok(())
}
