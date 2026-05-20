# Radix Large Module Refactor Factory Plan

**Status**: factory-approved phased plan. No refactor work has been performed by this document.

This plan turns the large-module housekeeping finding into a staged implementation program. The goal is not to extract one or two helpers, but to comprehensively split oversized compiler modules into cohesive submodules while preserving behavior.

Primary repo:

- `/Users/ianzepp/work/ianzepp/faber`

Important rules:

- Do not run Git commands that create locks in parallel in this repository.
- Commit after each completed phase.
- Prefer line-moving tools such as `slice` for bulk refactors instead of regenerating large Rust files.
- Keep public module surfaces stable unless a phase explicitly says otherwise.
- Run the validation gate after every phase.

## Interpreted Problem

The current `radix` compiler has several production Rust modules that are too large to review or change comfortably:

- `radix/crates/radix/src/semantic/passes/typecheck.rs`
- `radix/crates/radix/src/codegen/faber/mod.rs`
- `radix/crates/radix/src/codegen/go/expr.rs`
- `radix/crates/radix/src/codegen/rust/expr.rs`

These files are not merely long. They mix dispatch, phase state, syntax rendering, type inference, access checking, conversion handling, and target-specific lowering rules in single modules. That makes localized changes harder than they need to be.

The desired result is a module tree where each file owns a coherent compiler concept:

- `mod.rs` files keep public entry points, shared state, and orchestration.
- expression modules own expression dispatch only at the top level.
- operation, call, access, aggregate, pattern, conversion, inference, and literal logic have named homes.
- tests continue to use the existing companion test convention.

## Factory Approval

This plan is suitable for factory execution because it has:

- a fixed phase set
- clear commit boundaries
- bounded owned paths per phase
- no intended semantic behavior changes
- strong existing verification gates
- resumable checkpoints after each phase

Factory should execute one phase at a time. Each phase should have a short delivery spec saved before implementation begins, then a completed implementation commit after validation passes.

Suggested delivery spec location:

```text
docs/factory/radix-large-module-refactor/
```

Suggested factory ledger:

```text
docs/factory/radix-large-module-refactor/ledger.md
```

## Validation Gate

Run this gate after every implementation phase:

```bash
bun run ci
bun run lint
bunx eslint .
bun run prettier:check
bun run build:radix
```

Phase-specific smoke checks may add narrower commands, but they do not replace the full gate.

Expected final test shape is the current radix gate:

- radix library tests pass
- radix CLI tests pass
- hygiene ratchet passes
- doctests pass
- ignored slow E2E tests remain ignored unless a phase explicitly opts into them

## Stage Graph

```text
Phase 0: Preflight and delivery-spec setup
  -> Phase 1: Split Faber codegen
  -> Phase 2: Split typecheck pass
  -> Phase 3: Split Go expression codegen
  -> Phase 4: Split Rust expression codegen
  -> Phase 5: Documentation and final hygiene review
```

Phases must run serially. They touch overlapping compiler APIs and should not be parallelized in the same worktree.

## Phase 0: Preflight and Delivery-Spec Setup

Objective: establish the exact baseline and create factory artifacts before moving code.

Owned paths:

- `docs/factory/radix-large-module-refactor/**`

Steps:

1. Confirm the worktree is clean.
2. Record current HEAD and validation status in the ledger.
3. Create a compact delivery spec for Phase 1.
4. Run the validation gate once before any refactor.

Checkpoint:

- baseline validation is green
- ledger exists
- Phase 1 delivery spec exists

Commit:

```text
docs: add radix large-module refactor factory ledger
```

## Phase 1: Split Faber Codegen

Objective: reshape `codegen/faber/mod.rs` into the same broad module pattern already used by Go, Rust, and TypeScript codegen.

Owned paths:

- `radix/crates/radix/src/codegen/faber/mod.rs`
- `radix/crates/radix/src/codegen/faber/*.rs`
- `radix/crates/radix/src/codegen/faber/mod_test.rs`

Target shape:

```text
radix/crates/radix/src/codegen/faber/
├── mod.rs
├── decl.rs
├── stmt.rs
├── expr.rs
├── pattern.rs
├── types.rs
├── literal.rs
├── names.rs
└── ops.rs
```

Responsibilities:

