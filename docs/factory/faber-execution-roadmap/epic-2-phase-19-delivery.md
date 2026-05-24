# Epic 2 Phase 19 Delivery: Rust Method Receivers

## Interpreted Problem

Several valid `genus` examples still fail in the Rust e2e corpus because struct methods are emitted as associated functions with no receiver. Inside those method bodies, `ego` has already lowered to the enclosing struct `DefId`, and Rust codegen currently renders that path as the type name, producing invalid target code such as `Rectangle.width`.

## Normalized Spec

- Emit inherent Rust methods with an implicit receiver.
- Lower the enclosing struct path to `self` while emitting that struct's methods.
- Infer `&mut self` only for methods that assign through `ego`; otherwise emit `&self`.
- Do not address constructor hooks, empty struct literals, or broader object semantics in this phase.
- Add focused Rust backend coverage proving both immutable and mutable method receivers.

## Repo-Aware Baseline

- HIR lowering records the enclosing struct while lowering `ego`, then emits `HirExprKind::Path(struct_def_id)`.
- `HirMethod.receiver` exists but lowering currently records `HirReceiver::None`.
- Rust struct method emission reuses ordinary function generation, so no receiver appears in method signatures.
- Rust expression path emission always calls `resolve_def`, so an `ego` path inside a method becomes the struct type name.

## Stage Graph

1. Add Rust backend self context to `RustCodegen`.
2. Generate struct methods through a method-specific helper that inserts the inferred receiver.
3. Set current self context while emitting method bodies.
4. Teach Rust path emission to render `self` for the active self `DefId`.
5. Add focused codegen coverage.
6. Run focused tests and the ignored Rust exempla e2e suite.

## Epic Candidates And Scopable Issues

This phase covers only receiver and `ego` lowering. Remaining struct issues, especially `creo` constructor execution and empty struct literal emission, stay as later phases unless validation proves they are trivial fallout.

## Checkpoints

- Generated `area`-style methods use `&self` and read `self.field`.
- Generated mutating methods use `&mut self` and assign `self.field = ...`.
- Existing top-level function generation is unchanged.
- E2E count is recorded after the phase.

## Companion Skill Plan

- `factory`: preserve phase boundary, validation, and commit discipline.
- `delivery`: this saved implementation artifact.

## Gate Plan

- `cargo test -p radix rust_methods_emit_self_receivers`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`
- `cargo fmt`
- Commit only this phase if the focused test passes and e2e does not regress unexpectedly.

## Open Questions

- Whether `creo` should become a generated constructor hook or post-literal initializer remains out of scope for this phase.
