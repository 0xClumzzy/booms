# booms

[![CI](https://github.com/0xClumzzy/booms/workflows/CI/badge.svg)](https://github.com/0xClumzzy/booms/actions)
[![Crates.io](https://img.shields.io/crates/v/booms.svg)](https://crates.io/crates/booms)
[![Documentation](https://docs.rs/booms/badge.svg)](https://docs.rs/booms)
[![License](https://img.shields.io/crates/l/booms.svg)](https://github.com/0xClumzzy/booms/blob/main/LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.85+-blue.svg)](https://github.com/0xClumzzy/booms#msrv)

A small, safe TUI framework in Rust: a cell grid renderer, a layout engine,
widgets, orb animations, mouse + paste input, a starship-style prompt, and an
event loop -- plus eased and spring-physics transition primitives, ratatui-style
data widgets, optional headless wgpu rendering, and a full AI agent harness.

## For Developers

- **Documentation**: [docs.rs/booms](https://docs.rs/booms)
- **Repository**: [github.com/0xClumzzy/booms](https://github.com/0xClumzzy/booms)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Code of Conduct**: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- **MSRV**: Rust 1.85 (pinned; updated deliberately with minor versions)

### Quick Links

| Task | Command |
|------|---------|
| Run core demo | `cargo run --example demo` |
| Run orb showcase | `cargo run --example orbs` |
| Run transitions | `cargo run --example transitions` |
| Run rich gallery | `cargo run --example rich` |
| Run aesthetic showcase | `cargo run --example aesthetic` |
| Run AI agent harness | `cargo run --example agent_harness` |
| Run with ratatui widgets | `cargo run --features ratatui --example native_ratatui` |
| Run GPU plasma demo | `cargo run --features gpu --example gpu` |
| Full test suite | `cargo test --all-targets --all-features` |
| Lint + format check | `cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --all -- --check` |
| Build docs locally | `cargo doc --all-features --no-deps --open` |

## Widgets

- `Text`: lines wrapped to the widget width.
- `Block`: a bordered box with an optional title.
- `Input`: a single-line text field.
- `List`: a vertical list with a moving cursor.
- `Scrollbar`: a vertical scrollbar with a proportional thumb, showing the
  viewport position within longer content.
- `MarkdownText`: glow-style markdown rendering (headings, lists, task
  lists, code blocks, blockquotes, links, inline styles), wrapped to the
  widget width.
- `Asciify`: an animated ASCII scene (wave / fog / orbits / pulses) filling
  its area, dithered with an ordered Bayer pattern.
- `Panel`: a bordered box (plain, rounded `╭╮╰╯`, double, or thick) whose
  title wraps around the border perimeter and continues past corners; content
  is word-wrapped. See [Panels](#panels).
- `Gauge`, `LineGauge`, `Sparkline`, `BarChart`, `Tabs`, `Table`:
  ratatui-inspired data widgets. See [Ratatui-inspired widgets](#ratatui-inspired-widgets).
- `StarshipPrompt`, `StarshipInput`: a starship-style prompt with noctalia
  capsules and an editable input line. See [Starship prompt](#starship-prompt).
- `Panel`: a bordered box (plain, rounded `╭╮╰╯`, double, or thick) whose
  title wraps around the border perimeter and continues past corners; content
  is word-wrapped; supports opacity for crisp transparency. See [Panels](#panels).
- `Gauge`, `LineGauge`, `Sparkline`, `BarChart`, `Tabs`, `Table`:
  ratatui-inspired data widgets. See [Ratatui-inspired widgets](#ratatui-inspired-widgets).
- `RatatuiView` (feature `ratatui`): render real `ratatui` widgets
  (Paragraph, Block, Gauge, Table, List, Tabs, Sparkline, BarChart, …) inside
  booms. See [Native ratatui support](#native-ratatui-support).

## Orb animations

- `PulseOrb`: nano-style tiny pulsing status indicator (breathes dim →
  bright → dim on a sine curve).
- `CrystalOrb`: a faceted crystal gem with a sweeping shine and shifting
  facet colors.
- `RingOrb`: concentric rotating rings of geometric characters around a
  pulsing core.
- `LiquidGooeyOrb`: an organic blob whose edge ripples with sine waves and
  settles with spring-physics wobble (ported from liquid-gooey).
- `WaveOrb`: distorted rings whose radius varies with interfering sine
  waves, colors pulsing along the wave phase.
- `SpinnerOrb`: a braille spinner (`⠋⠙⠹…`) for compact "working" states.
- `GlassOrb`: a frosted-glass orb: concentric rings of decreasing opacity that
  breathe in and out; ideal for "thinking" states on transparent panels.
- `NeonOrb`: a neon sign orb: fading rings around a bright flickering core,
  glowing on dark themes.
- `AuraOrb`: a layered glow (` ·░▒▓█`) whose radius breathes on a sine pulse.
- `CometOrb`: an orbiting particle with a fading trail.
- `HeartbeatOrb`: an ECG-style pulse line that spikes on each tick.
- `ThinkingOrb`: the Libraries.dev port: nine hand-tuned animated states for
  AI thinking indicators, each a distinct dot-field animation: `Working`
  (particles on tilted orbits), `Searching` (a scan meridian sweeps a dotted
  globe), `Solving` (bands scramble then click back), `Listening` (a waveform
  rolls through rings), `Connecting` (a constellation wires itself),
  `Weaving` (three strands plait around the sphere), `Composing` (an
  undulating multi-band sash), `Breathing` (a morphing ring), `Shaping`
  (circle → triangle → square). Two tuned sizes: `OrbSize::Size64`
  (chat-avatar scale) and `OrbSize::Size20` (inline-text scale). Drive with
  `speed`, `dark`, `paused`.

  ```rust
  ThinkingOrb::new()
      .state(OrbState::Searching)
      .size(OrbSize::Size64)
      .speed(1.5)
      .dark(true);
  ```

## Animations and transitions

The [`anim`](src/anim.rs) module interpolates values over time with eased
curves: [`Easing`](src/anim.rs) carries thinking-orbs' smoothstep curve;
[`Spring`](src/anim.rs) is a spring-physics transition (ported from the
liquid-gooey spring engine). Advance it from a widget's `on_tick` and read
[`Anim::value`](src/anim.rs):

```rust
use std::time::Duration;
use booms::anim::{Anim, Easing};
use booms::style::Color;

let mut fade = Anim::new(Color::Rgb(30, 30, 30), Color::Rgb(210, 210, 210), Duration::from_millis(700))
    .easing(Easing::SineInOut);
fade.update();
let color: Color = *fade.value();
```

The [`transitions`](src/transitions.rs) module provides animated
transitions between full-screen views: `Fade` (speckled dissolve),
`Slide` (directional reveal), and `Cube` (vertical strip wipe), all driven
by eased progress. Use [`App::push_view`](src/app.rs) and
[`App::pop_view`](src/app.rs) to switch views with a transition:

```rust
use booms::app::App;
use booms::Transition;
use booms::transitions::SlideDirection;

let mut app = App::new()?;
app.add(Text::new(["view one".to_string()]));
app.push_view(
    Transition::fade(std::time::Duration::from_millis(700)),
    vec![Box::new(Text::new(["view two".to_string()]))],
);
app.run()?;
```

## Markdown rendering

The [`markdown`](src/markdown.rs) module renders GitHub-flavored markdown
into styled lines: a Rust port of the glow/glamour approach (glow is a CLI
for glamour, the markdown renderer). `render_markdown(text, width,
role_color)` returns `MdLine`s of styled `MdSpan`s; paint them yourself or
use the `MarkdownText` widget:

```rust
use booms::markdown::render_markdown;
use booms::style::Color;

let lines = render_markdown("# Title\n\n- **bold** item", 40, Color::Ansi(14));
```

Supported: headings (`#`…`######`, colored + underline), horizontal rules,
blockquotes (left margin, italic), ordered/unordered lists, task lists
(`- [ ]` / `- [x]` with `✓`/`✗`), inline `**bold**`, `*italic*`,
`~~strike~~`, `` `code` `` (dim, background tint), `[links](url)`
(underlined + the URL shown), and fenced code blocks with a language label
and background tint. Markdown keeps its own colors; `role_color` only
tints list markers, so a message can be tinted per speaker.

## Asciify animations

The [`asciify`](src/asciify.rs) module ports the
[asciify-engine](https://github.com/ayangabryl/asciify-engine) idea: density
ramps (` .:-=+*#%@`), hue-swept color, and ordered Bayer dithering for
smooth animation. Each `AsciifyEffect` (`Wave`, `Fog`, `Orbits`,
`Pulses`) is a pure function of elapsed time `t`: the same `t` always
produces the same frame:

```rust
use booms::asciify::{AsciifyEffect, FILL};
use booms::grid::{Cell, Grid};

let mut grid = Grid::new(48, 14, Cell::blank());
AsciifyEffect::Wave.tick(1.5, &mut grid);
```

The `Asciify` widget wraps an effect for the layout engine and advances it
from `on_tick`; the agent harness shows one while the agent thinks and
offers a full-screen vibe view (Ctrl+V cycles effects).

## Panels

`Panel` draws a bordered box around content. Four border styles are available
via `BorderType`: `Plain` (`┌┐└┘`), `Rounded` (`╭╮╰╯`), `Double` (`╔╗╚╝`), and
`Thick` (`┏┓┗┛`). A title can sit on any edge (`TitlePosition`) with any
alignment (`TitleAlignment`). If the title is longer than its edge, it keeps
going around the perimeter -- across corners -- instead of being clipped, so a
single label can wrap all the way around the box. Content is word-wrapped to
the inner width and indented by the border plus optional padding. Every panel
supports `opacity(0..=255)` (`Panel::opacity`): below `255` the border, title,
and content blend over the canvas background, producing crisp transparency:
a frosted-glass look on dark terminal themes.

```rust
use booms::widgets::{BorderType, Panel, Widget};
use booms::layout::Rect;
use booms::grid::{Cell, Grid};
use booms::style::Color;

let panel = Panel::new()
    .borders(BorderType::Rounded)
    .title("a long title that wraps around the corners")
    .content(["some wrapped content for the panel"])
    .opacity(180); // semi-transparent glass panel
let mut grid = Grid::new(24, 6, Cell::blank());
panel.render(Rect::new(0, 0, 24, 6), &mut grid);
```

## Crisp transparency and glass panels

Semi-transparent rendering is built into booms at the style level. [`Style::opacity`](src/style.rs)
takes `0..=255` (`255` = opaque, `0` = fully transparent). Cells with
opacity below `255` are alpha-blended over the grid's canvas colors before
being stored, so the diff renderer only ever writes solid ANSI/RGB cells; no
terminal alpha support is required.

Set the canvas colors (what transparency blends toward) with
[`App::set_canvas_colors`](src/app.rs), e.g. to match your terminal theme:

```rust
use booms::app::App;
use booms::style::Color;

let mut app = App::new()?;
// Catppuccin Mocha base: blend transparent cells over mocha base (#1e1e2e)
app.set_canvas_colors(Color::Rgb(205, 214, 244), Color::Rgb(30, 30, 46));
```

The default canvas is black on both sides. Any widget -- panel border, orb,
text -- can carry its own opacity, so a `Panel` at `opacity(160)` over a neon
`NeonOrb` reads as glass over glow. Combined with rounded borders and
wrap-around titles, this is booms' answer to crisp transparent TUI interfaces.

## Ratatui-inspired widgets

A small set of familiar data widgets, kept in booms' reference-based `Widget`
style: `Gauge` (a labeled `████░░░░` progress bar), `LineGauge` (a thin one-line
track), `Sparkline` (`▁▂▃▄▅▆▇█` bars), `BarChart` (labeled horizontal bars),
`Tabs` (a selectable tab bar), and `Table` (a header plus rows with automatic
column widths). Unlike ratatui's consuming `Widget::render(self, …)`, these
borrow `&self` so they can be held in an `App` and re-rendered every tick.

## Native ratatui support (feature `ratatui`)

The `ratatui` Cargo feature adds
[`RatatuiView`](src/widgets/ratatui_native.rs), an adapter that wraps any
cloneable [`ratatui::widgets::Widget`](https://docs.rs/ratatui/latest/ratatui/widgets/trait.Widget.html)
and paints it into booms' grid each frame. Ratatui colors map to booms' ANSI
and truecolor colors, modifiers (bold, italic, underline, blink, reverse,
strikethrough) carry across, and wide symbols (CJK, emoji) advance by their
display width. The wrapped widget is cloned and re-rendered every tick, so it
can show live data, clocks, or animations.

```rust
use booms::app::App;
use booms::widgets::RatatuiView;
use ratatui::widgets::{Block, Paragraph};

let mut app = App::new()?;
app.add(RatatuiView::new(
    Paragraph::new("real ratatui inside booms")
        .block(Block::bordered()),
));
app.run()?;
```

Because ratatui is a large dependency tree, it is opt-in:

```sh
cargo run --features ratatui --example native_ratatui
```

The [`native_ratatui`](examples/native_ratatui.rs) example shows a Paragraph,
Gauge, Tabs, and List -- all genuine ratatui widgets -- rendered inside booms.

## Mouse and rich CLI input

The terminal is put into alternate screen with mouse capture, bracketed paste,
and focus-change events enabled. `App` dispatches `Event::Mouse` to the widget
under the cursor (0-based coordinates) and focuses it on a left click; wheel
scrolls are forwarded too. Widgets implement `on_mouse(&mut self, event, area)`:
`List`, `Input`, and `Scrollbar` already do (click to select / place the
cursor / jump the thumb). Bracketed `Event::Paste` text is forwarded to the
focused widget as key events, so pasting a message into the agent harness just
works. `FocusGained` / `FocusLost` are also surfaced.

## Starship prompt

`StarshipPrompt` reproduces the local starship setup: a Catppuccin-Mocha
("noctalia") palette, powerline-style capsules (`<icon> <text> ▸`), and user,
host, directory, rust, battery (with threshold colors), os, and model segments.
`StarshipInput` pairs that prompt with an editable `Input` and a right-aligned
`◂󰥔 HH:MM:SS` time capsule. On narrow terminals it drops leading capsules
(never the `❯` indicator) so there is always room to type. It is the input line
used by the agent harness.

## GPU rendering (optional)

The `gpu` Cargo feature adds [`GpuSurface`](src/gpu.rs), a widget that renders
an animated WGSL plasma shader with wgpu entirely offscreen and reads the
pixels back into terminal cells using the truecolor half-block trick (`▀`, top
pixel as foreground, bottom as background). It initializes a headless adapter
with `pollster`, so no window is required; if no adapter is available it falls
back to a text message instead of panicking. The dependency is optional so the
core crate stays lightweight.

```sh
cargo run --features gpu --example gpu
```

See [`examples/demo.rs`](examples/demo.rs),
[`examples/orbs.rs`](examples/orbs.rs),
[`examples/transitions.rs`](examples/transitions.rs),
[`examples/agent_harness.rs`](examples/agent_harness.rs),
[`examples/aesthetic.rs`](examples/aesthetic.rs),
[`examples/rich.rs`](examples/rich.rs), and
[`examples/gpu.rs`](examples/gpu.rs): fade-in banners,
traveling-beam blocks, bouncing orbs, crystal gems, rotating rings,
liquid gooey blobs, wave rings, ticking clocks, selectable lists, text
inputs, view transitions, a full AI agent harness with streaming output, a
rich panel/orb/data-widget gallery ([`examples/rich.rs`](examples/rich.rs)),
an aesthetic glass-panel/orb showcase ([`examples/aesthetic.rs`](examples/aesthetic.rs)),
and an offscreen wgpu plasma demo ([`examples/gpu.rs`](examples/gpu.rs)).

## Quick start

```rust
use booms::app::App;
use booms::widgets::{CrystalOrb, Text};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;
    app.add(Text::new(["hello".to_string()]));
    app.add(CrystalOrb::new());
    app.run()?;
    Ok(())
}
```

Run the demos:

```sh
cargo run --example demo
cargo run --example orbs
cargo run --example transitions
cargo run --example agent_harness
cargo run --example aesthetic
cargo run --example rich
cargo run --features ratatui --example native_ratatui
cargo run --features gpu --example gpu
```

Keys: arrow keys move the list, Tab/Shift+Tab move focus, Escape or Ctrl+C quits. In the agent harness: Enter sends a message, PageUp/PageDown scroll the message history, Ctrl+N starts a new chat, Ctrl+S opens settings, Ctrl+V opens the asciify vibe view (Ctrl+V again cycles the effect), Ctrl+A opens about.

## Design notes

- Animations run at the app's tick rate (default 50 ms) from `on_tick`: no extra threads.
- The grid is diffed each frame; only changed cells are written to the terminal.
- Views are stacked; `push_view` snapshots the outgoing grid and composites it with the incoming grid during the transition.
- Widgets borrow `&self` when rendering (like ratatui's `WidgetRef`), so they can live in an `App` and animate from `on_tick`; mouse hit-testing uses the layout rects computed the same way as rendering.
- The `gpu` feature is off by default; wgpu, pollster, and the offscreen render/readback path are only compiled in when it is enabled.
- Orb widgets reference primitives from thinking-orbs (smoothstep), liquid-gooey (spring engine), and Libraries.dev border-beam (traveling beams): those projects are read-only references, not dependencies.
- Transparency is simulated by alpha-blending semi-transparent cells over the grid's canvas colors before storing; the diff renderer writes only solid cells. Set canvas colors with `App::set_canvas_colors` to match your terminal theme so glass panels look crisp.

## AI agent harness

The `agent_harness` example demonstrates booms for building AI agent interfaces:

- **Chat view**: message history with role indicators (user=blue, agent=yellow)
- **Glass panels**: each view (chat/settings/vibe/about) is rendered inside a
  semi-transparent rounded panel with crisp transparency over the canvas
- **Glow-style markdown**: agent replies render as markdown: headings,
  numbered/bulleted/task lists, code blocks, blockquotes, links, and inline
  bold/italic/strike/code: streamed character-by-character while it types
- **Word-wrapped messages**: long messages wrap with indented continuations
- **Scrollable history**: PageUp/PageDown scroll, a `Scrollbar` shows position,
  and the view follows the newest messages while at the bottom
- **Streaming output**: agent responses reveal character-by-character with a cursor
- **Orb-like interface**: `GlassOrb` shows agent thinking state, `NeonOrb`
  glows in the status bar, `PulseOrb` pulses, and an `Asciify` wave scene runs
  under the header while the agent thinks
- **Multi-view navigation**: chat → settings → vibe → about, each view in its
  own glass panel
- **Asciify animations**: an animated ASCII scene (wave/fog/orbits/pulses)
  runs under the header while the agent thinks, and a full-screen vibe
  view shows the effects in large, cycling on Ctrl+V
- **Input field**: a starship-style prompt line with capsules, a right-side clock, and an editable text field for user messages; pasting and mouse clicks work
- **Status bar**: real-time tick counter and status indicator with neon orb

```sh
cargo run --example agent_harness
```

Keys: type and press Enter to send, PageUp/PageDown scroll history, Ctrl+N starts a new chat, Ctrl+S for settings, Ctrl+A for about, Escape to quit.
