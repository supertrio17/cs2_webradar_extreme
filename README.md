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

## Visual Studio desktop build (Windows)

You can build the full desktop app (frontend + Rust runtime) directly from Visual Studio via the included solution.

### One-click prerequisites install (recommended)

From an **elevated Command Prompt** in the repository root:

```bat
scripts\install-prerequisites.bat
```

The installer is idempotent and uses `winget` to install/verify:

- Visual Studio 2022 Build Tools + C++ workload (MSVC v143 + Windows SDK)
- Microsoft Edge WebView2 Runtime
- Rust stable + `x86_64-pc-windows-msvc` target
- Node.js LTS + npm
- Git

Optional verification-only check:

```bat
scripts\verify-prerequisites.bat
```

If `winget` is unavailable, the installer prints clear fallback guidance (including an optional Chocolatey command).

### Manual prerequisites (fallback)

- Visual Studio 2022 (or Build Tools) with **Desktop development with C++** installed.
  - MSVC v143 x64 toolset
  - Windows 10/11 SDK
- Rust stable toolchain + Windows target:
  - `rustup toolchain install stable`
  - `rustup target add x86_64-pc-windows-msvc`
- Node.js (LTS) + npm (for building the React UI)
- Git
- WebView2 Runtime (required by Tauri-based desktop runtimes)

### Build steps

1. Run `scripts\install-prerequisites.bat` (recommended) or install prerequisites manually.
2. Open `cs2_webradar_extreme.sln` in Visual Studio.
3. Select configuration: **Release**.
4. Select platform: **x64**.
5. Build the solution (`Build > Build Solution`) or run `Rebuild`.

Visual Studio calls `scripts/build-windows.ps1`, which runs the end-to-end pipeline:

1. `npm --prefix ./ui ci`
2. `npm --prefix ./ui run build` (generates `ui/dist` used by desktop packaging)
3. `cargo build -p cs2_webradar_extreme --target x86_64-pc-windows-msvc --release`

No separate Vite dev server is required for production builds.

### Output artifacts

- Primary desktop executable (Release/x64):
  - `target/x86_64-pc-windows-msvc/release/cs2_webradar_extreme.exe`
- Debug executable (Debug/x64):
  - `target/x86_64-pc-windows-msvc/debug/cs2_webradar_extreme.exe`

If/when Tauri installer bundling is enabled for this project, bundled installer/exe artifacts are emitted under:

- `target/x86_64-pc-windows-msvc/release/bundle/<bundle-type>/...`

For distribution in that mode, share the installer artifact from the `bundle` directory; otherwise share `target/x86_64-pc-windows-msvc/release/cs2_webradar_extreme.exe`.

## Repository layout

- `apps/desktop/src-tauri`: desktop runtime executable.
- `crates/contracts`: versioned snapshot schema for backend/frontend parity.
- `crates/dump_runtime`: dump selection and validation.
- `crates/dump_refresh`: normalized dump pack generation.
- `crates/core_engine`: extraction/normalization pipeline.
- `ui`: React + TypeScript frontend.
- `data/dumps/embedded`: bundled fallback dump pack.
- `docs`: architecture and operation docs.
- `scripts/build-windows.ps1`: one-command Windows release build helper.
- `scripts/install-prerequisites.bat`: one-click Windows prerequisite installer (winget-first).
- `scripts/verify-prerequisites.bat`: prerequisite verification with non-zero exit on missing dependencies.
