# Norma HTTP HAL Factory Plan

**Status**: design captured, not started
**Created**: 2026-06-04
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/norma-http-hal/`
**Mode**: stdlib runtime expansion / Rust HAL implementation / compiler bridge
**Commit Policy**: Commit after each completed phase and validation gate pass
**Related Plan**: [`../../design/futura-proba-delivery.md`](../../design/futura-proba-delivery.md)

## Interpreted Problem

Faber has a language-facing HTTP interface in `stdlib/norma/hal/http.fab`, but
there is no matching Rust runtime implementation in `crates/norma/hal/http.rs`
and no Rust backend bridge for imported `norma/hal/http` calls.

That makes HTTP a good first serious Norma HAL expansion:

- the interface is constrained and familiar,
- the Rust implementation can mostly proxy to an existing Rust HTTP client,
- the work exercises async runtime behavior,
- the result unlocks useful package tests and blackbox integration tests,
- the pattern should generalize to future Norma modules.

The goal is not to design a general networking framework. The goal is to make
the existing Faber HTTP pactum real for the Rust target, with enough compiler
and package plumbing that a Faber package can import `norma/hal/http`, issue
requests, and inspect responses.

## Current Baseline

Existing language-facing interface:

- `stdlib/norma/hal/http.fab`
  - declares `http.petet`, `mittet`, `ponet`, `delet`, `mutabit`, `rogabit`
  - declares server-side `exspectabit`
  - declares `Replicatio`, `Rogatio`, and `Servitor` pacta

Existing Rust runtime shape:

- `crates/norma/hal/mod.rs` exports `arca`, `consolum`, `processus`, and
  `solum`; it does not export `http`.
- `crates/norma/Cargo.toml` already has Tokio, but not an HTTP client crate.
- `crates/norma/datum.rs` provides the canonical `norma::datum::Valor` value
  type for JSON-like dynamic data.
- `crates/norma/json.rs` can convert through `Valor`, and `valor` is already a
  Faber primitive mapped to `norma::datum::Valor` in Rust codegen.

Existing compiler and package shape:

- built-in library import resolution can discover `norma/hal/http` from
  `stdlib/norma/hal/http.fab`.
- generated Rust packages depend on the local `norma` crate.
- Rust codegen has a small Norma runtime bridge for selected module bindings in
  `crates/radix/src/codegen/rust/expr/call/runtime.rs`, currently including
  `json`, `toml`, and `consolum`, but not `http`.
- pactum return types such as `Replicatio` need a concrete Rust type mapping
  strategy before generated code can reliably type annotations against the
  runtime struct.

## Non-Goals

- Do not implement HTTP server support in the first slice.
- Do not implement a general async test feature here; that is tracked by the
  `@ futura proba` delivery plan.
- Do not replace `ad` capability calls with HTTP library calls. They are
  different surfaces.
- Do not introduce a broad package manager or registry requirement.
- Do not make `quidlibet` the HTTP JSON ABI. Prefer `valor`.
- Do not hide missing type information in codegen with guesses. If pactum
  return types do not map cleanly, fix the bridge or type model explicitly.

## Runtime Contract

The first shipped Rust HTTP HAL should implement the client side:

```fab
importa ex "norma/hal/http" privata http

