//! AI Agent Harness demo: a beautiful, aesthetic chat-style agent interface
//! using booms.
//!
//! Demonstrates:
//! - Glass panels with crisp transparency (semi-transparent borders + content
//!   blending over the canvas background)
//! - Orb-like interface: thinking GlassOrb, status NeonOrb, PulseOrb status
//!   bar
//! - Glow-style markdown rendering of messages
//! - Streaming agent responses (character-by-character reveal)
//! - Scrollable message history (word-wrapped, PageUp/PageDown, Scrollbar)
//! - Multi-view navigation (chat → settings → vibe → about), each view in its
//!   own glass panel
//! - Starship-style prompt input (noctalia capsule theme, right-side clock)
//! - Mouse support (click to focus/select, scroll to move) and bracketed paste
//! - Noctalia (Catppuccin Mocha) color theme throughout
//!
//! Run with: `cargo run --example agent_harness`

use std::time::{Duration, Instant};

use booms::app::App;
use booms::asciify::AsciifyEffect;
use booms::grid::Grid;
use booms::layout::{Constraint, Rect};
use booms::markdown::{MdLine, MdSpan, render_markdown};
use booms::style::{Color, Modifiers, Style};
use booms::widgets::{
    Asciify, BorderType, GlassOrb, NeonOrb, Panel, Scrollbar, StarshipInput, StarshipPrompt,
    TitlePosition, Widget,
};
use booms::{KeyCode, KeyModifiers};

// ── Noctalia palette (Catppuccin Mocha) ───────────────────────────────────

const BLUE: Color = Color::Rgb(137, 180, 250);
const GREEN: Color = Color::Rgb(166, 227, 161);
const YELLOW: Color = Color::Rgb(249, 226, 175);
const CYAN: Color = Color::Rgb(148, 226, 213);
const MAGENTA: Color = Color::Rgb(245, 194, 231);
const PEACH: Color = Color::Rgb(250, 179, 135);
const SKY: Color = Color::Rgb(137, 220, 235);
const TEXT: Color = Color::Rgb(205, 214, 244);
const OVERLAY0: Color = Color::Rgb(108, 112, 134);
const BASE: Color = Color::Rgb(30, 30, 46); // mocha base #1e1e2e

/// A single chat message with role and text.
#[derive(Clone, Debug)]
struct ChatMessage {
    role: String,
    text: String,
}

impl ChatMessage {
    fn user(text: &str) -> ChatMessage {
        ChatMessage {
            role: "you".to_string(),
            text: text.to_string(),
        }
    }

    fn agent(text: &str) -> ChatMessage {
        ChatMessage {
            role: "agent".to_string(),
            text: text.to_string(),
        }
    }
}

/// The main chat view: message history + input + status.
struct ChatView {
    messages: Vec<ChatMessage>,
    thinking: GlassOrb,
    /// The asciify-engine animation shown while the agent thinks.
    think_anim: Asciify,
    agent_status: String,
    streaming_text: String,
    streaming_full: String,
    streaming_pos: usize,
    streaming_active: bool,
    stream_timer: Instant,
    /// First visible display line (in wrapped lines).
    scroll: usize,
    /// Whether the viewport follows the newest messages.
    at_bottom: bool,
}

impl ChatView {
    fn new() -> ChatView {
        ChatView {
            messages: vec![
                ChatMessage::user("hello, can you help me with a task?"),
                ChatMessage::agent("Hello! I'd be happy to help. What do you need?"),
                ChatMessage::user("write a function to parse JSON"),
                ChatMessage::agent("I can help with that. Here's a simple approach:"),
            ],
            thinking: GlassOrb::new().color(CYAN),
            think_anim: Asciify::new(AsciifyEffect::Wave),
            agent_status: "ready".to_string(),
            streaming_text: String::new(),
            streaming_full: String::new(),
            streaming_pos: 0,
            streaming_active: false,
            stream_timer: Instant::now(),
            scroll: 0,
            at_bottom: true,
        }
    }

    fn clear(&mut self) {
        self.messages.clear();
        self.agent_status = "ready".to_string();
        self.streaming_active = false;
        self.streaming_text.clear();
        self.streaming_full.clear();
        self.scroll = 0;
        self.at_bottom = true;
    }

