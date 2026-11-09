# swayve-hud

A standalone terminal Heads-Up Display (HUD) for Swayve.

`swayve-hud` consumes Swayve's `daemon.subscribe_diff` stream over a Unix domain socket, maintains a live folded map of active agent windows, and renders in-place terminal updates with automatic reconnection and backoff across socket disconnects or daemon restarts.

## Socket Path Resolution

The daemon socket path is resolved using the following order of precedence:
1. `$SWAYVE_SOCKET`: Used if set and non-empty.
2. `$XDG_RUNTIME_DIR/swayve/swayve.sock`: Fallback if `$SWAYVE_SOCKET` is not set or empty.

If neither environment variable provides a usable path, the HUD reports an error and halts.

## Building and Running

### Prerequisites
- Rust (Edition 2024 compatible toolchain, e.g. rustc >= 1.85 / 1.99)
- Cargo
- `make`

### Build Targets
- `make check`: Run compiler check and Clippy with `-D warnings`.
- `make test`: Run unit and integration tests.
- `make build`: Compile the optimized release binary (`target/release/swayve-hud`).
- `make all`: Run `check`, `test`, and `build`.
- `make clean`: Remove build artifacts.

### Running
Run directly via Cargo (or from the workspace root via `swayve hud` / `make hud`):
```bash
cargo run -p swayve-hud
```
Or run the compiled release binary:
```bash
./target/release/swayve-hud
```
