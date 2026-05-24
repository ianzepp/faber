# Goal: Executable Rust Coverage For Exempla

**Status**: Epic 2 closeout reached; Rust e2e gate is truthful with one Epic 3 expected failure
**Created**: 2026-05-24
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/exempla-rust-e2e/`
**Mode**: compiler correctness and exemplar corpus cleanup
**Commit Policy**: Commit after each completed phase and validation gate pass
**Coordinating Roadmap**: [`../faber-execution-roadmap/goal.md`](../faber-execution-roadmap/goal.md)

## Summary

Make the `examples/exempla/` corpus truthful against the Rust target: every remaining exemplar is a single-file language example that compiles to executable Rust and runs successfully through the standalone end-to-end harness, or the exemplar is corrected, moved out of the exempla boundary, or removed because it is no longer valid Faber.

## Current State

- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` now reports `99/100` files passing.
- The only expected Rust e2e failure is `examples/exempla/ad/ad.fab`, which remains linked to Epic 3 capability-call work.
- `examples/exempla/` currently contains `100` `.fab` files.
- `examples/fixtures/exempla-boundary/` preserves `37` relocated `.fab` fixtures that are not standalone Rust exempla.
- The Rust e2e harness now fails on any unexpected exemplar failure and also fails if `ad/ad.fab` starts passing without removing the expected-failure metadata.

## Original Problem

- The ignored Rust e2e harness currently reports `71/138` exemplar files passing end-to-end and `67` failing.
- The failures are not one bug. They mix source files that are stale or intentionally invalid, Rust backend semantic gaps, unsupported target features, package examples, dependency-backed examples, and files that should not live in a single-file language-example corpus.
- The harness already exercises the generated-code format/linter path before `rustc`, so future fixes must preserve that coverage and avoid hiding formatter or linter regressions.
- The corpus currently blurs several categories: single-file language examples, package examples, test fixtures, dependency-backed examples, library/helper files, intentionally failing cases, and language sketch files.

## Desired End State

- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports all `examples/exempla/**/*.fab` files passing as standalone Rust executables.
- Every `.fab` file under `examples/exempla/` is intentionally compilable as a single Rust file. The e2e harness should not need package/module assembly to make exempla pass.
- Files that need package structure, test-package selection, external crates, host/runtime dependencies, or multi-file module assembly are rewritten to avoid those dependencies, moved to a sibling examples tree such as `examples/automation`, or removed.
- When an exemplar fails, the failure points to a real compiler/runtime/source bug rather than harness mismatch, stale syntax, or accidental corpus drift.

## Ground Truth Researched

- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`: the ignored Rust e2e harness passes as a Rust test but reports `71/138` exemplar files passing and `67` failing internally.
- `/tmp/faber-exempla-rust-e2e.log`: captured the full e2e run and the `[fail]` records used for this taxonomy.
- `crates/radix/src/exempla_e2e_test.rs`: the Rust e2e harness collects every `.fab` file under `examples/exempla`, compiles it with the default Rust target, runs generated code through `format_generated_code` and best-effort `lint_generated_code`, then invokes standalone `rustc` and executes the resulting binary.
- `crates/radix/src/tool.rs`: `--format` and `--linter` are implemented as generated-code post-processing helpers; the e2e harness uses those helpers directly rather than invoking the CLI flags.
- `AGENTS.md`: Rust-only tooling, no invented syntax, no papering over missing type information, and correctness over completion are project constraints.
- `docs/factory/remove-ab-dsl/goal.md`: `ab` collection DSL removal is tracked as its own language simplification goal and should not be treated as an ordinary Rust backend repair.
- `docs/factory/capability-calls/goal.md`: `ad` capability-call policy is tracked as its own language/runtime-boundary goal and should not be treated as an ordinary Rust backend repair.

## Failure Taxonomy

### A. Corpus Boundary Mismatch

These failures look partly or wholly caused by files living inside `examples/exempla/` even though they are not single-file language examples. Epic 2 should not rebuild the Rust e2e harness around these cases. It should either rewrite them to become standalone exempla, move them to sibling package/example trees, or remove/quarantine them as non-exempla fixtures.

- `cli/main.fab`: generated Rust imports `crate::commands`; this is package/module structure and belongs in a sibling package example tree, not `examples/exempla/`.
- `importa/importa.fab`, `importa/auxilia.fab`: demonstrate imports/helper-library shape; rewrite as standalone language examples or move out of exempla.
- `hal/aleator.fab`, `hal/consolum.fab`, `hal/json.fab`, `hal/processus.fab`, `hal/yaml.fab`: depend on `norma`/HAL runtime surfaces; rewrite to avoid runtime dependencies or move to host/norma/package examples.
- `expressionis/expressionis.fab`: generated Rust references `regex`; rewrite to avoid external crates or move out of exempla.
- `proba/proba.fab`, `proba/modificatores.fab`: generated Rust contains test-like code without a standalone `main`; move to test/package fixtures or rewrite as executable language examples.
- Package examples under `examples/exempla/proba/packages/*` have real `faber.toml` package structure and should move to a sibling package-example tree.

### B. Stale, Invalid, Or Non-Executable Exemplar Sources

These should be corrected to canonical Faber, moved out of the executable corpus, or removed. Do not teach the compiler to accept retired syntax merely to make these pass.

- `tempta/tempta.fab`, `tempta/in-functione.fab`, `iace/iace.fab`: use retired `tempta`; diagnostics say to use `fac { ... } cape err { ... }`.
- `literalis/literalis.fab`, `proba/packages/selection-failure/src/main.fab`: compile fails at unknown identifiers.
- `cli/commands/greet.fab`: compile fails with expression type mismatch and non-boolean condition.
- `qua/qua.fab`: generated Rust contains a placeholder type hole (`let len: /* error */ = ...`), indicating either source invalidity or an upstream diagnostic/codegen boundary bug.
- `annotatio/annotatio.fab`, `annotatio/grammatica-nova.fab`, `externa/externa.fab`: emit free Rust function declarations without bodies; decide whether these are declaration-only examples, FFI examples needing target support, or stale executable exemplars.

### C. Capability Calls And Unsupported Rust Target Features

These fail before or at codegen because the Rust target does not support the represented Faber feature yet.

- `ad/ad.fab`: `ad` capability calls are not supported for Rust codegen today. The policy and implementation goal for permissive unresolved capability calls lives in `docs/factory/capability-calls/goal.md`.
- `fac/cape.fab`, `fac/fac.fab`, `functio/exitus.fab`, `custodi/validatio.fab`: `cape` and/or `iace` are not supported for Rust targets.
- Any fix must choose between implementing the feature for Rust, marking the exemplar as non-Rust-executable, or removing/correcting the source if it is no longer part of the active language.

### D. Rust Backend Semantic Lowering Gaps

These appear to be real backend/lowering defects where Faber source compiles but generated Rust is not valid or not executable.

- Option/nullability and `ignotum`/dynamic value lowering: `optionalis/optionalis.fab`, `functio/optionalis.fab`, `ternarius/ternarius.fab`, `si/est.fab`, `assignatio/assignatio.fab`, `conversio/conversio.fab`, `innatum/innatum.fab`, `membrum/membrum.fab`, `mori/mori.fab`, `objectum/objectum.fab`, `redde/redde.fab`.
- Enum, variant, match, and `finge` construction: `discerne/discerne.fab`, `finge/finge.fab`, `ordo/ordo.fab`, `elige/*.fab`.
- Struct construction, methods, `ego`, and receiver lowering: `genus/creo.fab`, `genus/methodi.fab`, `pactum/pactum.fab`, `vocatio/vocatio.fab`.
- Iteration and range lowering: `itera/cursor-iteratio.fab`, `itera/intervallum.fab`, `itera/intervallum-gradus.fab`, `itera/nidificatus.fab`, `itera/de.fab`, `itera/in-functione.fab`.
- Ownership and borrowing in generated Rust: `destructura/destructura.fab`, `varia/destructura.fab`, `vel/vel.fab`, repeated vector/map reuse cases.
- Primitive conversion and mixed numeric operation lowering: `functio/typicus.fab`, `conversio/conversio.fab`, `praefixum/praefixum.fab`.
- Stdlib method translation gaps: `incipiet/incipiet.fab` (`longitudo` on `String`), `lista/lista.fab` type annotation needs, and related collection method examples.

### E. Linter/Formatter Interaction Noise

The harness currently applies formatter and best-effort linter before `rustc`. Linter failures are noisy because `lint_generated_code` builds temporary Cargo projects and prints clippy/rustc output even when the harness later falls back to unlinted code. That noise does not appear to be counted as e2e failure unless final `rustc` or execution fails, but it makes diagnosis harder.

## Failed Files From Baseline

```text
ab/ab.fab
ad/ad.fab
annotatio/annotatio.fab
annotatio/grammatica-nova.fab
assignatio/assignatio.fab
clausa/clausa.fab
cli/commands/greet.fab
cli/main.fab
conversio/conversio.fab
custodi/validatio.fab
destructura/destructura.fab
destructura/objectum.fab
discerne/discerne.fab
elige/ceterum.fab
elige/elige.fab
elige/ergo-redde.fab
elige/in-functione.fab
expressionis/expressionis.fab
externa/externa.fab
fac/cape.fab
fac/fac.fab
finge/finge.fab
functio/exitus.fab
functio/optionalis.fab
functio/typicus.fab
genus/creo.fab
genus/methodi.fab
hal/aleator.fab
hal/consolum.fab
hal/json.fab
hal/processus.fab
hal/yaml.fab
iace/iace.fab
importa/auxilia.fab
importa/importa.fab
incipiet/incipiet.fab
innatum/innatum.fab
inter/inter.fab
itera/cursor-iteratio.fab
itera/de.fab
itera/in-functione.fab
itera/intervallum-gradus.fab
itera/intervallum.fab
itera/nidificatus.fab
lista/lista.fab
literalis/literalis.fab
membrum/membrum.fab
mori/mori.fab
morphologia/morphologia.fab
objectum/objectum.fab
optionalis/optionalis.fab
ordo/ordo.fab
pactum/pactum.fab
praefixum/praefixum.fab
proba/modificatores.fab
proba/packages/selection-failure/src/main.fab
proba/proba.fab
qua/qua.fab
redde/redde.fab
si/ergo-redde.fab
si/est.fab
tempta/in-functione.fab
tempta/tempta.fab
ternarius/ternarius.fab
varia/destructura.fab
vel/vel.fab
vocatio/vocatio.fab
```

## Goals

- Establish an explicit disposition for every `examples/exempla/**/*.fab` file: keep as a standalone executable Rust language example, rewrite into that shape, move to a sibling example/fixture tree, or remove as stale/invalid.
- Keep the Rust e2e harness centered on the `examples/exempla/` corpus as standalone single-file Rust validation, not package/module/dependency assembly.
- Correct stale exempla to current grammar and active language semantics, especially retired `tempta` and non-canonical constructs.
- Fix Rust backend/codegen/type-lowering issues that prevent valid exempla from becoming executable Rust.
- Preserve generated-code formatter and linter coverage in the e2e path, and make their diagnostics useful rather than misleading.
- End with a corpus where failures are actionable and rare: a failing executable exemplar means either the source is invalid or the compiler/runtime is wrong.

## Non-Goals

- Do not add compatibility support for retired syntax solely to keep old exempla alive.
- Do not guess missing type information in Rust codegen; repair the semantic/typecheck/lowering source of truth.
- Do not lower all Rust output through MIR as part of this goal unless a focused phase explicitly proves it is the smallest correct fix for a failure class.
- Do not implement unrelated language features just because an old exemplar mentions them; decide whether the exemplar still belongs.
- Do not remove failing exempla as a shortcut before classifying whether they are compiler obligations, stale examples, or non-executable fixtures.
- Do not rebuild the `examples/exempla/` Rust e2e path into package-aware or dependency-aware infrastructure; non-standalone examples belong outside this corpus.

## Constraints And Invariants

- Faber syntax remains type-first: `textus name`, not `name: textus`.
- Empty collections need explicit declared types with `vacua`.
- `cum` remains banned.
- Nullable value types use `T ∪ nihil`; `ignotum` is not a nullability substitute.
- The stdlib source of truth remains `stdlib/norma`; runtime-backed Rust support remains `crates/norma`.
- Use Cargo and `scripta/` helpers, not Bun or Node.
- Codegen must fail closed on unsupported HIR instead of emitting placeholder Rust such as `/* error */`.
- The e2e command should keep exercising generated `--format` and `--linter` behavior for Rust.
- `examples/exempla/` is a language-example corpus. Every remaining file in it must compile as one Faber source file lowered to one standalone executable Rust file.
- Files that require `faber.toml`, sibling modules, external crates, host providers, runtime package layout, or generated test harnesses must be rewritten to avoid that requirement, moved to a sibling tree such as `examples/automation`, or removed.

## Reference Packet

Before editing, inspect:

- `crates/radix/src/exempla_e2e_test.rs`: current corpus discovery, format/linter post-processing, standalone `rustc` validation, and failure reporting.
- `crates/radix/src/tool.rs`: formatter/linter helper behavior and CLI flag semantics.
- `crates/radix/src/driver/mod.rs`: compile pipeline and target-specific diagnostics.
- `crates/radix/src/codegen/rust/`: Rust backend lowering and emitted source shape.
- `crates/radix/src/semantic/passes/`: typecheck, borrow, lint, and target-specific validation boundaries.
- `stdlib/norma/` and `crates/norma/`: stdlib metadata and Rust runtime dependencies used by HAL examples.
- `examples/exempla/`: corpus classification and source corrections.
- `EBNF.md`: canonical syntax before changing any exemplar.
- `/tmp/faber-exempla-rust-e2e.log`: baseline failure evidence from 2026-05-24.
- `docs/factory/remove-ab-dsl/goal.md`: linked goal for retiring the `ab` collection DSL and migrating its exempla.
- `docs/factory/capability-calls/goal.md`: linked goal for `ad` capability-call semantics and unresolved-provider runtime behavior.

## Supporting Skills

- `warmup`: use before implementation if the active agent is not already oriented in this repo.
- `delivery`: use to lower each phase into a concrete implementation plan before edits.
- `factory`: use as the outer loop if this becomes an autonomous multi-phase repair campaign.
- `poker-face`: use after each phase to verify the phase actually reduces the intended failure class and does not merely move failures around.
- `zombie-docs`: use when updating or removing exempla requires documentation and explain corpus consistency.

## Implementation Shape

### Phase 0: Baseline Ledger And Classification

Create a durable ledger under `docs/factory/exempla-rust-e2e/` that records every exemplar, current pass/fail state, expected disposition, and first failure reason. Classify before fixing. The checkpoint is a reviewed table where each failure has an owner category: keep and fix, rewrite into standalone shape, move out of `examples/exempla/`, remove stale/invalid source, unsupported Rust feature, backend semantic bug, or linter/formatter noise.

### Phase 1: Corpus Boundary Enforcement

Apply the hard `examples/exempla/` boundary: it contains only standalone single-file language examples. Move package examples, helper modules, generated test crates, dependency-backed examples, runtime/host examples, and intentionally invalid fixtures to sibling locations or rewrite them into standalone examples. This phase should not claim language correctness wins; its job is to make the corpus honest. The checkpoint is that there are no false standalone-`rustc` failures caused by package shape, external dependencies, helper-only files, or test/package fixtures inside `examples/exempla/`.

### Phase 2: Source Corpus Canonicalization

Correct or retire exempla that are stale, invalid, or no longer canonical. Replace retired `tempta` examples with current `fac { ... } cape err { ... }` only if the Rust target is meant to support that behavior; otherwise move them to a non-Rust or future-feature area. Move or quarantine intentionally failing package fixtures outside `examples/exempla/`. The checkpoint is that source-invalid failures no longer pollute Rust backend validation.

### Phase 3: Dependency-Free Rewrite Or Relocation

Resolve exempla that currently need `norma`, `regex`, imports, package module trees, CLI commands, host/runtime providers, or generated test harnesses without expanding the exempla harness. Rewrite them into dependency-free single-file language examples where that preserves the lesson; otherwise move them to sibling package, host, norma, or fixture trees. The checkpoint is that HAL, regex, import, CLI, and proba examples no longer require package/dependency/runtime context from inside `examples/exempla/`.

### Phase 4: Core Rust Backend Semantics

Fix valid-source backend failures in focused slices: option/null lowering, dynamic/`ignotum` representation, map/object literals, typed empty collections, mixed numeric conversions, string/stdlib method translations, enum/variant construction, `elige`/`discerne`, struct construction, methods/receivers, and ownership/borrowing of moved values. The checkpoint is a measurable reduction in backend-generated `rustc` errors without weakening type safety or guessing in codegen.

### Phase 5: Iteration, Ranges, And Collection Semantics

Handle `itera` over ranges, stepped ranges, maps, cursors, nested iteration, and ordinary collection library behavior as a focused backend correctness phase. Do not repair the `ab` DSL here; `ab` removal and example migration belong to `docs/factory/remove-ab-dsl/goal.md`. The checkpoint is that all valid `itera/*` and collection-library exempla either compile/run or expose a smaller semantic blocker recorded for a later phase.

### Phase 6: Effects, Alternate Exit, And Unsupported Rust Features

Decide and implement the Rust target policy for `iace`, `cape`, `fac`, declaration-only `externa`/annotation examples, and other currently unsupported surfaces. Do not design capability calls in this phase; `ad` policy and implementation belong to `docs/factory/capability-calls/goal.md`. This phase may split into separate delivery plans if the remaining semantic design is still unsettled. The checkpoint is that no executable Rust exemplar fails only because the target policy is ambiguous.

### Phase 7: Formatter/Linter Signal Hardening

Keep the generated-code `--format --linter` path in the e2e run, but make failures distinguishable: formatter failure, linter fix failure, final `rustc` failure, and runtime failure should be reported separately. Suppress or capture temporary clippy noise so the final `[fail]` summary remains the primary truth. The checkpoint is a readable e2e report that helps fix the next failure instead of hiding it in tool output.

### Phase 8: Closeout And Drift Prevention

Run the full ignored Rust e2e harness and regular test suite. Update docs or explain entries that mention removed or moved exempla. Add a guard or review rule that prevents non-standalone package/dependency/runtime fixtures from silently entering `examples/exempla/`. The checkpoint is `0` unexpected Rust e2e failures and a clear rule for future examples.

## Acceptance Criteria

- The baseline ledger exists and accounts for all `138` files from the original corpus.
- Every remaining `.fab` file under `examples/exempla/` compiles, links, and runs through the standalone Rust e2e harness as a single-file language example.
- Every file moved out of `examples/exempla/` or removed has an explicit reason: library helper, package-only, test-only, external dependency, host/runtime dependency, intentionally invalid, future feature, non-Rust target, or stale source.
- `examples/exempla/ab/ab.fab` is resolved through the linked `ab` removal goal: migrated to ordinary collection calls, moved to a legacy/negative fixture, or removed.
- `examples/exempla/ad/ad.fab` is resolved through the linked capability-call goal: compiled with expected unresolved-provider runtime failure, migrated, moved, or otherwise classified explicitly.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports no unexpected exemplar failures.
- The e2e path still applies generated-code formatting and linter logic before final Rust validation.
- Source corrections obey `EBNF.md` and the project grammar rules.
- No fix introduces placeholder codegen or hides missing semantic type information.

## Validation

- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` should pass with all expected Rust-executable exemplars succeeding.
- `./scripta/test` should pass after implementation phases that affect compiler behavior.
- `./scripta/lint` should pass after codegen, diagnostics, or tooling changes.
- `cargo run -p faber -- check examples/exempla/salve-munde.fab` should remain successful.
- `cargo run -p faber -- emit -t rust --format --linter examples/exempla/salve-munde.fab` should still exercise the public CLI flag path.
- Manual review should verify that moved or removed exempla still leave the docs and explain corpus truthful.

## Open Questions

- Which sibling tree should receive package, runtime, host, and test-fixture examples: existing trees such as `examples/automation`, new focused siblings, or compiler test fixtures under `crates/radix`?
- Should declaration-only examples such as `externa` and annotation grammars become compile-only checks, executable examples with stubs, or non-executable reference examples?
- Should intentionally failing examples remain in the repo as negative fixtures, and if so, where should they live outside `examples/exempla/` so positive e2e corpus discovery does not collect them?

## Stop Conditions

- Stop before deleting an exemplar if classification cannot prove it is stale, intentionally invalid, or out of scope.
- Stop before adding compatibility for retired syntax such as `tempta`; source correction is preferred unless the language policy changes.
- Stop if a backend fix requires guessing missing type information in codegen.
- Stop if a proposed fix expands the exempla e2e harness into package/module/dependency execution instead of rewriting, moving, or removing the non-standalone file.
- Stop if validating a package/runtime example would require network access or non-repo dependencies that are not already part of the workspace contract.
- Stop if the work implies a larger semantic decision for `iace`, `cape`, FFI declarations, or effect handling that is not already settled by the language docs. Capability-call design belongs to `docs/factory/capability-calls/goal.md`.