@ futura
functio lege() → Replicatio {
    redde cede http.petet("https://example.com")
}
```

The runtime implementation should provide:

- `petet(url)` for GET
- `mittet(url, corpus)` for POST with text body
- `ponet(url, corpus)` for PUT with text body
- `delet(url)` for DELETE
- `mutabit(url, corpus)` for PATCH
- `rogabit(modus, url, capita, corpus)` for explicit method, headers, and body

`Replicatio` should be a concrete Rust struct in `norma::hal::http` with:

- `status() -> i64`
- `corpus() -> String`
- `corpus_octeti() -> Vec<u8>`
- `corpus_json() -> norma::datum::Valor`
- `capita() -> HashMap<String, String>`
- `caput(nomen) -> Option<String>`
- `bene() -> bool`

The initial Faber interface should change `corpusJson()` from `quidlibet` to
`valor`, and server response builder `json(status, data)` should use `valor`
if it remains in the interface during this factory pass.

## Error Contract

The existing HTTP pactum returns `Replicatio` directly rather than an alternate
exit channel. For the first implementation, runtime transport failures should
produce a deterministic synthetic response rather than panic:

- status `0`
- empty body or a diagnostic body
- an implementation-owned header such as `x-faber-error`
- `bene() == falsum`

This keeps the current interface usable without inventing failable HTTP syntax
mid-slice. A later phase may introduce failable variants or alternate-exit
signatures if the language standard library chooses that direction.

JSON parse failures in `corpus_json()` should return `Valor::Nihil` rather than
panic. Callers can inspect raw `corpus()` when they need diagnostics.

## Dependency Decision

Use `reqwest` for the Rust client implementation unless implementation evidence
shows it causes an unacceptable dependency or runtime conflict.

Required Norma dependency shape is expected to be:

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
tokio = { version = "1", features = ["fs", "io-std", "io-util", "process", "rt"] }
```

If `#[tokio::test]` or generated async test support later requires more Tokio
features, that belongs to the async test phase, not this runtime module, unless
runtime tests in `crates/norma` require it.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Baseline Ledger | Record current HTTP interface, missing runtime module, bridge state, and package behavior. | Ledger captures truth before edits. |
| 1 | Interface Tightening | Normalize `http.fab` around client-only first slice and `valor` JSON response values. | `faber check stdlib/norma/hal/http.fab` succeeds. |
| 2 | Norma Runtime Client | Add `crates/norma/hal/http.rs` and export it. | `cargo test -p norma http` proves request/response helpers with local mocked data or direct unit construction. |
| 3 | Rust Codegen Bridge | Teach Rust backend that imported `http` module calls lower to `norma::hal::http::*`, and map concrete HTTP types. | Generated Rust for `http.petet` uses `norma::hal::http::petet` and compiles. |
| 4 | Package Fixture | Add a repeatable Faber package fixture proving import, request, response status/body/header access, and JSON value access without public internet dependency. | `faber test` or `faber run` fixture passes locally. |
| 5 | Docs And Expansion Pattern | Document the Norma HAL implementation pattern for future modules. | Factory plan and docs identify the reusable interface/runtime/bridge/fixture checklist. |
| 6 | Validation Gate | Run focused and broad checks. | Required Cargo, Faber, and fixture commands pass. |

## Phase Details

### Phase 0: Baseline Ledger

Create `docs/factory/norma-http-hal/ledger.md` with:

- `git status --short`
- `faber check stdlib/norma/hal/http.fab`
- `cargo check -p norma`
- current `crates/norma/hal/mod.rs`
- current Rust Norma bridge table
- current generated package behavior for a package importing `norma/hal/http`

No source behavior changes in this phase.

### Phase 1: Interface Tightening

Steps:

- Change JSON-related HTTP signatures from `quidlibet` to `valor`.
- Decide whether server-side `exspectabit`, `Rogatio`, and `Servitor` remain in
  the file as deferred declarations or move behind a clear future section.
- Ensure method names in `.fab` match Rust snake_case bridge behavior:
  `corpusJson` -> `corpus_json`, `corpusOcteti` -> `corpus_octeti` after bridge
  normalization.
- Keep Faber source type-first and grammar-compliant.

Checkpoint:

- `cargo run -p faber -- check stdlib/norma/hal/http.fab` succeeds.
- JSON response values are `valor` in the interface.

### Phase 2: Norma Runtime Client

Steps:

- Add `crates/norma/hal/http.rs`.
- Add `pub mod http;` to `crates/norma/hal/mod.rs`.
- Add the chosen HTTP client dependency to `crates/norma/Cargo.toml`.
- Implement `Replicatio` as an owned response snapshot:
  status, headers, body bytes.
- Implement methods from the Faber interface using owned return values.
- Implement `corpus_json()` through `serde_json::Value` -> `Valor`.
- Avoid panics on network and parse failures.

