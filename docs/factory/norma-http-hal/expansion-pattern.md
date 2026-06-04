# Norma HAL Expansion Pattern

This checklist captures the reusable pattern proven by the HTTP HAL factory
work. Use it for future `stdlib/norma/hal/*` modules that need Rust runtime
support and package-level generated Rust coverage.

## 1. Interface

- Keep the Faber source of truth in `stdlib/norma/...`.
- Preserve type-first syntax and check `EBNF.md` before adding new grammar
  shapes.
- Use concrete Faber primitives where the runtime ABI is known. For JSON-like
  dynamic data, prefer `valor` over `quidlibet`.
- Mark unsupported surfaces as deferred in comments instead of implying runtime
  support exists.
- If generated Rust must call methods directly, prefer source method names that
  already match Rust runtime names. Otherwise confirm the bridge normalizes them
  deterministically.

## 2. Rust Runtime Module

- Add the runtime implementation under `crates/norma`.
- Export nested HAL modules from `crates/norma/hal/mod.rs`.
- Keep response/handle types owned where generated packages need to hold them
  across calls.
- Runtime errors at HAL boundaries should return the interface's documented
  failure value instead of panicking.
- Parse or conversion failures should return the documented sentinel value when
  the Faber interface has no failable channel.

## 3. Dependencies

- Add runtime dependencies to `crates/norma/Cargo.toml`.
- Let `Cargo.lock` change through Cargo commands, not by hand.
- If generated package code references a crate directly, add that crate to the
  generated package `Cargo.toml`; transitive dependencies are not enough.

## 4. Rust Codegen Bridge

- Add the module receiver to
  `crates/radix/src/codegen/rust/expr/call/runtime.rs`.
- Keep bridge matching narrow and explicit. Do not turn every unresolved pactum
  into a runtime call.
- For runtime-owned pacta that should not become local Rust traits, record an
  exact HIR shape and elide the generated trait declaration.
- For runtime-owned concrete return values, map the specific HIR interface
  definition to the runtime Rust type in `type_to_rust`.
- Do not guess from missing type information in expression codegen.

## 5. Tests

- Add runtime tests in dedicated `_test.rs` files near the runtime module.
- Add focused Rust codegen tests for bridge output and concrete type rendering.
- Add a package-level fixture when the work crosses package import, generated
  crate emission, Cargo build, and runtime execution.
- Prefer local deterministic servers, files, or fakes over public internet.
- Check both the generated source shape and observable runtime behavior when
  possible.

## 6. Validation Gates

For a typical Rust-backed HAL module, run the closest equivalent of:

```bash
cargo run -p faber -- check stdlib/norma/hal/<module>.fab
cargo test -p norma <module>
cargo test -p radix <module>
cargo test -p faber <fixture-test-name>
cargo check -p norma
```

Before marking a factory complete, run the broad gate named by the plan, usually
`./scripta/test`.

## HTTP Factory Evidence

The HTTP HAL factory proved this pattern through:

- `crates/norma/hal/http.rs` and `crates/norma/hal/http_test.rs`;
- `crates/radix/src/codegen/rust/expr/call/runtime.rs`;
- HTTP runtime interface recognition in `crates/radix/src/codegen/rust/mod.rs`;
- concrete `Replicatio` type rendering in
  `crates/radix/src/codegen/rust/types.rs`;
- the local-server package fixture in `crates/faber/src/package_test.rs`.
