---
id: "0010"
title: "TOML Configuration at ~/.config/swayve/config.toml with Serde Defaults"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-07-28
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Commits 46e5c13, fdbf1cd, 3ae5438; src/config.rs; README.md Configuration section"
---

# 10. TOML Configuration at ~/.config/swayve/config.toml with Serde Defaults

## Context and Problem Statement
Border colours, terminal command, ancestry depth, and hook commands vary per user. The config needs to load without an installer step, and a new option must not break existing files.

## Decision Drivers
* Users edit one file by hand.
* New fields must not invalidate older config files.
* Sway users already keep config under `~/.config`.

## Considered Options
1. Environment variables only.
2. TOML at `$HOME/.config/swayve/config.toml`, loaded by `serde` with defaults.
3. XDG-aware lookup using `XDG_CONFIG_HOME` with a library.

## Decision Outcome
Chosen option: "TOML at `$HOME/.config/swayve/config.toml` with `serde` defaults", because it needs no extra crate and missing sections fall back to defaults. `DaemonConfig` and `HooksConfig` carry `#[serde(default)]` for this reason.

### Consequences
* **Positive**: Adding a field is a default function plus a struct field. Old files still load.
* **Negative**: The path is built from `HOME` directly, so `XDG_CONFIG_HOME` is ignored. Commit `46e5c13` simplified path construction and kept this behaviour.
* Amended 2026-11-08 (Orchestration 20): XDG_CONFIG_HOME is honored when absolute; see the Configuration section of README.md.
* **Negative**: A malformed file falls back silently to defaults in `load_config` (`src/config.rs`). Users see defaults, not an error.

## Validation & Compliance
`src/config.rs` tests cover hooks defaults, hooks parsing, and daemon auto-monitor parsing. Process-tree audit options (`fdbf1cd`) were added as a defaulted section, showing the pattern works.
