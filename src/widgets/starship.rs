//! Starship-style prompt: a Rust port of the "Noctalia" trailing-bubble
//! capsule prompt from a `starship.toml`, built with booms primitives.
//!
//! Each module is a capsule: `<icon> <content>▸`, colored with the
//! Noctalia palette (Catppuccin Mocha–inspired). A [`StarshipPrompt`] paints
//! a line of capsules plus the character indicator; [`StarshipInput`]
//! pairs it with an [`Input`] so an agent harness gets a full prompt line,
//! including the right-side `$time` capsule.
//!
//! Nerd Font glyphs are used for the bubble arcs and module icons, matching
//! the source config. Render them in a Nerd Font terminal.

use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::event::KeyEvent;

use crate::grid::{Grid, str_width};
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Modifiers, Style};
use crate::widgets::Widget;
use crate::widgets::input::Input;

/// The Catppuccin-Mocha-inspired `noctalia` palette from `starship.toml`.
pub mod noctalia {
    use crate::style::Color;

    pub const BRIGHT: Color = Color::Rgb(240, 240, 240);
    pub const GREEN: Color = Color::Rgb(166, 227, 161);
    pub const PEACH: Color = Color::Rgb(250, 179, 135);
    pub const MAUVE: Color = Color::Rgb(203, 166, 247);
    pub const BLUE: Color = Color::Rgb(137, 180, 250);
    pub const YELLOW: Color = Color::Rgb(249, 226, 175);
    pub const RED: Color = Color::Rgb(243, 139, 168);
    pub const CYAN: Color = Color::Rgb(148, 226, 213);
    pub const MAGENTA: Color = Color::Rgb(245, 194, 231);
    pub const SKY: Color = Color::Rgb(137, 220, 235);
    pub const OVERLAY0: Color = Color::Rgb(108, 112, 134);
}

/// ``: left bubble arc.
pub const BUBBLE_START: char = '\u{e0b7}';
/// ``: right bubble arc.
pub const BUBBLE_END: char = '\u{e0b5}';
/// `▸`: trailing flow arrow.
pub const FLOW_ARROW: char = '\u{25b8}';
/// `◂`: leading arrow for right-side capsules.
pub const FLOW_ARROW_LEFT: char = '\u{25c2}';

/// Nerd Font icons used by the source config.
pub mod icon {
    pub const USER: char = '\u{f08c7}'; // 󰣇
    pub const HOST: char = '\u{f08c7}'; // 󰢩 (SSH host)
    pub const DIRECTORY: char = '\u{f024b}'; // 󰉋
    pub const RUST: char = '\u{f1c17}'; // 󱘗
    pub const BATTERY: char = '\u{f00b9}'; // 󰁹
    pub const CLOCK: char = '\u{f0d94}'; // 󰥔
    pub const MODEL: char = '\u{2726}'; // ✦: agent model capsule
}

/// One trailing-bubble capsule: `<icon> <content>▸`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capsule {
    icon: char,
    text: String,
    color: Color,
    bold: bool,
}

impl Capsule {
    pub fn new(icon: char, text: impl Into<String>, color: Color) -> Capsule {
        Capsule {
            icon,
            text: text.into(),
            color,
            bold: true,
        }
    }

    /// Whether the icon and content are drawn bold.
    pub fn bold(mut self, bold: bool) -> Capsule {
        self.bold = bold;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn color(&self) -> Color {
        self.color
    }

    /// Number of terminal columns the capsule occupies.
    pub fn width(&self) -> u16 {
        (str_width(&self.text) + 5) as u16
    }

    fn paint(&self, x: u16, y: u16, grid: &mut Grid) {
        let arc = Style::DEFAULT.fg(self.color);
        let content = if self.bold {
            arc.mods(Modifiers::BOLD)
        } else {
            arc
        };
        let mut cx = x;
        grid.set(cx, y, BUBBLE_START, arc);
        cx += 1;
        grid.set(cx, y, self.icon, content);
        cx += 1;
        grid.set(cx, y, ' ', content);
        cx += 1;
        cx = grid.set_str(cx, y, &self.text, content);
        grid.set(cx, y, BUBBLE_END, arc);
        cx += 1;
        grid.set(cx, y, FLOW_ARROW, arc);
    }
}

/// A line of starship capsules plus the prompt character (`❯ `).
///
/// ```
/// use booms::widgets::StarshipPrompt;
/// use booms::widgets::starship::{icon, noctalia};
///
/// let prompt = StarshipPrompt::new()
///     .user("clumzzy")
///     .directory("~/Projects/mytui/booms")
///     .rust("v1.97")
///     .character("❯ ", noctalia::GREEN);
/// assert!(prompt.width() > 10);
/// ```
#[derive(Clone, Debug)]
pub struct StarshipPrompt {
    capsules: Vec<Capsule>,
    character: String,
    character_color: Color,
}

impl Default for StarshipPrompt {
    fn default() -> StarshipPrompt {
        StarshipPrompt::new()
    }
}

impl StarshipPrompt {
    /// An empty prompt with a green `❯ ` character (matching
    /// `style_success = "fg:green bold"` in the config).
    pub fn new() -> StarshipPrompt {
        StarshipPrompt {
            capsules: Vec::new(),
            character: "❯ ".to_string(),
            character_color: noctalia::GREEN,
        }
    }

