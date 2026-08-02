# Execution Plan: Wave 8 — Claude Code Lifecycle Hook Integration

## Goal & Scope

**Goal**: Provide a production-grade, automated setup for integrating Claude Code with `swayve` using Claude's native lifecycle hooks. Create a template configuration and an installation script `integrations/claude/install-hooks.sh` that configures Claude to automatically emit `agent.update_status` updates to `swayve` when starting sessions or running tools.

**In scope:**
- Creating `integrations/claude/install-hooks.sh` to automate the configuration of Claude's hooks.
- Authoring `integrations/claude/README.md` to document the setup.
- Verifying the generated settings JSON syntax and bash script logic.

**Out of scope:**
- Modifying `swayve` Rust source files.

## Decision Log

- **D1: Hook Setup Location**: Hooks will be configurable project-wide (in a project's `.claude/settings.json`) or globally (in `~/.claude/settings.json`).
- **D2: Automation Tool**: We will provide a bash installation script (`install-hooks.sh`) that reads the current `settings.json`, merges the `hooks` configuration, and writes it back cleanly.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 8** | **8.1** | Create Claude hook templates and installer | `integrations/claude/install-hooks.sh`, `integrations/claude/README.md` | None |

---

## Task Specifications

### Task 8.1: Create Claude Hook Templates and Installer

**ID**: `8.1`  
**Repo**: `swayve`  
**Files-in-scope**: `integrations/claude/install-hooks.sh`, `integrations/claude/README.md`  
**Depends-on**: None  

**Spec prose**:
1. Create `integrations/claude/install-hooks.sh`:
   - It should check if `~/.claude/settings.json` (or a local `.claude/settings.json`) exists.
   - It should parse/construct a clean JSON merge of the `hooks` section (events: `SessionStart`, `PreToolUse`, `PostToolUse`, `Stop`) wrapping standard `nc -U` status reporting calls.
   - It should safely update the settings file. If `jq` is available, use it for merging JSON; otherwise, write a template settings file or append cleanly.
2. Create `integrations/claude/README.md`:
   - Explain how Claude Code uses the lifecycle hooks (`SessionStart`, `PreToolUse`, `PostToolUse`, `Stop`) to report state back to `swayve` dynamically.
   - Provide command examples for running `install-hooks.sh`.

**Constraints**:
- Ensure all scripts have appropriate execution permissions and correct shebangs where applicable.

**Acceptance**: The script executes without syntax errors and correctly writes/generates a valid `.claude/settings.json` containing the hooks.
