# Norma HTTP HAL Baseline Ledger

Phase: 0 - Baseline Ledger

## Worktree

Command:

```bash
git status --short --untracked-files=all
```

Result before ledger creation:

```text
?? docs/factory/norma-http-hal/phase-0-delivery.md
```

Only Phase 0 documentation artifacts were present as untracked changes.

## HTTP Interface Check

Command:

```bash
cargo run -p faber -- check stdlib/norma/hal/http.fab
```

Result:

```text
ok: stdlib/norma/hal/http.fab
```

Observation:

- The current HTTP interface parses and checks.
- JSON-related signatures still use `quidlibet`:
  - `http.json(numerus status, quidlibet data) -> Replicatio`
  - `Replicatio.corpusJson() -> quidlibet`
  - `Rogatio.corpusJson() -> quidlibet`
- Response method names are camel-case in the interface:
  - `corpusOcteti`
  - `corpusJson`

## Norma Crate Check

Command:

```bash
cargo check -p norma
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

Observation:

- `crates/norma` checks before adding HTTP runtime support.

## Current Norma HAL Exports

Source: `crates/norma/hal/mod.rs`

```rust
pub mod arca;
pub mod consolum;
pub mod processus;
pub mod solum;
```

Observation:

- There is no `pub mod http;`.
- There is no `crates/norma/hal/http.rs`.

## Current Rust Runtime Bridge

Source: `crates/radix/src/codegen/rust/expr/call/runtime.rs`

Current bridged modules:

```rust
"json" => Some("norma::json"),
"toml" => Some("norma::toml"),
"consolum" => Some("norma::hal::consolum"),
```

Observation:

- There is no `"http" => Some("norma::hal::http")` bridge entry.
- The existing method normalizer converts ASCII camel-case names to snake case,
  which is relevant to `corpusJson` and `corpusOcteti`.

## Package Import Behavior

Temporary import-only package:

```fab
importa ex "norma/hal/http" privata http

incipit {
    nota "http import baseline"
}
```

Command:

```bash
cargo run -p faber -- check /tmp/faber-http-hal-phase0.4WP2yn
```

Result:

```text
ok: /tmp/faber-http-hal-phase0.4WP2yn
```

Command:

```bash
cargo run -p faber -- emit /tmp/faber-http-hal-phase0.4WP2yn
```

Result:

- Emit succeeds.
- Generated Rust includes local traits for `http`, `Replicatio`, `Rogatio`,
  and `Servitor`.
- Generated Rust does not reference `norma::hal::http`.

## Package Call Behavior

Temporary call package:

```fab
importa ex "norma/hal/http" privata http

incipiet {
    fixum _ responsum ← cede http.petet("http://127.0.0.1:9")
    nota responsum.status()
}
```

Command:

```bash
cargo run -p faber -- check /tmp/faber-http-hal-phase0-call.1
```

Result:

```text
ok: /tmp/faber-http-hal-phase0-call.1
```

Command:

```bash
cargo run -p faber -- emit /tmp/faber-http-hal-phase0-call.1
```

Result excerpt:

```rust
let responsum: dyn Replicatio = http.petet("http://127.0.0.1:9".to_string()).await;
```

Command:

```bash
cargo run -p faber -- build /tmp/faber-http-hal-phase0-call.1
```

Result:

```text
error[E0423]: expected value, found trait `http`
error[E0277]: the size for values of type `dyn Replicatio` cannot be known at compilation time
```

Observation:

- Package import resolution can find `norma/hal/http`.
- A real HTTP call still emits against an generated trait-shaped surface, not
  the Norma runtime.
- Concrete `Replicatio` type mapping is not implemented; generated Rust uses
  unsized `dyn Replicatio`.

## Baseline Conclusion

Phase 0 confirms the plan's initial claim:

- the language-facing HTTP interface exists and checks;
- the Rust runtime module is missing;
- the Rust backend bridge is missing;
- import-only packages work;
- packages that call `http.petet` do not build yet because generated Rust lacks
  both the `norma::hal::http` bridge and a concrete runtime response type.