- `mod.rs`: `FaberCodegen`, constructor/default, `impl Codegen`, top-level generation orchestration.
- `decl.rs`: items, functions, proba functions, structs, enums, interfaces.
- `stmt.rs`: blocks, statements, `si` / `sin` / `secus`, `fac` loop helpers, branch body helpers.
- `expr.rs`: `write_expr`, `write_expr_prec`, expression precedence dispatch.
- `pattern.rs`: pattern rendering and pattern name collection.
- `types.rs`: `type_to_faber`, option flattening.
- `literal.rs`: literals, quoted text, quoted symbols, object fields.
- `names.rs`: name collection and `name_for_def` helpers.
- `ops.rs`: binary, assignment, unary, and precedence helpers.

Implementation notes:

- Prefer moving existing methods into separate `impl FaberCodegen` blocks.
- Keep function names stable where practical.
- Keep `mod_test.rs` path and test module convention unchanged.

Checkpoint:

- no behavior changes intended
- `codegen/faber/mod.rs` becomes orchestration-sized
- Faber codegen tests still pass
- full validation gate passes

Commit:

```text
refactor: split faber codegen modules
```

## Phase 2: Split Typecheck Pass

Objective: split `semantic/passes/typecheck.rs` by typechecker responsibility while preserving the public `typecheck(...)` entry point.

Owned paths:

- `radix/crates/radix/src/semantic/passes/typecheck.rs`
- `radix/crates/radix/src/semantic/passes/typecheck/**`
- `radix/crates/radix/src/semantic/passes/mod.rs`
- `radix/crates/radix/src/semantic/passes/typecheck_test.rs`

Target shape:

```text
radix/crates/radix/src/semantic/passes/typecheck/
├── mod.rs
├── collect.rs
├── finalize.rs
├── item.rs
├── stmt.rs
├── expr.rs
├── ops.rs
├── call.rs
├── access.rs
├── aggregate.rs
├── control.rs
├── pattern.rs
├── convert.rs
├── infer.rs
└── lookup.rs
```

Responsibilities:

- `mod.rs`: shared structs, public `typecheck`, `TypeChecker::new`, pass orchestration.
- `collect.rs`: top-level item, struct, and function signature collection.
- `finalize.rs`: HIR finalization for items, functions, blocks, statements, expressions, and object fields.
- `item.rs`: item, const, and function checking.
- `stmt.rs`: blocks, statements, locals, returns.
- `expr.rs`: expression dispatch and expected-type routing.
- `ops.rs`: binary, unary, assignment operators, numeric common-type logic.
- `call.rs`: calls, method calls, call args, spread compatibility, function signatures.
- `access.rs`: paths, fields, indexes, optional chains, non-null, lvalue checks.
- `aggregate.rs`: arrays, tuples, struct literals, struct field checks.
- `control.rs`: `if`, conditions, and `discerne` / match checking.
- `pattern.rs`: pattern checking and pattern binding rules.
- `convert.rs`: `verte`, `conversio`, dereference behavior.
- `infer.rs`: fresh inference vars, unify, bind, occurs checks.
- `lookup.rs`: alias resolution, type lookup helpers, primitive type helpers, method signature lookup.

Implementation notes:

- Use `pub(super)` or private module visibility; do not expose internals outside the pass.
- Keep `semantic::passes::typecheck::typecheck(...)` stable.
- Prefer separate `impl TypeChecker<'_>` blocks per file over introducing new state carriers.
- Do not change inference behavior to make the split easier.

Checkpoint:

- `typecheck_test.rs` passes
- no diagnostics change unless explicitly proven equivalent and documented
- full validation gate passes

Commit:

```text
refactor: split typecheck pass modules
```

## Phase 3: Split Go Expression Codegen

Objective: split `codegen/go/expr.rs` into target-specific expression submodules without changing Go output.

Owned paths:

- `radix/crates/radix/src/codegen/go/expr.rs`
- `radix/crates/radix/src/codegen/go/expr/**`
- `radix/crates/radix/src/codegen/go/mod.rs`
- `radix/crates/radix/src/codegen/go/mod_test.rs`

Target shape:

```text
radix/crates/radix/src/codegen/go/expr/
├── mod.rs
├── literal.rs
├── ops.rs
├── collection.rs
├── access.rs
├── option.rs
├── call.rs
├── convert.rs
└── variants.rs
```

Responsibilities:

