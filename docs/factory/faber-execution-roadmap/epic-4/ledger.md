# Epic 4 Ledger

## 2026-05-24: Phase 4.1 Started

Delivery spec: [`delivery-4.1.md`](delivery-4.1.md)

Scope:

- Implement the in-process Faber-owned host frame/kernel route proof in `hosts/macos-arm64`.
- Add `host:echo`, unresolved `E_NO_ROUTE`, manifest output, CLI commands, tests, provenance notes, and first HAL migration candidate.

Out of scope:

- Wasm/component loading.
- Background daemon/server transport.
- Full `norma` migration.
- Shared/common host crate extraction.

## 2026-05-24: Phase 4.1 Implemented

Implemented:

- Added a Faber-owned host kernel under `hosts/macos-arm64/src/kernel/`.
- Added serializable `Frame`, `Status`, and `HostError` contracts.
- Added prefix routing, `host:echo`, and structured `E_NO_ROUTE` unresolved-call behavior.
- Added manifest output for built-in syscalls and registered providers.
- Added route-proof CLI commands: `manifest` and `call <name> [json-object]`.
- Recorded Muninn semantic provenance in `hosts/macos-arm64/README.md`.
- Recorded `norma:hal/consolum` as the first HAL migration candidate.

Validation:

- `cargo fmt -p faber-host-macos-arm64`
- `cargo test -p faber-host-macos-arm64`
- `cargo run -p faber-host-macos-arm64 -- manifest`
- `cargo run -p faber-host-macos-arm64 -- call host:echo '{"value":"salve"}'`
- `cargo run -p faber-host-macos-arm64 -- call pg:query '{}'`
- `cargo tree -p faber-host-macos-arm64`

Evidence:

- `manifest` includes `host:echo`.
- `host:echo` returns a `done` frame with echoed payload.
- `pg:query` returns an `error` frame containing `E_NO_ROUTE`.
- `cargo tree` shows only `serde` and `serde_json` dependency families; there is no Muninn dependency.

Next recommended phase:

- `4.2`: attach a minimal Wasm/component import to the same frame router. The import ABI may be smaller than a full frame if the host wraps it into a `Frame` immediately before routing.
