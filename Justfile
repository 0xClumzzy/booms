# justfile - common development tasks for booms
# Install: cargo install just
# Run: just <command>

# Default recipes
default: check

# Format, lint, test
fmt:
	cargo fmt --all

check:
	cargo check --all-targets --all-features

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all-targets --all-features

test-doc:
	cargo test --doc --all-features

# All CI checks locally
ci: fmt clippy test test-doc

# Documentation
doc:
	cargo doc --all-features --no-deps --open

# Examples
run-demo:
	cargo run --example demo

run-orbs:
	cargo run --example orbs

run-transitions:
	cargo run --example transitions

run-rich:
	cargo run --example rich

run-aesthetic:
	cargo run --example aesthetic

run-agent:
	cargo run --example agent_harness

run-ratatui:
	cargo run --features ratatui --example native_ratatui

run-gpu:
	cargo run --features gpu --example gpu

# Run all examples
run-all: run-demo run-orbs run-transitions run-rich run-aesthetic run-agent

# Publish dry-run
publish-dry-run:
	cargo publish --dry-run

# Clean
clean:
	cargo clean

# Update dependencies
update:
	cargo update

# Generate lock file (if missing)
lock:
	cargo generate-lockfile

# Audit dependencies
audit:
	cargo audit

# Check MSRV locally (requires rustup toolchain install 1.85)
msrv:
	rustup run 1.85 cargo check --all-targets --all-features