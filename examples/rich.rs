//! A showcase of booms' "rich" widgets: rounded and square panels whose
//! titles wrap around the border, a gallery of thinking orbs, and a set of
//! ratatui-inspired data widgets (gauge, sparkline, bar chart, tabs, table).
//!
//! Run with `cargo run --example rich`. Click a line in the list (or use the
//! mouse wheel) to see mouse dispatch in action; press Escape to quit.

use booms::app::App;
use booms::style::{Color, Style};
use booms::widgets::{
    AuraOrb, BarChart, BorderType, CometOrb, Gauge, HeartbeatOrb, Panel, Sparkline, SpinnerOrb,
    Table, Tabs, TitleAlignment, TitlePosition,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;

    // A rounded panel: its long title wraps around the corners of the border.
    app.add(
        Panel::new()
            .borders(BorderType::Rounded)
            .title("Rounded: a title long enough to wrap around the corners of the border")
            .title_style(Style::DEFAULT.fg(Color::Rgb(245, 194, 231)))
            .content([
                "Round corners, square content.",
                "Titles follow the border perimeter,",
                "continuing past ╭ ─ ╮ and ╰ ─ ╯.",
            ])
            .border_style(Style::DEFAULT.fg(Color::Rgb(203, 166, 247)))
            .padding(1),
    );

    // Expanding progress bars and a sparkline.
    app.add(Gauge::new(0.68).label("render progress").ratio(0.68));
    app.add(Sparkline::new([
        1u64, 4, 9, 16, 25, 36, 49, 64, 49, 36, 25, 16, 9, 4, 1,
    ]));

    // A square block with a bottom-centered title and animated orb gallery.
    app.add(
        Panel::new()
            .borders(BorderType::Plain)
            .title("Thinking orbs")
            .title_position(TitlePosition::Bottom)
            .title_alignment(TitleAlignment::Center)
            .content(["spinner  aura  comet  heartbeat"])
            .border_style(Style::DEFAULT.fg(Color::Rgb(137, 180, 250))),
    );
    app.add(SpinnerOrb::new().color(Color::Rgb(166, 227, 161)));
    app.add(AuraOrb::new().color(Color::Rgb(137, 220, 235)));
    app.add(CometOrb::new().color(Color::Rgb(250, 179, 135)));
    app.add(HeartbeatOrb::new().color(Color::Rgb(243, 139, 168)));

    // Ratatui-inspired data widgets.
    app.add(Tabs::new(["overview", "metrics", "logs"]).selected(0));
    app.add(BarChart::new([
        ("cpu", 42u64),
        ("mem", 73),
        ("net", 21),
        ("disk", 57),
    ]));
    app.add(Table::new(
        vec!["name".into(), "state".into(), "ms".into()],
        vec![
            vec!["alpha".into(), "ok".into(), "12".into()],
            vec!["beta".into(), "warm".into(), "34".into()],
            vec!["gamma".into(), "ok".into(), "8".into()],
        ],
    ));

    // A double-bordered panel with wrapping content.
    app.add(
        Panel::new()
            .borders(BorderType::Double)
            .title("notes")
            .content([
                "wgpu GPU rendering is available behind the `gpu` feature.",
                "Run `cargo run --features gpu --example gpu` to see plasma.",
            ])
            .border_style(Style::DEFAULT.fg(Color::Rgb(148, 226, 213))),
    );

    app.run()?;
    Ok(())
}
