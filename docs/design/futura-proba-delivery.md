# Delivery: `@ futura` Faber Tests

**Status:** Proposed
**Last updated:** 2026-06-04

---

## Interpreted Problem

Faber can already express test cases with `proba` and async functions with
`@ futura`, but the two surfaces do not meet. A test body that needs `cede`
for live HTTP, database, process, or other async HAL calls has no canonical
source form today.

The motivating case is a Rust blackbox integration test shaped like:

- live configuration gate
- authenticated HTTP client setup
- async POST / GET / DELETE requests
- JSON request and response inspection
- grouped assertions over response rows
- ignored-by-default execution because it requires external service state

Faber has enough language complexity for much of the structure, but an async
test body currently lacks a clear declaration surface and a correct Rust test
harness lowering.

## Normalized Spec

Support `@ futura` on `proba` declarations:

```fab
@ futura
proba "aggregate query and group lifecycle" omitte "requires live API" {
    fixum _ response ← cede http.petet(url)
    adfirma response.bene()
}
```

The annotation means the generated test function is async. Test modifiers such
as `omitte`, `futurum`, `solum`, `tag`, `temporis`, `repete`, `fragilis`,
`requirit`, and `solum_in` remain test-selection or test-metadata modifiers.
They do not acquire async semantics.

Do not add a `futura` or `async` test modifier:

```fab
proba "case" futura { ... }       # not approved
proba "case" async { ... }        # not Faber syntax
```

`futurum "reason"` already means pending/todo. A neighboring `futura` modifier
would be an avoidable footgun.

## Repo-Aware Baseline

Current grammar defines test declarations as:

```ebnf
probandumDecl := 'probandum' STRING probaModifier* '{' probandumBody '}'
probaStmt     := 'proba' STRING probaModifier* blockStmt
```

Current test modifiers are metadata and selection only:

```ebnf
probaModifier := 'omitte' STRING | 'futurum' STRING | 'solum' | 'tag' STRING
              | 'temporis' NUMBER | 'metior' | 'repete' NUMBER | 'fragilis' NUMBER
              | 'requirit' STRING | 'solum_in' STRING
```

`@ futura` is already established on function declarations and HAL pactum
methods. `cede` is parsed as an await/yield unary operator and Rust codegen can
emit `.await` for ordinary async functions and async entrypoints.

Rust test codegen currently emits `#[test]` for Faber tests in
`crates/radix/src/codegen/rust/decl.rs`. A plain Rust `#[test] async fn` is not
a valid Cargo test shape. The implementation must either select an async test
runtime attribute or wrap the async body in a synchronous test function.

## Approved Contract

### Syntax

Annotations may precede `proba` in the same way they precede other executable
declaration-like source items:

```fab
@ futura
proba "async case" {
    fixum _ value ← cede async_value()
    adfirma value ≡ "ok"
}
```

`@ futura` may also combine with existing test modifiers:

```fab
@ futura
proba "live API aggregate lifecycle" omitte "requires SWARM_API_TEST_API_URL" tag "blackbox" {
    ...
}
```

If the grammar grows annotated `probandum`, the annotation must not imply async
for all nested tests unless that inheritance is deliberately specified later.
This delivery only approves `@ futura` directly on `proba`.

### Diagnostics

An async test body must be explicit:

```fab
proba "missing async marker" {
    fixum _ response ← cede http.petet(url)
}
```

The compiler should diagnose this with a repair-oriented message such as:

```text
async test body uses cede but the proba is not marked @ futura
help: add @ futura before proba
```

`@ futura` on unsupported test-like containers should fail closed with a clear
diagnostic rather than being silently ignored.

### Rust Lowering Strategy

Preferred Rust output for async Faber tests:

```rust
#[tokio::test]
async fn proba_1000000() {
    ...
}
```

Rationale:

- live HTTP and database HAL surfaces already tend toward Tokio-backed Rust
  implementations
- `#[tokio::test]` matches the ecosystem shape of the motivating blackbox test
- a homegrown spin-loop executor is not a good default for real network IO

Fallback option, only if the package layer cannot provide Tokio cleanly:

```rust
#[test]
fn proba_1000000() {
    __faber_block_on(async {
        ...
    });
}
```

If this fallback is chosen, it must be documented as a temporary runtime
compromise and must be tested against at least one future that can become
pending before completion.

## Stage Graph

### Stage 1: Parse And AST Surface

