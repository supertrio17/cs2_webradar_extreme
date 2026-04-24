# Architecture

## Workspace

`cs2_webradar_extreme` is split into focused Rust crates plus a TypeScript UI:

- `apps/desktop/src-tauri`: desktop entrypoint and command surface.
- `crates/contracts`: versioned snapshot schema for backend/frontend parity.
- `crates/dump_runtime`: runtime dump pack loading with build-aware fallback.
- `crates/dump_refresh`: integrated refresh pipeline producing normalized JSON dump packs.
- `crates/core_engine`: extraction/normalization loop and smart feature scoring.
- `ui`: neon React application with componentized radar layout.

## Data flow

1. Desktop runtime selects active dump pack (`user` first, then `embedded`).
2. Engine polls provider for raw frame data.
3. Raw entities are normalized to `RadarSnapshot` (protocol versioned).
4. Backend publishes envelopes over internal event stream.
5. UI ingests snapshots, keeps previous+latest for interpolation, renders at animation frame cadence.

## Reliability model

- Feature code never reads compile-time offsets directly.
- Dump validation is build-aware and logs explicit mismatch diagnostics.
- Engine emits recovering/waiting states during process or map transitions.
- Snapshot contracts include diagnostics and stale entity indicators for rendering/UI fallback behavior.
