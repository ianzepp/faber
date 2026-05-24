# Phase 1 Delivery: Rust Writer Naming Cleanup

**Parent Ledger**: `docs/factory/radix-codegen-writer-cleanup/ledger.md`
**Phase Goal**: Replace legacy short writer names in the Rust-facing Radix codegen tree with explicit `writer` naming, and align nested `ExprEmitter` construction sites with the struct-based emitter style.

## Interpreted Problem

Radix Rust code generation has mostly moved toward `writer.write(...)` through `ExprEmitter`, but older modules still use `w.write(...)` and pass local closure variables named `w` into nested emitters. This leaves the Rust backend split between two conventions and makes the newer `ExprEmitter` structure less obvious.

## Normalized Spec

In scope:

- Rename Rust backend `CodeWriter` parameters and locals named `w` to `writer`.
- Replace `w.write(...)` with `writer.write(...)` in Rust-facing codegen files.
- Rename nested expression closure parameters from `w` to `writer` where those closures create `ExprEmitter` instances.
- Rename the MIR Rust probe writer field from `w` to `writer`.
- Preserve behavior, formatting shape, generated output, and public APIs.

Out of scope:

- Go, TypeScript, and Faber target emitters that still use `w.write(...)`.
- CodeWriter API changes.
- Generated output changes.
- Larger Rust backend restructuring.

## Repo-Aware Baseline

Confirmed hotspots before implementation:

- `crates/radix/src/codegen/rust/cli.rs`
- `crates/radix/src/codegen/rust/decl.rs`
- `crates/radix/src/codegen/rust/stmt.rs`
- `crates/radix/src/codegen/rust/mod.rs`
- `crates/radix/src/codegen/rust/prelude.rs`
- `crates/radix/src/mir/rust_probe.rs`
- Nested `ExprEmitter::new(..., w, ...)` sites under `crates/radix/src/codegen/rust/expr/`

The non-Rust target emitters also contain many `w.write(...)` calls, but this phase deliberately avoids them so the checkpoint stays narrow and reviewable.

## Stage Graph

1. Inventory Rust-facing `w.write(...)` and nested emitter `w` sites.
2. Apply mechanical renames in small, scoped passes.
3. Format the touched Rust code.
4. Search for leftover scoped `w.write(...)` and `ExprEmitter::new(..., w, ...)` sites.
5. Run focused Radix verification.
6. Audit the diff for behavior changes before committing.

## Epic Candidates And Scopable Issues

- `rust-expr-nested-writer`: Rename nested emitter closure variables in Rust expression modules.
- `rust-backend-writer-params`: Rename backend helper parameters and uses from `w` to `writer`.
- `mir-rust-probe-writer`: Rename the probe writer field and references.

These are handled in one phase because they are all behavior-preserving convention cleanup over the same naming contract.

## Checkpoints

The phase passes when:

- No `w.write(...)` remains in `crates/radix/src/codegen/rust` or `crates/radix/src/mir/rust_probe.rs`.
- No `ExprEmitter::new(..., w, ...)` remains in `crates/radix/src/codegen/rust`.
- `cargo fmt --check` passes.
- `cargo check -p radix` passes.
- The diff contains only the delivery docs and naming-only source changes.

## Companion Skill Plan

- `factory`: Owns phase execution, verification, poker-face gate, and commit.
- `delivery`: Supplies this single-phase delivery spec.
- `poker-face`: Perform an end-of-phase completion audit against this document.

## Gate Plan

Gate result is `PASS` only if the searches and verification commands pass and the diff does not include behavior changes. If verification fails for a pre-existing unrelated reason, document the failure and do not commit without a clear reason. If the cleanup expands into non-Rust targets, stop and split a new phase.

## Open Questions

- Should the Go, TypeScript, and Faber emitters later adopt `writer` naming too?