- `mod.rs`: public `generate_expr` dispatch and shared expression helpers.
- `literal.rs`: literal rendering, string/scriptum template helpers.
- `ops.rs`: unary, binary, assignment operator rendering.
- `collection.rs`: arrays, maps, typed collection literals, `ab` pipelines and filters.
- `access.rs`: field/index/map member access and asserted map value typing.
- `option.rs`: option wrapping, optional chain, coalesce, option field helpers.
- `call.rs`: intrinsics, translated method calls, receiver handling, spread recovery.
- `convert.rs`: `verte`, `conversio`, value and boolean conversion.
- `variants.rs`: enum/interface constructors and variant value detection.

Implementation notes:

- Preserve `expr::generate_expr(...)` as the call site used by `stmt.rs` and `decl.rs`.
- Keep Go type rendering in `types.rs`; do not move type system rendering into expression modules.
- Keep target behavior byte-for-byte where practical.

Checkpoint:

- Go codegen tests pass
- no Go output regressions in existing snapshots/assertions
- full validation gate passes

Commit:

```text
refactor: split go expression codegen
```

## Phase 4: Split Rust Expression Codegen

Objective: split `codegen/rust/expr.rs` into Rust expression submodules without changing Rust output.

Owned paths:

- `radix/crates/radix/src/codegen/rust/expr.rs`
- `radix/crates/radix/src/codegen/rust/expr/**`
- `radix/crates/radix/src/codegen/rust/mod.rs`
- `radix/crates/radix/src/codegen/rust/mod_test.rs`

Target shape:

```text
radix/crates/radix/src/codegen/rust/expr/
├── mod.rs
├── literal.rs
├── ops.rs
├── block.rs
├── pattern.rs
├── collection.rs
├── control.rs
├── call.rs
├── option.rs
└── format.rs
```

Responsibilities:

- `mod.rs`: public `generate_expr` dispatch and shared helpers.
- `literal.rs`: literal rendering, string escaping, regex flag handling.
- `ops.rs`: binary and unary expression generation.
- `block.rs`: expression block generation.
- `pattern.rs`: pattern generation.
- `collection.rs`: innatum arrays/maps, object map keys, collection helpers.
- `control.rs`: `if`, match, and loop-shaped expression forms.
- `call.rs`: function calls, method calls, failable call handling.
- `option.rs`: optional chain, coalesce, non-null, option-specific helpers.
- `format.rs`: `scriptum`, `scribe` format selection, format template rendering.

Implementation notes:

- Preserve `expr::generate_expr(...)`.
- Keep Rust type rendering in `types.rs` and failable analysis in `failable.rs`.
- Avoid changing ownership or `Result` propagation semantics during the split.

Checkpoint:

- Rust codegen tests pass
- package compilation tests pass
- full validation gate passes

Commit:

```text
refactor: split rust expression codegen
```

## Phase 5: Documentation and Final Hygiene Review

Objective: update developer documentation and verify the split improved navigability without creating new stale docs.

Owned paths:

- `README.md`
- `AGENTS.md`
- `docs/**/*.md`
- `radix/crates/radix/README.md`
- files changed by previous phases only for tiny follow-up cleanups

Steps:

1. Review `README.md`, `AGENTS.md`, `radix/crates/radix/README.md`, and relevant docs for stale module-shape claims.
2. Update docs only where the refactor changed described paths or contributor guidance.
3. Re-run the full validation gate.
4. Run a final source-size scan:

   ```bash
   find radix/crates -path '*/target' -prune -o -name '*.rs' -not -name '*test.rs' -not -path '*/tests/*' -print0 | xargs -0 wc -l | sort -nr | sed -n '1,40p'
   ```

Checkpoint:

- docs match the new module shape
- no new formatter/lint/test failures
- remaining large files are either intentional orchestration files or deferred with a specific reason

Commit:

```text
docs: update radix module refactor notes
```

## Out of Scope

Do not include these in this factory run unless a later phase explicitly expands scope:

- semantic behavior changes
- new syntax or grammar changes
- diagnostic wording changes
- public CLI changes
- test harness rewrites
- moving parser modules
- splitting `driver/mod.rs`
- splitting runtime HAL implementations

## Deferred Follow-Up Candidates

After this factory run, consider separate plans for:

- `radix/crates/radix/src/driver/mod.rs`
- `radix/crates/radix/src/syntax/ast.rs`
- `radix/crates/radix/src/semantic/passes/resolve.rs`
- `radix/crates/radix/src/parser/expr.rs`
- runtime HAL modules that cross 400 lines

Those should not be folded into this run. The first factory run should make the most painful compiler modules navigable and stop there.

