# Contributing to booms

Thank you for your interest in contributing! This document outlines the process for contributing to booms.

## Quick Start

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/your-feature-name`
3. Make your changes
4. Run the quality checks: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets --all-features`
5. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.85+ (MSRV; see `rust-version` in `Cargo.toml`)
- `cargo` (comes with Rust)

### Optional Dependencies

For GPU feature development:
- A GPU with Vulkan/Metal/DX12 support, or software adapter (llvmpipe, SwiftShader)

### Running Examples

```bash
# Core examples
cargo run --example demo
cargo run --example orbs
cargo run --example transitions
cargo run --example rich
cargo run --example aesthetic

# Feature-gated examples
cargo run --features ratatui --example native_ratatui
cargo run --features gpu --example gpu

# Full AI agent harness
cargo run --example agent_harness
```

## Code Style

- **Format**: `cargo fmt --all` (enforced in CI)
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings` (enforced in CI)
- **Tests**: `cargo test --all-targets --all-features` (must pass)
- **Docs**: `cargo doc --all-features --no-deps` (must build without warnings)

## Pull Request Guidelines

### Before Submitting

- [ ] Code compiles without warnings (`cargo check --all-targets --all-features`)
- [ ] All tests pass (`cargo test --all-targets --all-features`)
- [ ] Clippy passes (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --all -- --check`)
- [ ] Documentation builds (`cargo doc --all-features --no-deps`)
- [ ] New public API has doc comments with examples where appropriate
- [ ] CHANGELOG.md updated (under `## [Unreleased]`)

### Commit Messages

Follow conventional commits where practical:
- `feat: add new orb animation`
- `fix: handle resize during transition`
- `docs: update Panel opacity example`
- `refactor: simplify Widget trait`
- `test: add integration test for List focus`

### Breaking Changes

booms is pre-1.0. Minor version bumps (0.Y.0) may contain breaking changes.
- Document breaking changes clearly in the PR description
- Update CHANGELOG.md under `### Changed` or `### Removed`
- Consider providing migration notes

## Architecture Overview

```
src/
├── lib.rs           # Public API re-exports, crate docs
├── app.rs           # App event loop, view stack, transitions
├── grid.rs          # Cell grid, diff renderer, canvas colors
├── layout.rs        # Constraint-based column layout
├── term.rs          # Crossterm backend (raw mode, alt screen, mouse)
├── style.rs         # Color, Style, Modifiers, opacity
├── anim.rs          # Anim (eased), Spring, Easing, Interpolate
├── transitions.rs   # Fade, Slide, Cube transitions
├── markdown.rs      # Markdown parsing + rendering
├── asciify.rs       # Asciify effects (Wave, Fog, Orbits, Pulses)
├── widgets/
│   ├── mod.rs       # Widget trait, public re-exports
│   ├── text.rs      # Text widget
│   ├── block.rs     # Block widget
│   ├── input.rs     # Input widget
│   ├── list.rs      # List widget
│   ├── scrollbar.rs # Scrollbar widget
│   ├── panel.rs     # Panel widget (borders, titles, opacity)
│   ├── markdown.rs  # MarkdownText widget
│   ├── asciify.rs   # Asciify widget
│   ├── ratatui.rs   # Data widgets (Gauge, Sparkline, etc.)
│   ├── ratatui_native.rs # RatatuiView (feature: ratatui)
│   ├── starship.rs  # StarshipPrompt, StarshipInput
│   ├── orbs.rs      # Orb base + AuraOrb, CometOrb, GlassOrb, NeonOrb, HeartbeatOrb, RippleOrb, SpinnerOrb, MorphOrb
│   ├── crystal_orb.rs
│   ├── pulse_orb.rs
│   ├── ring_orb.rs
│   ├── gooey_orb.rs
│   ├── wave_orb.rs
│   └── thinking_orb.rs
```

### Key Design Principles

1. **Widgets borrow `&self` in `render`**: like ratatui's `WidgetRef`, enabling long-lived widgets in `App` that animate via `on_tick`
2. **Single-threaded, tick-driven**: no background threads; animations advance on `App::tick_interval` (default 50ms)
3. **Diff-based rendering**: only changed cells written to terminal
4. **View stack with transitions**: `push_view`/`pop_view` snapshot outgoing grid, composite during transition
5. **Transparency via alpha-blending**: semi-transparent cells blended over canvas colors before diff; no terminal alpha required

## Adding a New Widget

1. Create `src/widgets/your_widget.rs`
2. Implement the `Widget` trait from `crate::widgets::Widget`
3. Add module declaration in `src/widgets/mod.rs`
4. Re-export in `src/widgets/mod.rs` and `src/lib.rs`
5. Add example usage in an existing example or create a new one
6. Add tests in `src/widgets/your_widget.rs` (unit) and/or `tests/widgets.rs` (integration)

## Testing

- Unit tests: `#[cfg(test)] mod tests { ... }` in each source file
- Integration tests: `tests/widgets.rs`
- Doc tests: examples in doc comments (run with `cargo test --doc`)

## Releasing

Maintainers only:

```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md (move Unreleased entries to version section)
# 3. Commit: "chore: release vX.Y.Z"
# 4. Tag: git tag vX.Y.Z
# 5. Push: git push && git push --tags
# 6. CI will publish to crates.io (if configured with CARGO_REGISTRY_TOKEN)
```

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). By participating, you agree to uphold this code.

## Getting Help

- Open an issue for bugs or feature requests
- Start a discussion for questions or design proposals
- Check existing issues/PRs before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the MIT License (same as the project).