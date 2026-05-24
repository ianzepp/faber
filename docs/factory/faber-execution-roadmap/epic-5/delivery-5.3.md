# Epic 5 Delivery 5.3: Generated Artifact Proof

## Interpreted Problem

The final Epic 5 checkpoint requires proof from generated Faber code, not only
from hand-written Wasm fixtures. Phase 5.2 added the host-side core Wasm runner,
but the generated Rust helper still needed to return a frame-derived value to
Faber code and the proof needed to be repeatable.

## Normalized Spec

- Extend the temporary Wasm import ABI enough for `textus` `ad` results:
  - `capability-call(route-code) -> status`
  - `capability-text-len() -> length`
  - `capability-text-read(ptr, len) -> written`
- Make generated Wasm Rust `__faber_ad::<String, _>(...)` bind the host-returned
  text result instead of a default value.
- Keep native Rust unresolved-provider behavior unchanged.
- Add a repo-owned proof script that:
  - writes Faber source containing `ad "host:echo"` and `ad "pg:query"`,
  - emits Rust through `radix`,
  - appends a tiny export harness,
  - compiles to `wasm32-unknown-unknown`,
  - runs the generated Wasm through `faber-host-macos-arm64 wasm-call`,
  - validates `host:echo` success and `pg:query` `E_NO_ROUTE`.

## Repo-Aware Baseline

- The Homebrew Rust install still lacks Wasm target standard libraries.
- A temporary rustup toolchain under `/tmp/faber-rustup-epic5` provided
  `wasm32-unknown-unknown` for this proof without changing repo configuration
  or shell profile state.
- `scripta/prove-epic5-wasm` is target-agnostic and can be run with
  `WASM_RUSTC=/path/to/rustc` when the default `rustc` lacks the target.

## Stage Graph

1. Tighten generated Rust Wasm helper result handling for `String`.
2. Add matching text-result imports to `WasmHost`.
3. Add `scripta/prove-epic5-wasm`.
4. Run compiler, host, and generated-artifact validation.
5. Update the Epic 5 ledger and completion audit.

## Checkpoints

- Generated Rust helper tests prove the Wasm helper imports and result trait.
- Host tests prove core Wasm and component routing still work.
- The proof script passes with a rustc that has `wasm32-unknown-unknown`.
- The generated Faber Rust artifact reaches `HostKernel` for both `host:echo`
  and `pg:query`.
- `host:echo` returns the frame-derived text value `salve` into generated Faber
  code before the export harness returns success.
- `pg:query` returns structured `E_NO_ROUTE`.

## Gate Plan

Epic 5 can be marked complete once this phase passes together with the native
unresolved-provider sanity check and the earlier host/component tests.