- Allow annotations before `proba`.
- Preserve the annotation span on the test syntax node.
- Reject unrecognized annotation placement with normal parser recovery.
- Update `EBNF.md` to describe annotated tests.

Acceptance:

- parser accepts `@ futura proba "case" { ... }`
- parser still accepts ordinary `proba`
- malformed annotation placement reports a parse diagnostic and continues

### Stage 2: HIR Test Metadata

- Lower `@ futura` into `HirFunction.is_async` for generated proba functions,
  or equivalent explicit HIR test metadata if the implementation keeps test
  async separate from ordinary function async.
- Preserve existing `HirTestMetadata` modifiers unchanged.
- Do not reinterpret `futurum` as async.

Acceptance:

- HIR for annotated proba records async intent
- HIR for `proba ... futurum "todo"` remains ignored/pending metadata, not
  async intent

### Stage 3: Semantic Rule For `cede`

- Ensure `cede` is valid inside `@ futura proba`.
- Diagnose `cede` inside non-async `proba`.
- Keep ordinary `@ futura functio` and `incipiet` behavior unchanged.

Acceptance:

- positive async proba with `cede` typechecks
- non-async proba with `cede` fails with the new diagnostic
- non-async proba without `cede` remains valid

### Stage 4: Rust Codegen And Package Harness

- Emit `#[tokio::test] async fn` for async proba, or explicitly choose the
  temporary `__faber_block_on` fallback.
- Ensure generated package Cargo manifests include the required test runtime
  dependency when async tests are present.
- Preserve `#[ignore = "..."]` emission for `omitte`, `futurum`, selection
  filters, and `solum` interactions.

Acceptance:

- generated Rust compiles for sync tests and async tests in the same package
- ignored async tests remain ignored by default
- selected async tests run through `faber test`

### Stage 5: Documentation And Explain Corpus

- Update `EBNF.md`.
- Update `faber explain proba`, `faber explain futura`, and any test modifier
  entries that discuss `futurum`.
- Add an example under `examples/fixtures/exempla-boundary/proba/` or a package
  fixture with an ignored async test.

Acceptance:

- docs distinguish `@ futura` from `futurum "reason"`
- examples use type-first Faber syntax and no invented nullable syntax

## Epic Candidates And Scopable Issues

1. Parser and AST support for annotated `proba`.
2. HIR lowering of async test intent.
3. Semantic diagnostic for `cede` in non-async tests.
4. Rust codegen and package manifest support for async tests.
5. Docs, explain corpus, and fixture coverage.

The first three can land without a real HTTP HAL. The Rust codegen stage is the
point where the runtime strategy must be decided.

## Checkpoints

Checkpoint A: source surface compiles through parse/HIR.

- `@ futura proba "case" { adfirma verum }` appears in HIR as async.
- Ordinary `proba` output is unchanged.

Checkpoint B: semantic contract is enforced.

- `cede` in annotated tests is accepted.
- `cede` in unannotated tests is rejected with a repair diagnostic.

Checkpoint C: Rust package tests run.

- one sync `proba` and one async `@ futura proba` in the same package compile
  and run with `faber test`.
- ignored async tests show up as ignored Cargo tests.

Checkpoint D: docs and examples agree with implementation.

- `EBNF.md`, explain entries, and fixtures all spell `@ futura` before `proba`.
- no docs suggest `proba ... futura`.

## Companion Skill Plan

- Use normal compiler-engineering discipline from
  `docs/compiler-engineering-rules.md`: parser coverage, semantic positive and
  diagnostic cases, codegen output coverage, and package-level e2e tests.
- A later poker-face pass should compare this delivery document against the
  implementation before the feature is called done.

## Gate Plan

Minimum verification:

```bash
cargo test -p radix
cargo test -p faber
./scripta/test
```

If the implementation touches generated package manifests or `faber test`,
include at least one package fixture run:

```bash
cargo run -p faber -- test examples/fixtures/exempla-boundary/proba/packages/passing
```

Add a new async package fixture and run it directly once the feature exists.

## Open Questions

1. Should async test support require Tokio unconditionally for Rust packages, or
   only when an async proba is present?
2. Should `@ futura` ever be allowed on `probandum` as inherited metadata, or
   should async intent remain explicit on each `proba`?
3. Should non-Rust backends reject async `proba`, ignore tests entirely, or grow
   target-specific test harness support?
4. Should `requirit "network"` or a future capability requirement become the
   canonical live-test gate alongside `omitte`, or is `omitte` enough for now?

---
