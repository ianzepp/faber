# Library Import Provenance Ledger

## Phase 0 Baseline

Date: 2026-06-04

### Worktree

`git status --short` returned no entries before Phase 0 artifacts were written.

### Grammar State

`EBNF.md` already shows provider-qualified Norma syntax in the import examples:

```fab
importa ex "norma:hal/consolum" privata consolum
```

The grammar treats the import source as a string literal and does not itself
distinguish provider imports from local path imports.

### Active Slash-Form Tests

`crates/faber/src/package_test.rs` still has active package tests using built-in
Norma slash specifiers:

- `importa ex "norma/json" privata json`
- `importa ex "norma/toml" privata toml`
- `importa ex "norma/hal/consolum" privata consolum`
- `importa ex "norma/hal/http" privata http`

Those tests currently expect compile success and Rust output containing
`norma::...` runtime calls.

### Current Faber Library Resolver

`crates/faber/src/library.rs` currently resolves library imports by splitting
the source specifier on `/` and claiming any specifier whose first segment is
`norma`.

Current behavior:

- `norma/json` resolves to `stdlib/norma/json.fab`;
- `norma/toml` resolves to `stdlib/norma/toml.fab`;
- `norma/hal/http` resolves to `stdlib/norma/hal/http.fab`;
- unknown first path segments return `Ok(None)` and fall through to package
  import handling.

The resolver does not yet parse provider-qualified `provider:module/path`
syntax and has no targeted hard-cut diagnostic for old built-in slash forms.

### Current Rust Runtime Call Bridge

`crates/radix/src/codegen/rust/expr/call/runtime.rs` contains:

```rust
pub(super) fn norma_runtime_module_path(receiver_name: &str) -> Option<&'static str>
```

It maps local receiver names directly:

- `json` -> `norma::json`
- `toml` -> `norma::toml`
- `consolum` -> `norma::hal::consolum`
- `http` -> `norma::hal::http`

`crates/radix/src/codegen/rust/expr/call/mod.rs` calls this helper after
resolving the receiver `DefId` back to its local name. This means a user-defined
binding named `http`, `json`, `toml`, or `consolum` can still be considered for
Norma runtime lowering by spelling alone.

### Current HTTP Runtime Type Mapping

`crates/radix/src/codegen/rust/mod.rs` recognizes HTTP runtime interfaces in
`http_runtime_interface_info` by interface name and method-list matching.

Current recognized names include:

- `http`
- `Replicatio`
- `Rogatio`
- `Servitor`

This proves the Phase 4 risk is still present: a user interface with the same
name and matching method shape can be mapped as a Norma HTTP runtime type
without imported library item provenance.

### Focused Existing Test

Command:

```bash
cargo test -p radix http_hal_calls_emit_norma_runtime_bridge_and_concrete_response_type
```

Result: passed.

The test constructs `pactum http`, `pactum Replicatio`, `pactum Rogatio`, and
`pactum Servitor` directly in source and currently expects emitted Rust to use
`norma::hal::http::petet` and `norma::hal::http::Replicatio`. That is useful
baseline evidence for the unsafe name/shape bridge that later phases must
replace.
