//! The terminal backend: alternate screen, raw mode, diffed output.

use std::io::{self, Write};
use std::mem;
use std::time::Duration;

use crossterm::event::{
    self, DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture, Event,
};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};

use crate::grid::{Cell, Grid};
use crate::style::{Color, Modifiers, Style};

/// The terminal backend: enters the alternate screen, puts the terminal in
/// raw mode, hides the cursor, and writes diffs of the back-buffer grid.
pub struct Term {
    last: Grid,
    sgr: Option<Style>,
    left: bool,
}

impl Term {
    /// Enter the alternate screen and start raw mode; paint a blank frame.
    pub fn enter() -> io::Result<Term> {
        let (w, h) = terminal::size()?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste,
            EnableFocusChange
        )?;
        write_ansi(b"\x1b[?25l")?; // hide cursor
        terminal::enable_raw_mode()?;
        Ok(Term {
            last: Grid::new(w, h, Cell::blank()),
            sgr: None,
            left: false,
        })
    }

    /// The current terminal size `(width, height)`, falling back to the
    /// last known size if the query fails.
    pub fn size(&mut self) -> (u16, u16) {
        terminal::size().unwrap_or_else(|_| (self.last.width(), self.last.height()))
    }

    /// Write the changed cells of `grid` to the terminal.
    ///
    /// Compares against the previous frame and only emits cells that differ,
    /// resetting style state when it changes.
    pub fn draw(&mut self, grid: &Grid) -> io::Result<()> {
        let mut out = io::stdout();
        let old = mem::replace(&mut self.last, grid.clone());
        let mut buf = [0u8; 4];
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let cell = grid.get(x, y);
                if cell == old.get(x, y) {
                    continue;
                }
                if self.sgr != Some(cell.style) {
                    out.write_all(sgr_for(cell.style).as_bytes())?;
                    self.sgr = Some(cell.style);
                }
                out.write_fmt(format_args!("\x1b[{};{}H", y + 1, x + 1))?;
                out.write_all(cell.ch.encode_utf8(&mut buf).as_bytes())?;
            }
        }
        out.flush()
    }

    /// Whether an event is waiting. See [`crossterm::event::poll`].
    pub fn poll(&self, dur: Duration) -> io::Result<bool> {
        event::poll(dur)
    }

    /// Read the next event. See [`crossterm::event::read`].
    pub fn read(&self) -> io::Result<Event> {
        event::read()
    }

    /// Move the terminal cursor to 0-based `(x, y)`.
    pub fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        let mut out = io::stdout();
        out.write_fmt(format_args!("\x1b[{};{}H", y + 1, x + 1))?;
        out.flush()
    }

    /// Leave the alternate screen, restore the cursor, and leave raw mode.
    /// Idempotent.
    pub fn leave(&mut self) -> io::Result<()> {
        if self.left {
            return Ok(());
        }
        let mut out = io::stdout();
        execute!(
            out,
            DisableMouseCapture,
            DisableBracketedPaste,
            DisableFocusChange,
            LeaveAlternateScreen
        )?;
        write_ansi(b"\x1b[?25h")?; // show cursor
        terminal::disable_raw_mode()?;
        out.flush()?;
        self.left = true;
        Ok(())
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        let _ = self.leave();
    }
}

fn write_ansi(bytes: &[u8]) -> io::Result<()> {
    let mut out = io::stdout();
    out.write_all(bytes)?;
    out.flush()
}

/// The SGR escape sequence for a style, prefixed with a reset.
fn sgr_for(style: Style) -> String {
    let mut s = String::from("\x1b[0m");
    match style.fg {
        Color::Default => {}
        Color::Ansi(n) if n < 8 => s.push_str(&format!("\x1b[{}m", 30 + n)),
        Color::Ansi(n) => s.push_str(&format!("\x1b[{}m", 90 + n - 8)),
        Color::Ansi256(n) => s.push_str(&format!("\x1b[38;5;{}m", n)),
        Color::Rgb(r, g, b) => s.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b)),
    }
    match style.bg {
        Color::Default => {}
        Color::Ansi(n) if n < 8 => s.push_str(&format!("\x1b[{}m", 40 + n)),
        Color::Ansi(n) => s.push_str(&format!("\x1b[{}m", 100 + n - 8)),
        Color::Ansi256(n) => s.push_str(&format!("\x1b[48;5;{}m", n)),
        Color::Rgb(r, g, b) => s.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b)),
    }
    for (code, flag) in [
        (1u8, Modifiers::BOLD),
        (2, Modifiers::DIM),
        (3, Modifiers::ITALIC),
        (4, Modifiers::UNDERLINE),
        (5, Modifiers::BLINK),
        (7, Modifiers::REVERSE),
        (9, Modifiers::STRIKETHROUGH),
    ] {
        if style.mods.contains(flag) {
            s.push_str(&format!("\x1b[{}m", code));
        }
    }
    s
}
