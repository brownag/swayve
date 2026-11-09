.PHONY: help all build check test clean reload run daemon daemon-dry hud

# Default target: display help instructions
help:
	@echo "=========================================================================="
	@echo "                         swayve Development Makefile"
	@echo "=========================================================================="
	@echo "Available targets:"
	@echo "  all       : Run check, test, and build the release binaries"
	@echo "  build     : Compile the release workspace executables"
	@echo "  check     : Run cargo check and clippy across the workspace (warnings denied)"
	@echo "  test      : Run Rust workspace, Pi, Antigravity, and Claude hook tests"
	@echo "  clean     : Clean compile artifacts"
	@echo "  run       : Run the debug swayve CLI binary"
	@echo "  daemon    : Run the live background daemon (requires SWAYSOCK)"
	@echo "  daemon-dry: Run the daemon in dry-run mode (logs commands, offline safe)"
	@echo "  hud       : Run the live terminal HUD (swayve-hud)"
	@echo "  reload    : Reload Sway compositor configuration"
	@echo "  help      : Show this help message"
	@echo "=========================================================================="

# Build everything
all: check test build

# Compile release binaries
build:
	cargo build --workspace --release

# Compilation checks
check:
	cargo check --workspace
	cargo clippy --workspace --all-targets -- -D warnings

# Run tests
test:
	cargo test --workspace
	cd integrations/pi && npm test
	cd integrations/antigravity && node --test sidecar.test.mjs
	bash integrations/claude/test-hooks.sh

# Clean artifacts
clean:
	cargo clean

# Run debug version of the CLI
run:
	cargo run -p swayve -- --help

# Run the live background daemon (requires running Sway)
daemon:
	cargo run -p swayve -- daemon

# Run the daemon in dry-run mode for logging/mock testing
daemon-dry:
	cargo run -p swayve -- --dry-run daemon

# Run the live terminal HUD
hud:
	cargo run -p swayve-hud

# Reload Sway configuration rules
reload:
	swaymsg reload
