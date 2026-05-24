# Epic 4.2 Delivery Spec: Component Import Route Proof

## Interpreted Problem

Epic 4.1 proved the host kernel can route in-process frames. Epic 4.2 needs to prove the Wasm/component boundary does not bypass that frame model: a component import should enter the host as a small ABI, be wrapped into a `Frame`, and route through the same `HostKernel`.

## Normalized Spec

- Add a host-side Wasmtime component runner in `hosts/macos-arm64`.
- Expose one root component import named `capability-call`.
- Use a deliberately tiny proof ABI: `capability-call(route_code: u32) -> u32`.
- Map route code `1` to a `host:echo` frame request and route code `2` to an unresolved `pg:query` frame request.
- Store the routed response frame on the host side and return status code `0` for success or `1` for error to the component.
- Add a CLI command that loads a component file and calls an exported component function with a route code.
- Add tests with a minimal component fixture that re-exports the host import through a component function.
- Keep daemon/server transport out of scope.

## Repo-Aware Baseline

- `hosts/macos-arm64/src/kernel/` already owns the frame router.
- `hosts/macos-arm64/src/main.rs` already supports direct `manifest` and `call` route proofs.
- Wasmtime's component embedding API supports `component::Linker`, root imports, and `func_wrap`; the first Faber host can use this without binding a final WIT world.
- The route proof uses a numeric ABI rather than strings to avoid locking Faber's final WIT/string ABI before Epic 5 lowering work.

## Stage Graph

1. Add a `component` module that owns Wasmtime setup and route wrapping.
2. Add a `component-call` CLI command.
3. Add component fixture tests for successful `host:echo` and unresolved `pg:query`.
4. Update README and ledger with the 4.2 proof and remaining daemon transport deferral.

## Checkpoints

- `cargo test -p faber-host-macos-arm64` passes.
- A component fixture can call `capability-call(1)` and cause the host to produce a `done` frame for `host:echo`.
- The same fixture can call `capability-call(2)` and cause the host to produce an `E_NO_ROUTE` frame for `pg:query`.
- The CLI can load the fixture component and call its exported route function.

## Gate Plan

This phase is complete when a Wasm component import is proven to wrap into and route through the same frame kernel used by the direct CLI. This phase does not need to define the final Faber WIT world, implement generated compiler output, or add daemon/server transport.
