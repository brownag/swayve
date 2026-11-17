.PHONY: help all build check test clean reload run daemon daemon-dry hud install-user

.DEFAULT_GOAL := help

INSTALL_ROOT ?= $(HOME)/.local

help: ## Show this help message
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / { printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

all: check test build ## Run check, test, and build the release binaries

build: ## Compile the release workspace executables
	cargo build --workspace --release

check: ## Run cargo check and clippy across the workspace (warnings denied)
	cargo check --workspace
	cargo clippy --workspace --all-targets -- -D warnings

test: ## Run Rust workspace, Pi, Antigravity, and Claude hook tests
	cargo test --workspace
	cd integrations/pi && npm test
	cd integrations/antigravity && node --test sidecar.test.mjs
	bash integrations/claude/test-hooks.sh

clean: ## Clean compile artifacts
	cargo clean

run: ## Run the debug swayve CLI binary
	cargo run -p swayve -- --help

daemon: ## Run the live background daemon (requires SWAYSOCK)
	cargo run -p swayve -- daemon

daemon-dry: ## Run the daemon in dry-run mode (logs commands, offline safe)
	cargo run -p swayve -- --dry-run daemon

hud: ## Run the live terminal HUD (swayve-hud)
	cargo run -p swayve-hud

install-user: ## Install swayve and swayve-hud to ~/.local/bin (no sudo; override INSTALL_ROOT)
	cargo install --locked --path crates/swayve --root $(INSTALL_ROOT)
	cargo install --locked --path crates/swayve-hud --root $(INSTALL_ROOT)

reload: ## Reload Sway compositor configuration
	swaymsg reload