    fn add_user_message(&mut self, text: &str) {
        self.messages.push(ChatMessage::user(text));
        self.agent_status = "thinking".to_string();
        self.start_agent_response();
    }

    fn start_agent_response(&mut self) {
        // A markdown-rich reply, streamed character-by-character and
        // rendered glow-style (headings, lists, task lists, code blocks,
        // blockquotes, links, bold/italic/strike/inline code).
        self.streaming_full = "# Plan\n\nHere's what I'll do:\n\n1. **Analyze** the request\n2. Write a `parse` function\n3. Add *tests*\n\n- [x] analyze request\n- [ ] write code\n\n```rust\nfn parse(s: &str) -> Result<Value> { ... }\n```\n\n> Note: errors are handled.\n\nSee [docs](https://example.com) for details.".to_string();
        self.streaming_text.clear();
        self.streaming_pos = 0;
        self.streaming_active = true;
        self.stream_timer = Instant::now();
    }

    fn tick_streaming(&mut self) {
        if !self.streaming_active {
            return;
        }
        if self.stream_timer.elapsed() >= Duration::from_millis(30) {
            if self.streaming_pos < self.streaming_full.len() {
                let ch = self.streaming_full[self.streaming_pos..]
                    .chars()
                    .next()
                    .unwrap_or(' ');
                self.streaming_text.push(ch);
                self.streaming_pos += ch.len_utf8();
                self.stream_timer = Instant::now();
            } else {
                self.streaming_active = false;
                self.messages.push(ChatMessage::agent(&self.streaming_full));
                self.streaming_text.clear();
                self.streaming_full.clear();
                self.agent_status = "done".to_string();
            }
        }
    }

    /// Words per page when scrolling with PageUp/PageDown.
    const PAGE_SCROLL: usize = 5;

    /// Wrap every message (and the in-progress streaming line) into
    /// display lines of at most `width` columns, rendering each message
    /// as glow-style markdown. The `[role] ` prefix is drawn on the first
    /// line of each message; continuation lines are indented to align
    /// with the text.
    fn display_lines(&self, width: usize) -> Vec<MdLine> {
        let width = width.max(8);
        let mut lines: Vec<MdLine> = Vec::new();

        let push_message = |lines: &mut Vec<MdLine>, role: &str, text: &str, body: Style| {
            let prefix = format!("[{}] ", role);
            let prefix_w = booms::grid::str_width(&prefix);
            let indent = " ".repeat(prefix_w);
            let text_w = width.saturating_sub(prefix_w).max(1);
            let md = render_markdown(text, text_w, role_color(role));
            for (i, md_line) in md.iter().enumerate() {
                let head = if i == 0 {
                    prefix.clone()
                } else {
                    indent.clone()
                };
                let head_style = Style::DEFAULT.fg(role_color(role)).mods(Modifiers::BOLD);
                let mut spans = vec![MdSpan {
                    text: head,
                    style: head_style,
                }];
                // The markdown body is re-tinted to `body` so the role
                // coloring (streaming magenta, finished white) wins over
                // the base white while keeping inline styles distinct.
                for span in &md_line.spans {
                    spans.push(MdSpan {
                        text: span.text.clone(),
                        style: retint(span.style, body),
                    });
                }
                lines.push(MdLine { spans });
            }
        };

        for msg in &self.messages {
            let body = Style::DEFAULT.fg(TEXT);
            push_message(&mut lines, &msg.role, &msg.text, body);
        }
        if self.streaming_active && !self.streaming_text.is_empty() {
            let body = Style::DEFAULT.fg(MAGENTA).mods(Modifiers::BOLD);
            push_message(
                &mut lines,
                "agent",
                &format!("{}▌", self.streaming_text),
                body,
            );
        }
        lines
    }

    /// The visible window for `total` lines in `visible` rows: the first
    /// line to draw. Follows the bottom when `at_bottom`, otherwise uses
    /// the scroll offset (clamped).
    fn scroll_start(&self, total: usize, visible: usize) -> usize {
        let max_offset = total.saturating_sub(visible);
        if self.at_bottom {
            max_offset
        } else {
            self.scroll.min(max_offset)
        }
    }

    fn scroll_up(&mut self, n: usize) {
        self.at_bottom = false;
        self.scroll = self.scroll.saturating_sub(n);
    }

