//! A demo of booms view transitions: fade, slide, and cube wipes between
//! full-screen views, driven by eased [`Anim`](booms::anim::Anim) progress.
//!
//! Press any key to cycle transition type and switch view.
//! Press Escape or Ctrl+C to quit.
//!
//! Run with: `cargo run --example transitions`

use std::io;
use std::time::Duration;

use booms::Transition;
use booms::grid::{Cell, Grid};
use booms::layout::{Constraint, Rect, layout_col};
use booms::style::{Color, Style};
use booms::term::Term;
use booms::transitions::{SlideDirection, composite};
use booms::widgets::{Input, List, PulseOrb, Widget};

fn title_line(text: &str) -> impl Widget {
    struct Line {
        text: String,
    }
    impl Widget for Line {
        fn constraint(&self) -> Constraint {
            Constraint::Exact(1)
        }
        fn render(&self, area: Rect, grid: &mut Grid) {
            let y = area.y;
            if y >= area.bottom() {
                return;
            }
            grid.set_str(area.x, y, &self.text, Style::DEFAULT.fg(Color::Ansi(14)));
        }
    }
    Line {
        text: text.to_string(),
    }
}

struct Demo {
    term: Term,
    grid: Grid,
    views: Vec<Vec<Box<dyn Widget>>>,
    current: usize,
    transition: Option<Transition>,
    transition_from: Option<Grid>,
    tick: Duration,
}

impl Demo {
    fn new() -> io::Result<Demo> {
        let mut term = Term::enter()?;
        let (w, h) = term.size();
        let gallery: Vec<Box<dyn Widget>> = vec![
            Box::new(title_line("orb gallery")),
            Box::new(PulseOrb::new(Color::Ansi(10))),
            Box::new(PulseOrb::new(Color::Ansi(11))),
            Box::new(PulseOrb::new(Color::Ansi(9))),
            Box::new(PulseOrb::new(Color::Ansi(13))),
            Box::new(
                List::new(
                    [
                        "press any key: cycle transition + switch view",
                        "escape or ctrl+c: quit",
                    ]
                    .map(|s| s.to_string()),
                )
                .style(Style::DEFAULT.fg(Color::Ansi(7))),
            ),
            Box::new(Input::new().placeholder("type here")),
        ];
        let detail: Vec<Box<dyn Widget>> = vec![
            Box::new(title_line("detail view")),
            Box::new(
                List::new(
                    [
                        "you transitioned here!",
                        "press any key: return to gallery",
                        "escape or ctrl+c: quit",
                    ]
                    .map(|s| s.to_string()),
                )
                .style(Style::DEFAULT.fg(Color::Ansi(14))),
            ),
            Box::new(Input::new().placeholder("type here")),
        ];
        Ok(Demo {
            grid: Grid::new(w, h, Cell::blank()),
            term,
            views: vec![gallery, detail],
            current: 0,
            transition: None,
            transition_from: None,
            tick: Duration::from_millis(50),
        })
    }

    fn render_view(widgets: &[Box<dyn Widget>], grid: &mut Grid) {
        let constraints: Vec<Constraint> = widgets.iter().map(|w| w.constraint()).collect();
        let rects = layout_col(Rect::new(0, 0, grid.width(), grid.height()), &constraints);
        for (widget, rect) in widgets.iter().zip(rects.iter()) {
            widget.render(*rect, grid);
        }
    }

    fn render(&mut self) -> io::Result<()> {
        if self.transition.is_some() {
            self.grid.clear(Style::DEFAULT);
            Self::render_view(&self.views[self.current], &mut self.grid);
            let from = self.transition_from.as_ref().unwrap();
            let mut composite_grid = self.grid.clone();
            if let Some(t) = self.transition.as_mut() {
                composite(t, from, &self.grid, &mut composite_grid);
            }
            self.grid = composite_grid;
        } else {
            self.grid.clear(Style::DEFAULT);
            Self::render_view(&self.views[self.current], &mut self.grid);
        }
        self.term.draw(&self.grid)?;
        Ok(())
    }

    fn switch_view(&mut self, new_current: usize, transition: Transition) {
        self.transition_from = Some(self.grid.clone());
        self.current = new_current;
        self.transition = Some(transition);
    }

    fn run(mut self) -> io::Result<()> {
        self.render()?;
        let transitions: [Transition; 4] = [
            Transition::fade(Duration::from_millis(700)),
            Transition::slide(Duration::from_millis(700), SlideDirection::Left),
            Transition::slide(Duration::from_millis(700), SlideDirection::Right),
            Transition::cube(Duration::from_millis(700)),
        ];
        let mut ti = 0usize;

        loop {
            if self.term.poll(self.tick)? {
                match self.term.read()? {
                    crossterm::event::Event::Key(key) => match key.code {
                        crossterm::event::KeyCode::Esc => break,
                        crossterm::event::KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        _ => {
                            let target = if self.current == 0 { 1 } else { 0 };
                            self.switch_view(target, transitions[ti].clone());
                            ti = (ti + 1) % transitions.len();
                        }
                    },
                    crossterm::event::Event::Resize(w, h) => {
                        self.grid = Grid::new(w, h, Cell::blank());
                        if let Some(from) = &mut self.transition_from {
                            *from = Grid::new(w, h, Cell::blank());
                        }
                    }
                    _ => {}
                }
            }
            for widget in self.views[self.current].iter_mut() {
                widget.on_tick();
            }
            if let Some(transition) = &mut self.transition {
                if transition.tick() {
                    self.transition = None;
                    self.transition_from = None;
                }
            }
            self.render()?;
        }

        let _ = self.term.leave();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let demo = Demo::new()?;
    demo.run()?;
    Ok(())
}
