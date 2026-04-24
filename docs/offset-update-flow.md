# Offset and schema update flow

## Runtime selection order

`dump_runtime::resolve_active_dump` attempts packs in this order:

1. User-updated pack (`%LOCALAPPDATA%/cs2_webradar_extreme/dumps/active` by default).
2. Bundled pack (`data/dumps/embedded`).
3. Hard failure if no usable pack exists.

If an expected build number is supplied, the loader prefers exact build matches and records mismatch warnings.

## Refresh command

Desktop command:

```bash
cs2_webradar_extreme dump refresh --output <dir> --build-number <n> [--binary client.dll] [--schema client_dll.json]
```

The refresh pipeline:

1. Optionally scans client binary signatures for key offsets.
2. Normalizes schema JSON to a compact class/field shape.
3. Writes `info.json`, `offsets.json`, and `client_dll.json`.

## Forward compatibility strategy

- Protocol contracts are versioned (`protocol_version`).
- Build mismatch is visible in diagnostics before feature extraction fails.
- Embedded pack remains a safe fallback when user pack is missing/corrupt.