    fn scroll_down(&mut self, _n: usize) {
        // Scrolling down to the newest message re-enables following.
        self.at_bottom = true;
    }
}

/// The list-marker color per role.
fn role_color(role: &str) -> Color {
    if role == "you" { BLUE } else { YELLOW }
}

/// Re-tint plain body text (white) to `body`, keeping markdown's own
/// styling (heading colors, code backgrounds, link underlines) intact.
fn retint(style: Style, body: Style) -> Style {
    if style.fg == Color::Ansi(15) {
        Style {
            fg: body.fg,
            bg: style.bg,
            mods: style.mods.union(body.mods),
            opacity: style.opacity,
        }
    } else {
        style
    }
}

impl Widget for ChatView {
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    fn on_tick(&mut self) {
        self.thinking.on_tick();
        self.think_anim.on_tick();
        self.tick_streaming();
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match key.code {
            KeyCode::PageUp => {
                self.scroll_up(Self::PAGE_SCROLL);
                true
            }
            KeyCode::PageDown => {
                self.scroll_down(Self::PAGE_SCROLL);
                true
            }
            _ => false,
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let (w, h) = (area.width, area.height);
        if w == 0 || h == 0 {
            return;
        }

        // Header: agent status + thinking orb
        let header_style = Style::DEFAULT.fg(CYAN).mods(Modifiers::BOLD);
        let header = format!("agent status: {}", self.agent_status);
        grid.set_str(area.x, area.y, &header, header_style);

        // Thinking glass orb next to status
        if self.agent_status == "thinking" || self.agent_status == "working" {
            let orb_x = area.x + header.len() as u16 + 2;
            if orb_x < area.right() {
                let orb_area = Rect::new(orb_x, area.y, 1, 1);
                self.thinking.render(orb_area, grid);
            }
        }

        // While the agent thinks, an asciify-engine animation runs under
        // the header: the engine baked into the agent flow.
        let anim_h = if self.agent_status == "thinking" {
            3u16.min(h.saturating_sub(5))
        } else {
            0
        };
        if anim_h > 0 {
            self.think_anim
                .render(Rect::new(area.x, area.y + 2, w, anim_h), grid);
        }

        // Message area: reserve the right edge for the scrollbar.
        let msg_area_y = area.y + 2 + anim_h;
        let msg_area_h = h.saturating_sub(4 + anim_h);
        if msg_area_h == 0 {
            return;
        }
        let bar_x = area.right().saturating_sub(1);
        let content_right = bar_x; // message text stops before the bar
        let msg_w = content_right.saturating_sub(area.x) as usize;

        // Render messages as markdown and apply the scroll window.
        let lines = self.display_lines(msg_w);
        let total = lines.len();
        let visible = msg_area_h as usize;
        let start = self.scroll_start(total, visible);

        for (i, line) in lines.iter().skip(start).take(visible).enumerate() {
            let y = msg_area_y + i as u16;
            if y >= area.bottom() {
                break;
            }
            let mut end = area.x;
            for span in &line.spans {
                end = grid.set_str(end, y, &span.text, span.style);
            }
            for x in end..content_right {
                grid.set(x, y, ' ', Style::DEFAULT);
            }
        }

        // Scrollbar along the right edge of the message area.
        Scrollbar::new(total, visible, start)
            .render(Rect::new(bar_x, msg_area_y, 1, msg_area_h), grid);
    }

    fn is_focusable(&self) -> bool {
        false
    }
}

/// Settings view: rendered inside a glass panel.
struct SettingsView;

impl Widget for SettingsView {
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let title_style = Style::DEFAULT.fg(CYAN).mods(Modifiers::BOLD);
        grid.set_str(area.x, area.y, "⚙  settings", title_style);

        let items = [
            ("model: gpt-4", BLUE),
            ("temperature: 0.7", GREEN),
            ("max_tokens: 2048", YELLOW),
            ("stream: enabled", GREEN),
            ("system_prompt: helpful assistant", PEACH),
        ];
        for (i, (item, color)) in items.iter().enumerate() {
            let y = area.y + 2 + i as u16;
            if y >= area.bottom() {
                break;
            }
            grid.set_str(area.x, y, item, Style::DEFAULT.fg(*color));
        }
    }
}

/// About view: rendered inside a glass panel.
struct AboutView;

