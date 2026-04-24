# cs2_webradar_extreme

`cs2_webradar_extreme` is a clean rebuild of the older CS2 web radar stack into a modern desktop-oriented architecture.

## Highlights

- Rust-first engine and dump runtime.
- Versioned backend/frontend snapshot contracts.
- Runtime-loaded offsets and schemas (no hardcoded offsets in feature code).
- Dump refresh command that emits normalized JSON packs.
- Optional overlay-mode flags and desktop-safe fallback behavior.
- React + TypeScript neon radar UI (componentized and state-driven).
- Localhost browser hosting mode for UI testing and remote-style access.

## Safety and legal notice

This project interacts with game process memory and may violate game terms, anti-cheat policies, or regional law depending on use. You are solely responsible for compliance and risk.

## Runtime modes

### 1) Default desktop startup (no CLI flags required)

Double-clicking the built executable starts the app in default desktop mode automatically.

- Windows EXE behavior with no arguments:
  - `cs2_webradar_extreme.exe`
  - Equivalent to: `cs2_webradar_extreme.exe run --ui-mode desktop`
- No mandatory `--options` are required for normal startup.
- Advanced flags remain optional.

### 2) Localhost browser-hosted UI mode

Start browser-hosted mode with:

```bat
cs2_webradar_extreme.exe run --ui-mode browser
```

Localhost behavior:

- Host: `127.0.0.1`
- Default port: `4173`
- Deterministic fallback range: `4173` through `4183` (configurable)
- Runtime logs and stdout print the exact final URL in use.
- Browser auto-open is enabled by default in browser mode.

Default URL:

- `http://127.0.0.1:4173`

If the default port is busy, the app tries the next port in sequence until an available one is found.

Optional browser-hosting overrides:

```bat
cs2_webradar_extreme.exe run --ui-mode browser --ui-port 5000 --ui-port-fallback-span 20
cs2_webradar_extreme.exe run --ui-mode browser --open-browser=false
```

## CLI usage

### Common commands

```bat
cs2_webradar_extreme.exe
cs2_webradar_extreme.exe --ui-mode browser
cs2_webradar_extreme.exe run
cs2_webradar_extreme.exe run --ui-mode browser
cs2_webradar_extreme.exe dump refresh --output .\runtime-data\dumps\active --build-number 15000
```

### Optional advanced run flags (power users)

- `--rate-hz <n>`: engine tick rate (default `25`)
- `--stream-json`: emit JSON envelopes to stdout
- `--max-ticks <n>`: stop after N ticks
- `--overlay`: request overlay profile (Windows-capability dependent)
- `--click-through`: request overlay click-through mode
- `--ui-mode <desktop|browser>`: select runtime UI host mode
- `--ui-port <port>`: preferred localhost port for browser mode
- `--ui-port-fallback-span <n>`: number of sequential fallback ports to try
- `--open-browser=<true|false>`: auto-open browser in browser mode

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

### Build steps (Release / x64)

1. Run `scripts\install-prerequisites.bat` (recommended) or install prerequisites manually.
2. Open `cs2_webradar_extreme.sln` in Visual Studio.
3. Set **Configuration** to `Release`.
4. Set **Platform** to `x64`.
5. Build via `Build > Build Solution` (or `Rebuild`).

Visual Studio runs `scripts/build-windows.ps1`, which executes:

1. `npm --prefix ./ui ci`
2. `npm --prefix ./ui run build` (generates `ui/dist`)
3. `cargo build -p cs2_webradar_extreme --target x86_64-pc-windows-msvc --release`

No separate Vite dev server is required for production builds.

## Output artifacts and what to run/share

Primary executable paths:

- Release/x64:
  - `target/x86_64-pc-windows-msvc/release/cs2_webradar_extreme.exe`
- Debug/x64:
  - `target/x86_64-pc-windows-msvc/debug/cs2_webradar_extreme.exe`

Run this for normal usage/distribution in the current flow:

- `target/x86_64-pc-windows-msvc/release/cs2_webradar_extreme.exe`

If/when Tauri bundling is enabled, bundle artifacts are emitted under:

- `target/x86_64-pc-windows-msvc/release/bundle/<bundle-type>/...`

In bundling mode, share installer artifacts from `bundle`; otherwise share the release EXE above.

## Branch consolidation + default branch notes

If your automation token does not have repository-admin permissions, complete these steps in GitHub UI:

1. Open repository **Settings**.
2. Go to **Branches**.
3. Under **Default branch**, click **Change default branch**.
4. Select `main` and confirm.
5. Merge this implementation branch into `main` (via PR or fast-forward).
6. Delete obsolete branches after merge (for example old `cto/*` implementation branches) from:
   - **Pull Requests > Closed** (Delete branch button), or
   - **Branches** page.

Suggested CLI alternative for maintainers with permission:

```bash
git checkout main
git pull
git merge --ff-only cto/implement-default-startup-localhost-ui
git push origin main
git push origin --delete cto/add-install-prerequisites-bat cto/add-vs-build-files-exe cto/fix-type-mismatch-between-playerstate-cls cto/add-extreme-rebuild-setup
```

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