Checkpoint:

- `cargo test -p norma http` passes.
- `cargo check -p norma` passes.

### Phase 3: Rust Codegen Bridge

Steps:

- Add `"http" => Some("norma::hal::http")` to the Rust Norma runtime module
  bridge.
- Add tests proving `http.petet(url)` emits `norma::hal::http::petet(url)`.
- Resolve concrete return type handling for `Replicatio`. Acceptable strategies:
  - map imported built-in pactum names to runtime concrete types,
  - avoid explicit local type annotation in generated fixture until type mapping
    is implemented, then add the mapping as a separate focused step,
  - introduce a targeted Rust type-renderer rule for known Norma HAL types.
- Do not let codegen guess unknown pactum types as `dyn Replicatio` when a
  concrete runtime struct is required.

Checkpoint:

- a focused Rust codegen test proves the bridge output shape.
- generated Rust that awaits `http.petet` compiles.

### Phase 4: Package Fixture

Prefer a local deterministic fixture over public internet. Options:

1. a Rust unit/integration test that spins a local HTTP listener and invokes a
   generated Faber package,
2. a Faber package that targets a local server supplied by the test harness,
3. a narrow runtime-only Norma test for live requests plus a package compile
   test for Faber import/codegen.

The first complete package fixture should prove:

- import resolution for `norma/hal/http`
- async request call through `cede`
- `status()`
- `bene()`
- `corpus()`
- `caput(...)`
- `corpusJson()` returning `valor`

Checkpoint:

- fixture runs repeatably without public network access.

### Phase 5: Docs And Expansion Pattern

Document the reusable Norma HAL module checklist:

1. `.fab` interface in `stdlib/norma/...`
2. Rust implementation in `crates/norma/...`
3. export from `crates/norma/lib.rs` or nested `mod.rs`
4. dependency declaration in `crates/norma/Cargo.toml`
5. Rust backend runtime bridge entry
6. concrete type mapping when pactum returns are runtime structs
7. runtime unit tests
8. package-level fixture
9. explain/docs update when the interface is user-facing

Checkpoint:

- future HAL work can start from the checklist without rediscovering HTTP
  bridge mechanics.

### Phase 6: Validation Gate

Minimum validation:

```bash
cargo test -p norma
cargo test -p radix
cargo test -p faber
cargo run -p faber -- check stdlib/norma/hal/http.fab
```

If a package fixture exists:

```bash
cargo run -p faber -- test <fixture-path>
```

Broad gate before marking the factory complete:

```bash
./scripta/test
```

## Acceptance Criteria

- Faber can import `norma/hal/http` in a package.
- Rust codegen emits calls to `norma::hal::http::*` for HTTP module calls.
- `crates/norma` implements client-side HTTP without panics on normal network
  failures.
- Response inspection works for status, headers, text body, byte body, success
  predicate, and JSON-as-`valor`.
- At least one repeatable test proves the compiler/runtime bridge, not only the
  direct Rust helper functions.
- Server-side HTTP remains explicitly deferred unless implemented and tested.

## Open Questions

1. Should transport failures be synthetic `Replicatio` values forever, or should
   HTTP later grow `⇥ textus` failable signatures?
2. Should `Replicatio` be a `pactum` in Faber source long term, or should Norma
   support an external concrete `genus` contract for runtime structs?
3. Should `rogabit` accept `tabula<textus, textus>` only, or should headers be a
   richer case-insensitive type?
4. Should request bodies stay `textus` for the first slice, with byte and JSON
   convenience helpers later?
5. How should this interact with `@ futura proba` once async tests are
   implemented?

## Stop Conditions

- Stop if generated Rust needs to guess a concrete type for an unresolved
  pactum return.
- Stop if HTTP calls can only be tested through public internet.
- Stop if implementing server support becomes necessary for the client slice;
  split it into a later phase.
- Stop if `quidlibet` starts leaking into the JSON response ABI instead of
  using `valor`.
- Stop if runtime errors would panic for ordinary network failures.

---