impl Widget for AboutView {
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let title_style = Style::DEFAULT.fg(CYAN).mods(Modifiers::BOLD);
        grid.set_str(area.x, area.y, "ℹ  about", title_style);

        let lines = [
            ("booms AI agent harness", TEXT),
            ("", TEXT),
            ("A TUI framework for building AI agent interfaces", OVERLAY0),
            (
                "with animated orbs, streaming output, and view transitions.",
                OVERLAY0,
            ),
            ("", TEXT),
            ("Framework: booms v0.1.0", SKY),
            ("Backend: crossterm", SKY),
            ("Theme: noctalia (Catppuccin Mocha)", PEACH),
        ];
        for (i, (line, color)) in lines.iter().enumerate() {
            let y = area.y + 2 + i as u16;
            if y >= area.bottom() {
                break;
            }
            grid.set_str(area.x, y, line, Style::DEFAULT.fg(*color));
        }
    }
}

/// Vibe view: the asciify-engine animation baked in full-screen.
struct VibeView {
    scene: Asciify,
}

impl VibeView {
    fn new() -> VibeView {
        VibeView {
            scene: Asciify::new(AsciifyEffect::Wave),
        }
    }

    fn cycle_scene(&mut self) {
        self.scene.cycle_effect();
    }
}

impl Widget for VibeView {
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    fn on_tick(&mut self) {
        self.scene.on_tick();
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let label_h = 1u16;
        let scene_h = area.height.saturating_sub(label_h);
        if scene_h > 0 {
            self.scene
                .render(Rect::new(area.x, area.y, area.width, scene_h), grid);
        }
        let footer = format!(
            " asciify-engine: {} (ctrl+v cycles)",
            self.scene.effect().name()
        );
        let y = area.y + area.height.saturating_sub(1);
        grid.set_str(area.x, y, &footer, Style::DEFAULT.fg(OVERLAY0));
    }
}

/// Status bar at the bottom with a pulsing neon orb.
struct StatusBar {
    orb: NeonOrb,
    clock_ticks: u64,
    status_text: String,
}

impl StatusBar {
    fn new() -> StatusBar {
        StatusBar {
            orb: NeonOrb::new().color(GREEN),
            clock_ticks: 0,
            status_text: "ready".to_string(),
        }
    }
}

impl Widget for StatusBar {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn on_tick(&mut self) {
        self.orb.on_tick();
        self.clock_ticks += 1;
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let style = Style::DEFAULT.fg(SKY);
        let left = format!(" ticks: {}", self.clock_ticks);
        grid.set_str(area.x, area.y, &left, style);

        // Status orb in the middle
        let orb_x = area.x + area.width / 2;
        let orb_area = Rect::new(orb_x, area.y, 1, 1);
        self.orb.render(orb_area, grid);

        // Status text on the right
        let status = format!(" {} ", self.status_text);
        let right_x = area.right().saturating_sub(status.len() as u16);
        if right_x > area.x {
            grid.set_str(right_x, area.y, &status, style);
        }
    }
}

/// The agent harness: manages views and the input field.
struct AgentHarness {
    input: StarshipInput,
    chat_view: ChatView,
    settings_view: SettingsView,
    vibe_view: VibeView,
    about_view: AboutView,
    status_bar: StatusBar,
    current_view: usize, // 0=chat, 1=settings, 2=vibe, 3=about
}

impl AgentHarness {
    fn new() -> AgentHarness {
        AgentHarness {
            input: StarshipInput::new(
                StarshipPrompt::new()
                    .user("clumzzy")
                    .directory("~")
                    .rust("v1.97")
                    .battery(95)
                    .model("gpt-4"),
            )
            .placeholder("type your message..."),
            chat_view: ChatView::new(),
            settings_view: SettingsView,
            vibe_view: VibeView::new(),
            about_view: AboutView,
            status_bar: StatusBar::new(),
            current_view: 0,
        }
    }

    fn handle_submit(&mut self) {
        let text = self.input.value();
        if !text.trim().is_empty() {
            self.chat_view.add_user_message(&text);
            self.input.clear();
        }
    }

    fn switch_view(&mut self, idx: usize) {
        self.current_view = idx;
    }
}

impl Widget for AgentHarness {
    fn constraint(&self) -> Constraint {
        Constraint::Flex(1)
    }

