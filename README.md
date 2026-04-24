# cs2_webradar_extreme

`cs2_webradar_extreme` is a clean rebuild of the older CS2 web radar stack into a modern desktop-oriented architecture.

## Highlights

- Rust-first engine and dump runtime.
- Versioned backend/frontend snapshot contracts.
- Runtime-loaded offsets and schemas (no hardcoded offsets in feature code).
- Dump refresh command that emits normalized JSON packs.
- Optional overlay-mode flags and desktop-safe fallback behavior.
- React + TypeScript neon radar UI (componentized and state-driven).

## Safety and legal notice

This project interacts with game process memory and may violate game terms, anti-cheat policies, or regional law depending on use. You are solely responsible for compliance and risk.

## Quick start

```bash
cargo run -p cs2_webradar_extreme -- run --stream-json
cargo run -p cs2_webradar_extreme -- dump refresh --output ./runtime-data/dumps/active --build-number 15000
```

## Repository layout

- `apps/desktop/src-tauri`: desktop runtime executable.
- `crates/core_engine`: radar extraction/normalization pipeline.
- `crates/dump_runtime`: dump selection and validation.
- `crates/dump_refresh`: normalized dump pack generation.
- `crates/contracts`: versioned shared snapshot schema.
- `ui`: React + TypeScript frontend.
- `data/dumps/embedded`: bundled fallback dump pack.
- `docs`: architecture and operation docs.
- `scripts/build-windows.ps1`: one-command Windows release build helper.
