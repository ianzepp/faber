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

## 2026-05-24: Phase 4.2 Started

Delivery spec: [`delivery-4.2.md`](delivery-4.2.md)

Scope:

- Add a minimal Wasmtime component runner in `hosts/macos-arm64`.
- Expose a root component import named `capability-call`.
- Keep the proof ABI intentionally small and route-code based so the final Faber WIT/string ABI remains a later lowering decision.
- Prove component route code `1` reaches `host:echo` through the frame router.
- Prove component route code `2` reaches unresolved `pg:query` through the frame router and returns `E_NO_ROUTE`.

Out of scope:

- Final Faber WIT world.
- Faber compiler outputting Wasm components.
- Daemon/server transport.

## 2026-05-24: Phase 4.2 Implemented

Implemented:

- Added `hosts/macos-arm64/src/component.rs`.
- Added `ComponentHost`, a Wasmtime-backed component runner.
- Added root component import `capability-call(route-code: s32) -> s32`.
- The host import wraps route code `1` as a `host:echo` frame and route code `2` as a `pg:query` frame, then routes through the same `HostKernel` used by the direct CLI.
- Added `component-call <component> <export> <route-code>` CLI command.
- Added reusable component fixture `hosts/macos-arm64/tests/fixtures/route-proof.wat`.
- Added component boundary tests for successful `host:echo`, unresolved `pg:query`, and CLI fixture loading.

Validation:

- `cargo fmt -p faber-host-macos-arm64`
- `cargo test -p faber-host-macos-arm64`
- `cargo clippy -p faber-host-macos-arm64 -- -D warnings`
- `cargo test -p faber-host-macos-arm64 --test component_host_test`
- `cargo run -p faber-host-macos-arm64 -- component-call hosts/macos-arm64/tests/fixtures/route-proof.wat route 1`
- `cargo run -p faber-host-macos-arm64 -- component-call hosts/macos-arm64/tests/fixtures/route-proof.wat route 2`

Evidence:

- Route code `1` returns a `done` frame for `host:echo` with echoed payload.
- Route code `2` returns an `error` frame for `pg:query` containing `E_NO_ROUTE`.
- Both component routes use the same `HostKernel` path as direct frame calls.

Remaining deferred work:

- `4.3` daemon/server transport remains deferred until provider registration, shared service lifecycle, or multi-process capability routing needs it, as defined in the roadmap.