    fn on_tick(&mut self) {
        if self.current_view == 0 {
            self.chat_view.on_tick();
        }
        if self.current_view == 2 {
            self.vibe_view.on_tick();
        }
        self.status_bar.on_tick();
    }

    /// Render each view inside a semi-transparent glass panel with the
    /// noctalia border color. The panel blends over the canvas background,
    /// giving crisp transparency; the view content is drawn inside the
    /// panel's inner area.
    fn render(&self, area: Rect, grid: &mut Grid) {
        // Header bar
        let header_style = Style::DEFAULT.fg(CYAN).mods(Modifiers::BOLD);
        let nav_style = Style::DEFAULT.fg(OVERLAY0);
        let active_style = Style::DEFAULT.fg(CYAN).mods(Modifiers::BOLD);

        let header = " booms ai agent harness  │  ".to_string();
        grid.set_str(area.x, area.y, &header, header_style);

        // Navigation tabs
        let tabs = ["chat", "settings", "vibe", "about"];
        let mut x = area.x + header.len() as u16;
        for (i, tab) in tabs.iter().enumerate() {
            let style = if i == self.current_view {
                active_style
            } else {
                nav_style
            };
            let label = format!(" {} ", tab);
            let end = grid.set_str(x, area.y, &label, style);
            if i < tabs.len() - 1 {
                grid.set_str(end, area.y, "│", nav_style);
                x = end + 1;
            } else {
                x = end;
            }
        }

        // Main content area, wrapped in a glass panel.
        let content_y = area.y + 2;
        let content_h = area.height.saturating_sub(4); // header + input + status
        let content_area = Rect::new(area.x, content_y, area.width, content_h);

        let view_name = match self.current_view {
            0 => "chat",
            1 => "settings",
            2 => "vibe",
            _ => "about",
        };

        // Border color per view.
        let border_color = match self.current_view {
            0 => CYAN,
            1 => BLUE,
            2 => MAGENTA,
            _ => PEACH,
        };

        // Draw the glass panel border + title, then the view inside.
        let panel = Panel::new()
            .title(view_name)
            .borders(BorderType::Rounded)
            .border_style(Style::DEFAULT.fg(border_color).opacity(200))
            .content_style(Style::DEFAULT.fg(TEXT).opacity(220))
            .title_style(Style::DEFAULT.fg(border_color).mods(Modifiers::BOLD))
            .opacity(180)
            .title_position(TitlePosition::Top)
            .padding(1);
        panel.render(content_area, grid);
        let inner = panel.inner(content_area);

        match self.current_view {
            0 => self.chat_view.render(inner, grid),
            1 => self.settings_view.render(inner, grid),
            2 => self.vibe_view.render(inner, grid),
            _ => self.about_view.render(inner, grid),
        }

        // Input area: starship-style noctalia prompt.
        let input_y = area.y + area.height.saturating_sub(2);
        let input_area = Rect::new(area.x, input_y, area.width, 1);
        self.input.render(input_area, grid);

        // Status bar: neon orb.
        let status_y = area.y + area.height.saturating_sub(1);
        let status_area = Rect::new(area.x, status_y, area.width, 1);
        self.status_bar.render(status_area, grid);
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.input.set_focus(focused);
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => {
                self.handle_submit();
                true
            }
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.current_view != 0 {
                    self.switch_view(0);
                }
                self.chat_view.clear(); // New chat
                true
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.switch_view(1); // Settings
                true
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.current_view == 2 {
                    self.vibe_view.cycle_scene(); // Cycle asciify effect
                } else {
                    self.switch_view(2); // Vibe
                }
                true
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.switch_view(2); // About
                true
            }
            KeyCode::PageUp | KeyCode::PageDown => self.chat_view.on_key(key),
            _ => self.input.on_key(key),
        }
    }

    fn cursor_pos(&self, area: Rect) -> Option<(u16, u16)> {
        let input_y = area.y + area.height.saturating_sub(2);
        let input_area = Rect::new(area.x, input_y, area.width, 1);
        self.input.cursor_pos(input_area)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;
    // Noctalia canvas: mocha base so transparency blends against a dark
    // theme. Change these to match your terminal's background.
    app.set_canvas_colors(TEXT, BASE);
    app.add(AgentHarness::new());
    app.run()?;
    Ok(())
}
