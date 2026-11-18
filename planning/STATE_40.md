# Orchestration 40 State: OpenCode Session Discovery and Sway Window Association (G93)

Source: `planning/EXECUTION_PLAN_40.md`
Target repo: `/home/andrew/workspace/swayve`
Target branch: `feat/opencode-integration1`

## Baseline Checklist

- [x] `git status` confirmed branch `feat/opencode-integration1`; pre-existing `planning/RESEARCH_OPENCODE_INTEGRATION.md` change is preserved and must remain unstaged.
- [x] OpenCode runtime version recorded as v2.3.2; no live session contents or prompts inspected during planning.
- [x] OpenCode focused baseline: `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs` — 4/4 passed; existing harmless `MODULE_TYPELESS_PACKAGE_JSON` warning.
- [x] Full workspace baseline: `make all` passed on 2026-11-18. Workspace check/clippy/build passed; Rust tests passed (90 passed, 2 ignored); Pi 8/8; Antigravity 14/14; Claude hook tests passed. OpenCode tests are not part of `make all`.
- [x] Read root and HUD `AGENTS.md`; relevant ADRs 0003, 0004, 0006, 0007, 0008, 0009, and 0012; current OpenCode plugin/API docs and Swayve identity/daemon/HUD code.
- [x] Planning artifacts are to be tracked, with planning-only commits separate from implementation. No remote operations.

## Wave Board

| Wave | Task | Status | Worker / Session |
|---|---|---|---|
| 1 | 40.1.1 | pending | — |
| 2 | 40.2.1 | pending (D8 identity/deletion evidence gate) | — |
| 3 | 40.3.1 | pending | — |
| 3 | 40.3.2 | pending | — |
| 4 | 40.4.1 | pending; user-assisted validation only | — |

## Task Reports

### 40.1.1 — Establish Session and Window Identity Contract

Pending.
