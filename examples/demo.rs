//! A demo of the framework: a fade-in banner, a ticking clock, a bouncing
//! orb of dots, a selectable list and a text input.

use std::time::Duration;

use booms::anim::{Anim, Easing, Spring};
use booms::app::App;
use booms::grid::Grid;
use booms::layout::{Constraint, Rect};
use booms::style::{Color, Modifiers, Style};
use booms::widgets::{Block, Borders, Input, List, Widget};

/// A banner that fades in from a dim color with an eased transition.
struct Banner {
    lines: Vec<String>,
    fade: Anim<Color>,
}

impl Banner {
    fn new(lines: Vec<String>) -> Banner {
        Banner {
            lines,
            fade: Anim::new(
                Color::Rgb(30, 30, 30),
                Color::Rgb(210, 210, 210),
                Duration::from_millis(800),
            )
            .easing(Easing::SmoothStep),
        }
    }
}

impl Widget for Banner {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(self.lines.len() as u16)
    }

    fn on_tick(&mut self) {
        self.fade.update();
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let style = Style::DEFAULT.fg(*self.fade.value());
        for (i, line) in self.lines.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.bottom() {
                return;
            }
            grid.set_str(area.x, y, line, style);
        }
    }
}

/// A clock that ticks every interval.
struct Clock {
    ticks: u64,
}

impl Clock {
    fn new() -> Clock {
        Clock { ticks: 0 }
    }
}

impl Widget for Clock {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn on_tick(&mut self) {
        self.ticks += 1;
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let text = format!("ticks: {}", self.ticks);
        grid.set_str(area.x, area.y, &text, Style::DEFAULT.fg(Color::Ansi(6)));
    }
}

/// A pulsing orb: a row of dots that bounces between two rows, cycling
/// shape (● ○ ▲): driven by eased [`Anim`] transitions.
struct Orb {
    n: usize,
    rise: Anim<f32>,
    cycle: Anim<u8>,
}

const ORB_SHAPES: [(char, Color); 3] = [
    ('●', Color::Ansi(3)),
    ('○', Color::Ansi(3)),
    ('▲', Color::Ansi(3)),
];

impl Orb {
    fn new(n: usize) -> Orb {
        Orb {
            n,
            rise: Anim::new(1.0, 0.0, Duration::from_millis(400)).easing(Easing::SineOut),
            cycle: Anim::new(0u8, 2u8, Duration::from_millis(2100)),
        }
    }
}

impl Widget for Orb {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(3)
    }

    fn on_tick(&mut self) {
        if self.rise.update() >= 1.0 {
            self.rise.restart();
        }
        self.cycle.update();
        if self.cycle.is_finished() {
            self.cycle.restart();
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let center = area.y + 1 + (1.0 - *self.rise.value()) as u16;
        let (ch, color) = ORB_SHAPES[(*self.cycle.value()).min(2) as usize];
        let style = Style::DEFAULT.fg(color);
        for i in 0..self.n {
            grid.set(area.x + 1 + i as u16, center, ch, style);
        }
    }
}

/// A block whose top and bottom borders carry traveling light beams:
/// a terminal-flavored port of [`border-beam`]. The beam position moves
/// along the border with spring-physics transitions.
///
/// [`border-beam`]: https://github.com/Libraries.dev: referenced in this
/// framework's workspace.
struct BeamedBlock {
    block: Block,
    top: Spring,
    bottom: Spring,
}

impl BeamedBlock {
    fn new(title: &str) -> BeamedBlock {
        BeamedBlock {
            block: Block::new().title(title).borders(Borders::ALL),
            top: Spring::new(0.0, 1.0, 500.0, 110.0, 1.0),
            bottom: Spring::new(1.0, 0.0, 500.0, 110.0, 1.0),
        }
    }
}

impl Widget for BeamedBlock {
    fn on_tick(&mut self) {
        self.top.tick();
        if self.top.is_settled() {
            let to = if self.top.target() == 1.0 { 0.0 } else { 1.0 };
            self.top.set_target(to);
        }
        self.bottom.tick();
        if self.bottom.is_settled() {
            let to = if self.bottom.target() == 0.0 {
                1.0
            } else {
                0.0
            };
            self.bottom.set_target(to);
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        self.block.render(area, grid);
        if area.width == 0 || area.height == 0 {
            return;
        }
        let span = (area.width as f32 - 1.0).max(1.0);
        let top = Style::DEFAULT.mods(Modifiers::BOLD).fg(Color::Ansi(14));
        let pos = (self.top.value().clamp(0.0, 1.0) * span).round() as u16;
        for dx in 0..area.width {
            if dx.abs_diff(pos) <= 1 {
                grid.set(area.x + dx, area.y, '─', top);
            }
        }
        let bottom = Style::DEFAULT.mods(Modifiers::BOLD).fg(Color::Ansi(11));
        let pos = (self.bottom.value().clamp(0.0, 1.0) * span).round() as u16;
        for dx in 0..area.width {
            if dx.abs_diff(pos) <= 1 {
                grid.set(area.x + dx, area.y + area.height - 1, '─', bottom);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;
    app.add(Banner::new(vec![
        "tui framework".to_string(),
        "with animations and transitions".to_string(),
    ]));
    app.add(BeamedBlock::new("editor"));
    app.add(Clock::new());
    app.add(Orb::new(8));
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
