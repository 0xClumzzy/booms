//! The application event loop: polls input, ticks widgets, renders.
//! Supports a view stack with animated transitions between views.

use std::io;
use std::time::Duration;

use crossterm::event::{Event, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::grid::{Cell, Grid};
use crate::layout::{Constraint, Rect, layout_col};
use crate::style::{Color, Style};
use crate::term::Term;
use crate::transitions::{Transition, composite};
use crate::widgets::Widget;

/// The app event loop.
///
/// Widgets are added with [`App::add`], laid out as a column on the screen,
/// polled for input, ticked (for clocks and animations), and redrawn every
/// [`App::tick_interval`].
///
/// Views can be pushed and popped with [`App::push_view`] and
/// [`App::pop_view`], which run an animated [`Transition`] between the
/// outgoing and incoming views.
pub struct App {
    pub term: Term,
    pub grid: Grid,
    /// Stack of views; each view is a column of widgets. The last entry is
    /// the currently visible view.
    pub views: Vec<Vec<Box<dyn Widget>>>,
    pub focused: Option<usize>,
    /// How often the loop polls input; also the rate of widget ticks.
    pub tick_interval: Duration,
    /// Set to quit the loop.
    pub should_quit: bool,
    /// Active transition between views, if any.
    pub transition: Option<Transition>,
    /// Snapshot of the grid when a transition began (the outgoing view).
    pub transition_from: Option<Grid>,
    /// Canvas foreground color semi-transparent cells blend toward.
    pub canvas_fg: Color,
    /// Canvas background color semi-transparent cells blend toward.
    pub canvas_bg: Color,
}

impl App {
    /// Open the alternate screen and build the app with a single empty view.
    pub fn new() -> io::Result<App> {
        let mut term = Term::enter()?;
        let (w, h) = term.size();
        let mut grid = Grid::new(w, h, Cell::blank());
        grid.set_canvas_colors(Color::Default, Color::Default);
        Ok(App {
            grid,
            term,
            views: vec![Vec::new()],
            focused: None,
            tick_interval: Duration::from_millis(50),
            should_quit: false,
            transition: None,
            transition_from: None,
            canvas_fg: Color::Default,
            canvas_bg: Color::Default,
        })
    }

    /// Set the canvas colors that semi-transparent cells (glass panels,
    /// transparent orbs) blend toward. Defaults to black; set these to match
    /// your terminal theme (e.g. Catppuccin mocha base `#1e1e2e`) so
    /// transparency looks crisp against it.
    pub fn set_canvas_colors(&mut self, fg: Color, bg: Color) -> &mut App {
        self.canvas_fg = fg;
        self.canvas_bg = bg;
        self.grid.set_canvas_colors(fg, bg);
        self
    }

    /// Add a widget to the current (top) view; it becomes the last row of
    /// the layout.
    pub fn add(&mut self, widget: impl Widget + 'static) -> &mut App {
        self.current_view_mut().push(Box::new(widget));
        self
    }

    /// Push a new view onto the stack with an animated transition. The new
    /// view's widgets replace the current view's widgets during the
    /// transition; when the transition completes, the old view is discarded.
    pub fn push_view(&mut self, transition: Transition, widgets: Vec<Box<dyn Widget>>) -> &mut App {
        // Snapshot the current grid (outgoing view) before switching.
        self.transition_from = Some(self.grid.clone());
        // Replace the current view's widgets with the new view's.
        self.current_view_mut().clear();
        self.current_view_mut().extend(widgets);
        self.transition = Some(transition);
        // Reset focus for the new view.
        self.focused = None;
        self
    }

    /// Pop the current view, transitioning back to the previous view. If
    /// there is only one view, this is a no-op.
    pub fn pop_view(&mut self, transition: Transition) -> &mut App {
        if self.views.len() <= 1 {
            return self;
        }
        self.transition_from = Some(self.grid.clone());
        self.views.pop();
        self.transition = Some(transition);
        self.focused = None;
        self
    }

    /// A reference to the widgets of the current (top) view.
    pub fn widgets(&self) -> &[Box<dyn Widget>] {
        self.views.last().map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// The current view's widgets mutably.
    fn current_view_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
        self.views.last_mut().unwrap()
    }

    pub fn widget(&self, i: usize) -> &dyn Widget {
        &*self.views.last().unwrap()[i]
    }

    pub fn widget_mut(&mut self, i: usize) -> &mut dyn Widget {
        &mut *self.views.last_mut().unwrap()[i]
    }

    /// The current back-buffer grid.
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn size(&self) -> (u16, u16) {
        (self.grid.width(), self.grid.height())
    }

    pub fn focused(&self) -> Option<usize> {
        self.focused
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Run the event loop until [`quit`](App::quit) or an error.
    pub fn run(&mut self) -> io::Result<()> {
        let result = self.run_loop();
        let _ = self.term.leave();
        result
    }

    fn run_loop(&mut self) -> io::Result<()> {
        self.ensure_focus();
        self.render()?;
        loop {
            if self.term.poll(self.tick_interval)? {
                match self.term.read()? {
                    Event::Key(key) => self.handle_key(&key),
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    Event::Paste(text) => self.handle_paste(&text),
                    Event::Resize(w, h) => {
                        self.grid = Grid::new(w, h, Cell::blank());
                        self.grid.set_canvas_colors(self.canvas_fg, self.canvas_bg);
                        // Also resize transition snapshot if present.
                        if let Some(from) = &mut self.transition_from {
                            *from = Grid::new(w, h, Cell::blank());
                        }
                    }
                    _ => {}
                }
            }
            // Tick all widgets in the current view.
            for widget in self.current_view_mut().iter_mut() {
                widget.on_tick();
            }
            // Advance any active transition.
            if let Some(transition) = &mut self.transition {
                if transition.tick() {
                    self.transition = None;
                    self.transition_from = None;
                }
            }
            self.render()?;
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn render(&mut self) -> io::Result<()> {
        let focused = self.focused;
        let (views, grid) = (&mut self.views, &mut self.grid);
        let current = views.last_mut().unwrap();

        if self.transition.is_some() {
            // During a transition: render the incoming view into grid,
            // then composite it with the outgoing view snapshot.
            grid.clear(Style::DEFAULT);
            let constraints: Vec<Constraint> = current.iter().map(|w| w.constraint()).collect();
            let rects = layout_col(Rect::new(0, 0, grid.width(), grid.height()), &constraints);
            let mut cursor: Option<(u16, u16)> = None;
            for (i, (widget, rect)) in current.iter().zip(rects.iter()).enumerate() {
                if focused == Some(i) {
                    cursor = widget.cursor_pos(*rect);
                }
                widget.render(*rect, grid);
            }
            let from = self.transition_from.as_ref().unwrap();
            let mut composite_grid = grid.clone();
            if let Some(t) = self.transition.as_mut() {
                composite(t, from, grid, &mut composite_grid);
            }
            *grid = composite_grid;
            self.term.draw(grid)?;
            if let Some((x, y)) = cursor {
                self.term.set_cursor(x, y)?;
            }
            Ok(())
        } else {
            // Normal render: clear, render all widgets in the current view.
            grid.clear(Style::DEFAULT);
            let constraints: Vec<Constraint> = current.iter().map(|w| w.constraint()).collect();
            let rects = layout_col(Rect::new(0, 0, grid.width(), grid.height()), &constraints);
            let mut cursor: Option<(u16, u16)> = None;
            for (i, (widget, rect)) in current.iter().zip(rects.iter()).enumerate() {
                if focused == Some(i) {
                    cursor = widget.cursor_pos(*rect);
                }
                widget.render(*rect, grid);
            }
            self.term.draw(grid)?;
            if let Some((x, y)) = cursor {
                self.term.set_cursor(x, y)?;
            }
            Ok(())
        }
    }

    fn handle_key(&mut self, key: &KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Tab => self.focus_next(),
            crossterm::event::KeyCode::BackTab => self.focus_prev(),
            crossterm::event::KeyCode::Esc => self.should_quit = true,
            crossterm::event::KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.should_quit = true
            }
            _ => {
                if let Some(i) = self.focused {
                    self.views.last_mut().unwrap()[i].on_key(*key);
                }
            }
        }
    }

    /// Dispatch a mouse event to the widget whose layout rect contains the
    /// cursor. A left-click-down on a focusable widget also focuses it.
    fn handle_mouse(&mut self, event: MouseEvent) {
        let (w, h) = (self.grid.width(), self.grid.height());
        let constraints: Vec<Constraint> = self
            .views
            .last()
            .unwrap()
            .iter()
            .map(|widget| widget.constraint())
            .collect();
        let rects = layout_col(Rect::new(0, 0, w, h), &constraints);

        // Locate the widget under the cursor. Rects are laid out top-to-bottom
        // in a column, so the first containing rect is the target.
        let target = rects.iter().position(|r| {
            event.column >= r.x
                && event.column < r.right()
                && event.row >= r.y
                && event.row < r.bottom()
        });

        if let Some(i) = target {
            // A click focuses focusable widgets (buttons, inputs, lists).
            if matches!(event.kind, MouseEventKind::Down(MouseButton::Left)) {
                let is_focusable = self.views.last().unwrap()[i].is_focusable();
                if is_focusable {
                    self.set_focus(Some(i));
                }
            }
            let rect = rects[i];
            self.views.last_mut().unwrap()[i].on_mouse(event, rect);
        }
    }

    /// Forward a bracketed paste to the focused widget as individual key
    /// events so text inputs receive it naturally.
    fn handle_paste(&mut self, text: &str) {
        let keys: Vec<KeyEvent> = text
            .chars()
            .filter(|c| *c != '\r' && *c != '\n')
            .map(|c| KeyEvent::new(crossterm::event::KeyCode::Char(c), KeyModifiers::NONE))
            .collect();
        if let Some(i) = self.focused {
            let widget = &mut self.views.last_mut().unwrap()[i];
            for key in keys {
                widget.on_key(key);
            }
        }
    }

    fn ensure_focus(&mut self) {
        if self.focused.is_none() {
            let focusable_idx = {
                let current = self.views.last().unwrap();
                current.iter().position(|w| w.is_focusable())
            };
            if let Some(i) = focusable_idx {
                self.focused = Some(i);
                self.views.last_mut().unwrap()[i].set_focus(true);
            }
        }
    }

    fn set_focus(&mut self, i: Option<usize>) {
        // Clear old focus.
        if let Some(old) = self.focused {
            if self.views.last().unwrap()[old].is_focusable() {
                self.views.last_mut().unwrap()[old].set_focus(false);
            }
        }
        self.focused = i;
        if let Some(i) = i {
            if self.views.last().unwrap()[i].is_focusable() {
                self.views.last_mut().unwrap()[i].set_focus(true);
            }
        }
    }

    fn focus_next(&mut self) {
        let n = self.views.last().unwrap().len();
        if n == 0 {
            return;
        }
        let start = self.focused.unwrap_or(0);
        for k in 1..=n {
            let i = (start + k) % n;
            if self.views.last().unwrap()[i].is_focusable() {
                self.set_focus(Some(i));
                return;
            }
        }
    }

    fn focus_prev(&mut self) {
        let n = self.views.last().unwrap().len();
        if n == 0 {
            return;
        }
        let start = self.focused.unwrap_or(n);
        for k in 1..=n {
            let i = start.wrapping_sub(k) % n;
            if self.views.last().unwrap()[i].is_focusable() {
                self.set_focus(Some(i));
                return;
            }
        }
    }
}
