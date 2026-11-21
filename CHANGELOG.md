# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of booms v0.1.0
- Core framework: `App` event loop, `Grid` cell renderer, `Term` terminal backend, `layout` engine
- Built-in widgets: `Text`, `Block`, `Input`, `List`, `Scrollbar`, `MarkdownText`, `Asciify`, `Panel`
- Data widgets: `Gauge`, `LineGauge`, `Sparkline`, `BarChart`, `Tabs`, `Table`
- Orb animations: `PulseOrb`, `CrystalOrb`, `RingOrb`, `LiquidGooeyOrb`, `WaveOrb`, `SpinnerOrb`, `GlassOrb`, `NeonOrb`, `AuraOrb`, `CometOrb`, `HeartbeatOrb`, `ThinkingOrb`, `RippleOrb`, `MorphOrb`
- Animation primitives: `Anim` (eased), `Spring` (spring physics), `Easing` curves
- View transitions: `Fade`, `Slide`, `Cube` via `Transition`
- Markdown rendering: `render_markdown`, `MarkdownText` widget (GitHub-flavored)
- Asciify animations: `AsciifyEffect` (Wave, Fog, Orbits, Pulses) with Bayer dithering
- Panel features: 4 border types, wrap-around titles, opacity/transparency support
- Crisp transparency: `Style::opacity`, `App::set_canvas_colors` for theme-aware glass panels
- Starship-style prompt: `StarshipPrompt`, `StarshipInput` with capsules and clock
- Mouse + bracketed paste + focus events
- Optional feature `ratatui`: `RatatuiView` to embed real ratatui widgets
- Optional feature `gpu`: `GpuSurface` for headless wgpu plasma shader rendering
- AI agent harness example with streaming markdown, glass panels, orb indicators
- 58 tests (unit, integration, doc tests)
- Clean clippy, rustfmt, cargo audit

### Changed
- N/A (initial release)

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

---

## Release Notes Template

When releasing a new version, use this template:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- ...

### Changed
- ...

### Deprecated
- ...

### Removed
- ...

### Fixed
- ...

### Security
- ...
```