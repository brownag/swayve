.PHONY: help all build check test clean reload run daemon daemon-dry

# Default target: display help instructions
help:
	@echo "=========================================================================="
	@echo "                         swayve Development Makefile"
	@echo "=========================================================================="
	@echo "Available targets:"
	@echo "  all       : Run check, test, and build the release binary"
	@echo "  build     : Compile the release executable"
	@echo "  check     : Run cargo check for compilation sanity"
	@echo "  test      : Run Rust, Pi and hook tests"
	@echo "  clean     : Clean compile artifacts"
	@echo "  run       : Run the debug binary"
	@echo "  daemon    : Run the live background daemon (requires SWAYSOCK)"
	@echo "  daemon-dry: Run the daemon in dry-run mode (logs commands, offline safe)"
	@echo "  reload    : Reload Sway compositor configuration"
	@echo "  help      : Show this help message"
	@echo "=========================================================================="

# Build everything
all: check test build

# Compile release binary
build:
	cargo build --release

# Compilation checks
check:
	cargo check

# Run tests
test:
	cargo test
	cd integrations/pi && npm test
	bash integrations/claude/test-hooks.sh

# Clean artifacts
clean:
	cargo clean

# Run debug version of the CLI
run:
	cargo run -- --help

# Run the live background daemon (requires running Sway)
daemon:
	cargo run -- daemon

# Run the daemon in dry-run mode for logging/mock testing
daemon-dry:
	cargo run -- --dry-run daemon

# Reload Sway configuration rules
reload:
	swaymsg reload
