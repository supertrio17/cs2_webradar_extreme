# Overlay mode

Overlay mode is optional and gated by platform support.

## Behavior

- `--overlay` enables overlay profile request.
- `--click-through` requests pointer passthrough mode.
- On non-Windows environments, runtime logs fallback to desktop mode.

## Design constraints

- Keep standalone desktop mode as default and always available.
- Overlay state is represented in UI settings for profile persistence.
- Rendering remains the same snapshot pipeline in both modes.

## Future implementation details

When full native overlay APIs are wired:

- Use transparent always-on-top window profile.
- Bind global hotkey to toggle click-through at runtime.
- Add runtime capability probe in diagnostics so users understand when overlay degraded.
