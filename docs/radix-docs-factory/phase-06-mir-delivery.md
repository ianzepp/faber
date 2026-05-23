# Phase 06 Delivery Spec: MIR

## Target

Document the middle IR model, HIR-to-MIR lowering, validation, dumping, visitor
semantics, and the temporary Rust probe boundary.

## Files

- `crates/radix/src/mir/mod.rs`
- `crates/radix/src/mir/nodes.rs`
- `crates/radix/src/mir/visit.rs`
- `crates/radix/src/mir/dump.rs`
- `crates/radix/src/mir/validate.rs`
- `crates/radix/src/mir/rust_probe.rs`
- `crates/radix/src/mir/lower.rs`
- `crates/radix/src/mir/lower/context.rs`
- `crates/radix/src/mir/lower/item.rs`
- `crates/radix/src/mir/lower/expr.rs`
- `crates/radix/src/mir/lower/stmt.rs`
- `crates/radix/src/mir/lower/control.rs`
- `crates/radix/src/mir/lower/aggregate.rs`
- `crates/radix/src/mir/lower/place.rs`
- `crates/radix/src/mir/lower/runtime.rs`

## Acceptance Criteria

- MIR's role below typed HIR and above target codegen is clear.
- The MIR data model documents stable IDs, storage order, places, operands,
  values, statements, terminators, runtime calls, and type/layout placeholders.
- Lowering documentation explains supported HIR shapes, unsupported/fail-closed
  behavior, builder state, entry/function lowering, and error preservation.
- Validation documentation explains structural invariants, type checks, CFG
  target/reference checks, call/error-edge checks, and why validation is
  separate from lowering.
- Dump and visitor documentation explain deterministic rendering and traversal
  semantics.
- The Rust probe is documented as temporary, limited, and separate from normal
  target codegen.
- Documentation is behavior-preserving and excludes test files.

## Out Of Scope

- Extending MIR coverage or changing lowering behavior.
- Test file changes.
- Target backend rewrites.

## Validation Plan

- `cargo fmt --check`
- `cargo test -p radix`
- `git diff --check`

## Verification

- `cargo fmt --check` passed.
- `cargo test -p radix` passed: 425 passed, 0 failed, 3 ignored; hygiene and
  doctest targets passed.
- `git diff --check` passed.
- Poker-face completion estimate: 94%. Largest residual issue was this
  verification section being stale before final update; evaluator noted only a
  minor wording risk around CFG validation breadth, now clarified as
  target/reference checks.
