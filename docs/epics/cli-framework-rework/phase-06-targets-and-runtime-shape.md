# Phase 06: Targets and Runtime Shape

## Goal

Turn the first target-specific implementation into a maintainable multi-target strategy.

## Scope

- Revisit the Phase 03 choice to emit self-contained Rust parser logic.
- Decide whether repeated Rust parser logic should move into `radix/crates/norma` as `norma::cli`, a separate runtime crate, or remain emitted inline.
- Define target capability expectations for TypeScript, Rust, Go, and future targets.
- Decide how target capability limits surface in `radix targets`.
- Add conformance tests for shared CLI behavior across supported targets.
- Document intentionally unsupported target/feature combinations.

## Phase Decisions From Earlier Work

- Phase 03 deliberately starts with self-contained generated Rust parser code.
- Phase 06 is the first phase that should consider extracting reusable runtime support.
- If runtime support is needed, prefer an internal `norma::cli` module before creating a separate public crate.
- A separate public CLI runtime crate should require evidence that the API is stable enough to publish and version independently.

## Out Of Scope

- Adding every target at once.
- Advanced interactive CLI features.
- Shell completion generation.

## Design Questions

- Did the Phase 03 generated Rust parser create enough duplication or complexity to justify runtime extraction?
- Should runtime support live in `radix/crates/norma`, a new crate, or target-local generated helpers?
- Which target should become the reference implementation for behavior?
- How much help formatting should be shared versus target-local?
- What package/linkage contract should generated programs use when they depend on runtime support?

## Acceptance

- Target support is explicit and discoverable.
- At least two targets share the same CLI IR contract.
- Unsupported target/feature combinations fail before misleading code is emitted.
- Any runtime extraction decision is backed by working Phase 03 behavior and conformance tests, not by speculation.