    /// Append a raw capsule.
    pub fn capsule(mut self, icon: char, text: impl Into<String>, color: Color) -> StarshipPrompt {
        self.capsules.push(Capsule::new(icon, text, color));
        self
    }

    /// The `$username` capsule (`fg:purple bold` → magenta in this palette).
    pub fn user(self, name: &str) -> StarshipPrompt {
        self.capsule(icon::USER, name, noctalia::MAGENTA)
    }

    /// The `$hostname` capsule (SSH-only in starship; included explicitly).
    pub fn host(self, name: &str) -> StarshipPrompt {
        self.capsule(icon::HOST, name, noctalia::CYAN)
    }

    /// The `$directory` capsule (`fg:green bold`).
    pub fn directory(self, path: &str) -> StarshipPrompt {
        self.capsule(icon::DIRECTORY, path, noctalia::GREEN)
    }

    /// The `$rust` capsule (`fg:peach`, non-bold).
    pub fn rust(self, version: &str) -> StarshipPrompt {
        self.capsule(icon::RUST, version, noctalia::PEACH)
            .bold_capsule(false)
    }

    /// The `$battery` capsule with the config's threshold colors:
    /// 100→green, 75→teal, 50→yellow, 25→peach, ≤15→red bold, 0→red bold.
    pub fn battery(self, pct: u8) -> StarshipPrompt {
        let pct = pct.min(100);
        let (color, bold) = match pct {
            0..=15 => (noctalia::RED, true),
            16..=25 => (noctalia::PEACH, false),
            26..=50 => (noctalia::YELLOW, false),
            51..=75 => (noctalia::CYAN, false),
            _ => (noctalia::GREEN, false),
        };
        self.capsule(icon::BATTERY, format!("{}%", pct), color)
            .bold_capsule(bold)
    }

    /// The `$os` capsule (`fg:magenta`).
    pub fn os(self) -> StarshipPrompt {
        self.capsule(icon::USER, "", noctalia::MAGENTA)
    }

    /// An agent-specific model capsule (blue, bold).
    pub fn model(self, name: &str) -> StarshipPrompt {
        self.capsule(icon::MODEL, name, noctalia::BLUE)
    }

    /// Set the bold flag on the most recently added capsule.
    fn bold_capsule(mut self, bold: bool) -> StarshipPrompt {
        if let Some(cap) = self.capsules.last_mut() {
            cap.bold = bold;
        }
        self
    }

    /// Override the character indicator and its color.
    pub fn character(mut self, symbol: impl Into<String>, color: Color) -> StarshipPrompt {
        self.character = symbol.into();
        self.character_color = color;
        self
    }

    pub fn capsules(&self) -> &[Capsule] {
        &self.capsules
    }

    /// Number of terminal columns the whole prompt occupies.
    pub fn width(&self) -> u16 {
        let caps: u16 = self.capsules.iter().map(Capsule::width).sum();
        caps + str_width(&self.character) as u16
    }

    /// Paint the prompt at `(x, y)`, returning the next column.
    pub fn render_at(&self, x: u16, y: u16, grid: &mut Grid) -> u16 {
        self.render_at_from(x, y, grid, 0)
    }

    /// Paint the prompt starting from capsule index `drop` at `(x, y)`,
    /// returning the next column. Skipping leading capsules lets the input
    /// line shrink gracefully on narrow terminals while keeping the
    /// character indicator visible.
    pub fn render_at_from(&self, x: u16, y: u16, grid: &mut Grid, drop: usize) -> u16 {
        let mut cx = x;
        for cap in self.capsules.iter().skip(drop) {
            cap.paint(cx, y, grid);
            cx += cap.width();
        }
        let style = Style::DEFAULT
            .fg(self.character_color)
            .mods(Modifiers::BOLD);
        grid.set_str(cx, y, &self.character, style);
        cx + str_width(&self.character) as u16
    }
}

impl Widget for StarshipPrompt {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        self.render_at(area.x, area.y, grid);
    }
}

/// A full prompt line: starship capsules + an editable [`Input`] + the
/// right-side `$time` capsule (`◂󰥔 HH:MM:SS`).
///
/// ```
/// use booms::widgets::{StarshipInput, StarshipPrompt};
/// let line = StarshipInput::new(StarshipPrompt::new().user("me"))
///     .placeholder("type here");
/// assert_eq!(line.value(), "");
/// ```
#[derive(Clone, Debug)]
pub struct StarshipInput {
    prompt: StarshipPrompt,
    input: Input,
    show_time: bool,
}

