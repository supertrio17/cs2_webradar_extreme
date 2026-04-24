# Architecture

## Workspace

`cs2_webradar_extreme` is split into focused Rust crates plus a TypeScript UI:

- `apps/desktop/src-tauri`: desktop entrypoint and command surface.
- `crates/contracts`: versioned snapshot schema for backend/frontend parity.
- `crates/dump_runtime`: runtime dump pack loading with build-aware fallback.
- `crates/dump_refresh`: integrated refresh pipeline producing normalized JSON dump packs.
- `crates/core_engine`: extraction/normalization loop and smart feature scoring.
- `ui`: neon React application with componentized radar layout.

## Runtime UI hosting modes

Two UI host modes are supported by the runtime:

1. **Desktop mode** (`run --ui-mode desktop`, default)
   - No localhost server required.
   - Intended for normal EXE startup behavior.
   - Overlay-related flags (`--overlay`, `--click-through`) apply here.

2. **Browser mode** (`run --ui-mode browser`)
   - Hosts UI over `http://127.0.0.1:<port>`.
   - Default port is `4173`.
   - Deterministic fallback range is `4173..=4183` by default (configurable with `--ui-port` and `--ui-port-fallback-span`).
   - Final bound URL is logged at runtime and printed to stdout.
   - Browser auto-open is enabled by default (can be disabled with `--open-browser=false`).

## Data flow

### Desktop mode flow

1. Desktop runtime selects active dump pack (`user` first, then `embedded`).
2. Engine polls provider for raw frame data.
3. Raw entities are normalized to `RadarSnapshot` (protocol versioned).
4. Backend publishes envelopes over internal event stream.
5. UI ingests snapshots, keeps previous+latest for interpolation, renders at animation frame cadence.

### Browser mode flow

1. Runtime resolves frontend static assets (`ui/dist` or configured override path).
2. Localhost server binds on configured/default port with deterministic fallback.
3. Browser loads React UI from `127.0.0.1`.
4. If desktop bridge APIs are unavailable, UI uses mock snapshot stream for standalone browser operation.

## Reliability model

- Feature code never reads compile-time offsets directly.
- Dump validation is build-aware and logs explicit mismatch diagnostics.
- Engine emits recovering/waiting states during process or map transitions.
- Snapshot contracts include diagnostics and stale entity indicators for rendering/UI fallback behavior.
