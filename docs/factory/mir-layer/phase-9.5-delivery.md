# Phase 9.5 Delivery: MIR Hardening and Closeout

**Status**: planned.

## Interpreted Problem

Phases 0-9 established an execution-shaped MIR, lowered a meaningful Faber subset into it, validated it, exposed it through `radix mir`, and proved that validated MIR can feed an executable output path through a deliberately temporary Rust probe.

The next useful step is not Rust backend migration, WASM bytecode, or native planning. Those are larger backend projects. The useful closeout step is to harden the MIR work already done, reconcile documentation with implementation, improve confidence in correctness, and leave a clean handoff point for future lower-target work.

Phase 9.5 should make the MIR layer boringly reliable for its current scope. It should not expand into a new backend campaign.

## Normalized Spec

- Run an end-to-end review of the MIR implementation from data model through lowering, validation, dump output, CLI inspection, and the Rust probe.
- Use focused agents or independent review passes where they materially improve correctness, especially around MIR invariants, validation gaps, and lowered-shape parity.
- Reconcile factory docs, phase statuses, the ledger, and deferred roadmap entries.
- Harden validation or lowering where review finds clearly bounded correctness gaps.
- Improve representative tests for the current MIR contract.
- Clean up comments, names, module boundaries, and local readability in the MIR implementation.
- Keep the existing HIR-to-Rust backend as the stable Rust codegen path.
- Keep the MIR Rust probe temporary and deleteable.
- Document future WASM/lower-target prerequisites without implementing WASM, WAT, bytecode, native, Cranelift, or ABI machinery.
- Finish with repository-standard validation.

## Review Scope

The closeout review should cover:

- `crates/radix/src/mir/nodes.rs`
- `crates/radix/src/mir/lower.rs`
- `crates/radix/src/mir/validate.rs`
- `crates/radix/src/mir/dump.rs`
- `crates/radix/src/mir/rust_probe.rs`
- MIR-focused tests under `crates/radix/src/mir/`
- `radix mir` command wiring
- MIR factory documents under `docs/factory/mir-layer/`

Read neighboring HIR, semantic, and Rust-codegen files only as needed to verify the MIR contract or parity assumptions.

## Correctness Focus

Phase 9.5 should look for bounded, fixable issues in:

- block termination and unreachable block handling,
- local/temp/value reference validation,
- assignment and return type compatibility,
- failable-call and alternate-exit edge validation,
- structured `cape` lowering shape,
- option/null operation validation,
- aggregate construction/projection validation,
- runtime intrinsic identity and argument validation,
- deterministic MIR dump output,
- probe fail-closed behavior for unsupported MIR,
- parity fixtures that compare lowered MIR behavior against the existing HIR-to-Rust backend.

Do not add broad new semantics just to satisfy a hypothetical future backend. If a gap requires ABI, layout, memory ownership, host imports, or bytecode policy, document it as future work.

## Housekeeping Scope

The closeout should include normal repository hygiene around the MIR work:

- format and lint cleanup,
- focused removal of confusing dead code or redundant helpers introduced by the MIR effort,
- comment cleanup,
- small readability improvements where they reduce review risk,
- test helper cleanup,
- documentation drift repair,
- ledger updates,
- final validation.

Avoid unrelated refactors. Keep changes close to MIR and factory documentation.

## Future WASM/Lessons Handoff

Record future lower-target prerequisites explicitly:

- exported-function and `incipit` model,
- primitive value ABI,
- string and memory representation,
- option, struct, enum, array, and map layout,
- runtime intrinsic import contract,
- provider call boundary,
- host-side diagnostic capture,
- Wasmtime or equivalent validation harness,
- package/build artifact shape,
- source-map or diagnostic-location story.

These are blockers for a future WASM/lower-target project, not required Phase 9.5 implementation tasks.

## Out Of Scope

- Migrating Rust codegen from HIR to MIR.
- Making the Rust probe a permanent backend.
- Implementing WASM, WAT, bytecode, native output, Cranelift, or an ABI.
- Replacing package build behavior.
- Adding broad new Faber syntax support.
- Introducing SSA or optimization.
- Solving runtime memory management.
- Large unrelated cleanup outside MIR and MIR factory docs.

## Validation

- `cargo fmt --all --check`
- `cargo test -p radix mir`
- `cargo test -p radix`
- `./scripta/ci`

If a validation command fails because of an existing issue outside the MIR closeout scope, record the failure precisely and do not hide it.

## Completion Gate

Phase 9.5 is complete when the MIR layer has passed an end-to-end correctness and housekeeping review, bounded hardening fixes are applied, docs and ledger reflect the actual project state, Rust remains on the HIR backend, phases 10-12 are explicitly deferred, future WASM/lower-target prerequisites are recorded, and the repo's validation gates pass or have precisely documented external blockers.