impl StarshipInput {
    pub fn new(prompt: StarshipPrompt) -> StarshipInput {
        StarshipInput {
            prompt,
            input: Input::new(),
            show_time: true,
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> StarshipInput {
        self.input = self.input.placeholder(text);
        self
    }

    /// Whether to show the right-side time capsule (default: true).
    pub fn show_time(mut self, show: bool) -> StarshipInput {
        self.show_time = show;
        self
    }

    /// The typed text.
    pub fn value(&self) -> String {
        self.input.value()
    }

    /// Clear the typed text, keeping the prompt and placeholder.
    pub fn clear(&mut self) {
        self.input.clear();
    }

    pub fn prompt(&self) -> &StarshipPrompt {
        &self.prompt
    }

    pub fn prompt_mut(&mut self) -> &mut StarshipPrompt {
        &mut self.prompt
    }

    /// Width of the right-side time capsule, in columns.
    fn time_width() -> u16 {
        5 + 8 // ◂ +  + icon + space + HH:MM:SS + 
    }

    /// How many leading capsules to drop and how many columns the prompt
    /// then uses, so at least `min_input` columns remain for typing on a
    /// narrow terminal. The character indicator is never dropped.
    fn fit(&self, total_width: u16, min_input: u16) -> (usize, u16) {
        let reserved = if self.show_time {
            StarshipInput::time_width() + 1
        } else {
            0
        };
        let char_w = str_width(&self.prompt.character) as u16;
        let mut used: u16 = self.prompt.capsules.iter().map(Capsule::width).sum::<u16>() + char_w;
        let mut drop = 0;
        while drop < self.prompt.capsules.len()
            && total_width.saturating_sub(used).saturating_sub(reserved) < min_input
        {
            used -= self.prompt.capsules[drop].width();
            drop += 1;
        }
        (drop, used)
    }

    /// Current UTC time in `HH:MM:SS` (the config uses local time; no
    /// timezone crate is pulled in).
    fn clock_text() -> String {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let h = (secs / 3600) % 24;
        let m = (secs / 60) % 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }

    fn paint_time(&self, area: Rect, grid: &mut Grid) {
        if !self.show_time {
            return;
        }
        let tw = StarshipInput::time_width();
        if area.width <= tw + 1 {
            return;
        }
        let mut x = area.right() - tw;
        let style = Style::DEFAULT.fg(noctalia::BRIGHT);
        grid.set(x, area.y, FLOW_ARROW_LEFT, style);
        x += 1;
        grid.set(x, area.y, BUBBLE_START, style);
        x += 1;
        grid.set(x, area.y, icon::CLOCK, style);
        x += 1;
        grid.set(x, area.y, ' ', style);
        x += 1;
        let text = StarshipInput::clock_text();
        let end = grid.set_str(x, area.y, &text, style);
        grid.set(end, area.y, BUBBLE_END, style);
    }
}

impl Widget for StarshipInput {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let (drop, used) = self.fit(area.width, 12);
        self.prompt.render_at_from(area.x, area.y, grid, drop);

        let reserved = if self.show_time {
            StarshipInput::time_width() + 1
        } else {
            0
        };
        let avail = area.width.saturating_sub(used).saturating_sub(reserved);
        if avail > 0 {
            let in_x = area.x.saturating_add(used).min(area.right());
            self.input.render(Rect::new(in_x, area.y, avail, 1), grid);
        }
        self.paint_time(area, grid);
    }

    fn is_focusable(&self) -> bool {
        self.input.is_focusable()
    }

    fn set_focus(&mut self, focused: bool) {
        self.input.set_focus(focused);
    }

    fn on_key(&mut self, key: KeyEvent) -> bool {
        // Control keys are normally consumed by the harness for view
        // switching; the Input widget already ignores them.
        self.input.on_key(key)
    }

    fn cursor_pos(&self, area: Rect) -> Option<(u16, u16)> {
        let (_drop, used) = self.fit(area.width, 12);
        let reserved = if self.show_time {
            StarshipInput::time_width() + 1
        } else {
            0
        };
        let avail = area.width.saturating_sub(used).saturating_sub(reserved);
        if avail == 0 {
            return None;
        }
        let in_x = area.x.saturating_add(used).min(area.right());
        self.input.cursor_pos(Rect::new(in_x, area.y, avail, 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Cell;

    fn grid(w: u16, h: u16) -> Grid {
        Grid::new(w, h, Cell::blank())
    }

    #[test]
    fn prompt_width_matches_painted_cells() {
        let p = StarshipPrompt::new()
            .user("clumzzy")
            .rust("v1.97")
            .directory("~/booms");
        let w = p.width();
        let mut g = grid(80, 1);
        let end = p.render_at(0, 0, &mut g);
        assert_eq!(end, w);
        assert_eq!(g.get(0, 0).ch, BUBBLE_START);
        assert_eq!(g.get(1, 0).ch, icon::USER);
    }

    #[test]
    fn battery_colors_follow_thresholds() {
        let p = StarshipPrompt::new().battery(95);
        assert_eq!(p.capsules()[0].color(), noctalia::GREEN);
        let p = StarshipPrompt::new().battery(10);
        assert_eq!(p.capsules()[0].color(), noctalia::RED);
        assert!(p.capsules()[0].bold);
    }

    #[test]
    fn capsule_width_counts_text_and_arcs() {
        let c = Capsule::new('x', "abc", noctalia::BLUE);
        assert_eq!(c.width(), 8); // 5 + 3
        assert_eq!(c.text(), "abc");
    }
}
