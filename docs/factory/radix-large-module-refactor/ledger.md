# Radix Large Module Refactor Factory Ledger

**Factory Run**: radix-large-module-refactor per `docs/radix-large-module-refactor-factory-plan.md`
**Started**: 2026-05-20
**Target Repo**: /Users/ianzepp/work/ianzepp/faber (radix primary)
**Phase Set Source**: docs/radix-large-module-refactor-factory-plan.md (factory-approved)
**Delivery Spec Dir**: docs/factory/radix-large-module-refactor/
**Commit Policy**: Commit after each phase completion + validation gate pass
**Agent Policy**: Use subagents for exploration, poker-face, verification, bounded impl where appropriate; main agent supervises and integrates
**Checkpoint Policy**: Full validation gate after every phase; phase-specific smoke tests
**Current Status**: Phase 5 complete; second-pass refactor completion committed and gate PASS.

## Baseline (Phase 0 Intake)

- HEAD: 7619f4f3 (docs: clarify radix refactor extraction strategy)
- Branch: main (ahead of origin/main by 1)
- Working tree: clean
- Validation gate baseline run: PASS (see below)

## Phases

| Phase | Name | Status | Commit | Delivery Spec |
|-------|------|--------|--------|---------------|
| 0 | Preflight and delivery-spec setup | completed | a0e3838c | N/A (ledger + Phase 1 spec) |
| 1 | Split Faber codegen | completed | c4368940 | `phase-01-faber-codegen-delivery.md` |
| 2 | Split typecheck pass | completed | 4ee6a5e8 | `phase-02-typecheck-delivery.md` |
| 3 | Split Go expression codegen | completed | f6b2d01d | `phase-03-go-expr-delivery.md` |
| 4 | Split Rust expression codegen | completed | 34c41322 | `phase-04-rust-expr-delivery.md` |
| 5 | Documentation and final hygiene review | completed | 8632a5ab | `phase-05-docs-hygiene-delivery.md` |
| Follow-up | Second-pass refactor completion | completed | this commit | N/A |

## Validation Gate Log

**Baseline run started at Phase 0**:

- `bun run check:radix`: PASS (fmt clean)
- `bun run lint`: PASS (0.29s, cached)
- `bun run prettier:check`: PASS
- `bunx eslint .`: PASS (clean)
- `bun run test:radix`: PASS (full suite + hygiene + doctests ok, fast due to cache)
- `bun run build:radix`: PASS (release build cached, 0.03s)
- **FULL BASELINE VALIDATION GATE: PASS** (all components green at HEAD 39707302)

See per-phase sections for subsequent runs.

## Open Questions / Notes

- None blocking at intake.
- Will use `slice` CLI extensively for line-accurate moves during impl phases.
- No behavior changes; public APIs for `typecheck(...)` and `generate_expr` etc. preserved.
- Out of scope items deferred as documented.

## Phase 0 Completion

- [x] Ledger created at `docs/factory/radix-large-module-refactor/ledger.md`
- [x] Phase 1 compact delivery spec created at `phase-01-faber-codegen-delivery.md`
- [x] Full baseline validation gate PASS (fmt, clippy, prettier, eslint, test:radix, build:radix all green)
- [x] Ready for commit

**Phase 0 Gate Result**: PASS. All artifacts in place. Working tree still clean (only new docs files).

**Next**: Commit with message per plan, then select and hand Phase 1 to implementation (delivery spec already persisted).

## Commit Record

- Hash: a0e3838c
- Message: `docs: add radix large-module refactor factory ledger`
- Files: docs/factory/radix-large-module-refactor/ledger.md + phase-01-*.md

This completes Phase 0 per master plan. Factory run is resumable from ledger.

## Phase 1 Completion

- Split `radix/crates/radix/src/codegen/faber/mod.rs` from 1804 LOC to 100 LOC.
- Added cohesive Faber codegen modules:
  - `decl.rs`
  - `stmt.rs`
  - `expr.rs`
  - `pattern.rs`
  - `types.rs`
  - `literal.rs`
  - `names.rs`
  - `ops.rs`
- Kept `FaberCodegen::new()`, `impl Default`, `impl Codegen`, and `mod_test.rs` convention stable.
- Used method-anchored extraction; attached clippy attributes moved with their methods.

**Phase 1 Verification**:

- `cargo check --manifest-path radix/Cargo.toml -p radix`: PASS
- `bun run lint`: PASS
- `bun run test:radix`: PASS
- `cargo run --manifest-path radix/Cargo.toml -p radix -- emit -t faber examples/exempla/salve-munde.fab`: PASS
- `bun run ci`: PASS
- `bunx eslint .`: PASS
- `bun run prettier:check`: PASS
- `bun run build:radix`: PASS

**Phase 1 Gate Result**: PASS.

## Phase 1 Commit Record

- Hash: c4368940
- Message: `refactor: split faber codegen modules`
- Files: Faber codegen split into 8 target modules plus ledger update.

## Phase 2 Completion

- Moved `radix/crates/radix/src/semantic/passes/typecheck.rs` to `radix/crates/radix/src/semantic/passes/typecheck/mod.rs`.
- Split the typechecker into responsibility modules:
  - `collect.rs`
  - `finalize.rs`
  - `item.rs`
  - `stmt.rs`
  - `expr.rs`
  - `ops.rs`
  - `call.rs`
  - `access.rs`
  - `aggregate.rs`
  - `control.rs`
  - `pattern.rs`
  - `convert.rs`
  - `infer.rs`
  - `lookup.rs`
- Preserved public `semantic::passes::typecheck::typecheck(...)` entry point.
- Updated test module path for the new `typecheck/mod.rs` location.
- Removed three no-op `let _ =` statements to keep the hygiene ratchet green after the split.

**Phase 2 Verification**:

- `cargo check --manifest-path radix/Cargo.toml -p radix`: PASS
- `bun run lint`: PASS
- `bun run ci`: PASS
- `bunx eslint .`: PASS
- `bun run prettier:check`: PASS
- `bun run build:radix`: PASS

**Phase 2 Gate Result**: PASS.

## Phase 2 Commit Record

- Hash: 4ee6a5e8
- Message: `refactor: split typecheck pass modules`
- Files: Typecheck pass moved to `typecheck/mod.rs` and split into 14 target modules plus Phase 2 delivery spec and ledger update.

## Phase 3 Completion

- Moved `radix/crates/radix/src/codegen/go/expr.rs` to `radix/crates/radix/src/codegen/go/expr/mod.rs`.
- Split Go expression helper code into:
  - `literal.rs`
  - `ops.rs`
  - `collection.rs`
  - `access.rs`
  - `option.rs`
  - `call.rs`
  - `convert.rs`
  - `variants.rs`
- Preserved public `expr::generate_expr(...)` and `expr::generate_expr_for_go_type(...)` call sites.
- Kept shared helpers in `expr/mod.rs` and re-used helper modules through internal imports.

**Phase 3 Verification**:

- `cargo check --manifest-path radix/Cargo.toml -p radix`: PASS
- `bun run lint`: PASS
- `bun run ci`: PASS
- `bunx eslint .`: PASS
- `bun run prettier:check`: PASS
- `bun run build:radix`: PASS

**Phase 3 Gate Result**: PASS.

## Phase 3 Commit Record

- Hash: f6b2d01d
- Message: `refactor: split go expression codegen`
- Files: Go expression codegen moved to `go/expr/mod.rs`, split into 8 target modules, plus Phase 3 delivery spec and ledger update.

## Phase 4 Completion

- Moved `radix/crates/radix/src/codegen/rust/expr.rs` to `radix/crates/radix/src/codegen/rust/expr/mod.rs`.
- Split Rust expression codegen into:
  - `literal.rs`
  - `ops.rs`
  - `block.rs`
  - `pattern.rs`
  - `collection.rs`
  - `control.rs`
  - `call.rs`
  - `option.rs`
  - `format.rs`
- Preserved public `expr::generate_expr(...)` and internal `expr::generate_expr_unwrapped(...)` call sites.
- Kept Rust type rendering in `types.rs` and failable analysis in `failable.rs`.
- Used handler extraction for call, option, control, collection, block, operator, literal, pattern, and format responsibilities.

**Phase 4 Verification**:

- `cargo check --manifest-path radix/Cargo.toml -p radix`: PASS
- `bun run lint`: PASS
- `bun run ci`: PASS
- `bunx eslint .`: PASS
- `bun run prettier:check`: PASS
- `bun run build:radix`: PASS

**Phase 4 Gate Result**: PASS.

## Phase 4 Commit Record

- Hash: 34c41322
- Message: `refactor: split rust expression codegen`
- Files: Rust expression codegen moved to `rust/expr/mod.rs`, split into 9 target modules, plus Phase 4 delivery spec and ledger update.

## Phase 5 Completion

- Added Phase 5 delivery spec.
- Updated active documentation references that still pointed at removed monolithic paths:
  - `docs/faber-mechanics.md`: typecheck references now point at `typecheck/mod.rs` or `typecheck/finalize.rs`.
  - `docs/go-emitter-delivery-plan.md`: Go expression workstream paths now point at `go/expr/mod.rs` and `go/expr/**`.
  - `docs/radix-large-module-refactor-factory-plan.md`: status now records that the plan has been implemented.
- Reviewed `README.md`, `AGENTS.md`, `radix/crates/radix/README.md`, and `docs/**/*.md` for stale module-shape claims.

**Final Source-Size Scan Top Entries**:

```text
  32307 total
   1317 radix/crates/radix/src/driver/mod.rs
   1195 radix/crates/radix/src/syntax/ast.rs
   1184 radix/crates/radix/src/semantic/passes/resolve.rs
   1143 radix/crates/radix/src/parser/expr.rs
    985 radix/crates/radix/src/parser/decl.rs
    914 radix/crates/radix/src/codegen/ts/expr.rs
    874 radix/crates/radix/src/lexer/scan.rs
    827 radix/crates/radix/src/hir/lower/stmt.rs
    805 radix/crates/radix/src/codegen/go/stmt.rs
    771 radix/crates/radix/src/codegen/rust/expr/mod.rs
    727 radix/crates/radix/src/parser/stmt.rs
    709 radix/crates/radix/src/main.rs
    661 radix/crates/radix/src/hir/lower/expr.rs
    660 radix/crates/radix/src/semantic/passes/borrow.rs
    614 radix/crates/radix/src/codegen/go/mod.rs
    602 radix/crates/radix/src/codegen/go/expr/mod.rs
```

Remaining large files are outside this phase set or are orchestration/legacy-large surfaces not targeted by this plan.

**Phase 5 Verification**:

- `bun run lint`: PASS
- `bun run ci`: PASS
- `bunx eslint .`: PASS
- `bun run prettier:check`: PASS
- `bun run build:radix`: PASS

**Phase 5 Gate Result**: PASS.

## Phase 5 Commit Record

- Hash: 8632a5ab
- Message: `docs: update radix module refactor notes`
- Files: final docs hygiene updates and ledger notes.

## Second-Pass Refactor Completion

Second-pass review found that Go and Rust expression dispatch modules still held substantial target-specific lowering logic. This follow-up completed the intended module responsibility split while preserving behavior.

- Go `expr/mod.rs` now keeps dispatch and shared helpers; access, call, collection, control, conversion, formatting/literal, option, ops, and variant helpers live in named submodules.
- Rust `expr/mod.rs` now keeps dispatch and shared helpers; access, call, collection, control, conversion, format, literal, ops, option, pattern, and `verte` helpers live in named submodules.
- `docs/radix-large-module-refactor-factory-plan.md` was updated to record the additional submodules that were needed to make the responsibility split truthful.

**Second-Pass Source-Size Scan Top Entries**:

```text
  32946 total
   1317 radix/crates/radix/src/driver/mod.rs
   1195 radix/crates/radix/src/syntax/ast.rs
   1184 radix/crates/radix/src/semantic/passes/resolve.rs
   1143 radix/crates/radix/src/parser/expr.rs
    985 radix/crates/radix/src/parser/decl.rs
    914 radix/crates/radix/src/codegen/ts/expr.rs
    874 radix/crates/radix/src/lexer/scan.rs
    827 radix/crates/radix/src/hir/lower/stmt.rs
    805 radix/crates/radix/src/codegen/go/stmt.rs
    727 radix/crates/radix/src/parser/stmt.rs
    709 radix/crates/radix/src/main.rs
    661 radix/crates/radix/src/hir/lower/expr.rs
    660 radix/crates/radix/src/semantic/passes/borrow.rs
    614 radix/crates/radix/src/codegen/go/mod.rs
    569 radix/crates/radix/src/hir/nodes.rs
    492 radix/crates/radix/src/parser/mod.rs
    481 radix/crates/radix/src/lexer/token.rs
    477 radix/crates/radix/src/semantic/passes/lint.rs
    456 radix/crates/radix/src/hir/lower/decl.rs
    456 radix/crates/radix/src/codegen/rust/expr/mod.rs
    441 radix/crates/radix/src/codegen/faber/expr.rs
    438 radix/crates/norma/hal/solum.rs
    414 radix/crates/radix/src/hir/lower/mod.rs
    410 radix/crates/radix/src/driver/project.rs
    408 radix/crates/radix/src/semantic/passes/exhaustive.rs
    398 radix/crates/radix/src/semantic/types.rs
    359 radix/crates/norma/hal/arca.rs
    358 radix/crates/radix/src/codegen/go/decl.rs
    355 radix/crates/radix/src/codegen/faber/decl.rs
    352 radix/crates/radix/src/codegen/rust/decl.rs
    327 radix/crates/radix/src/codegen/rust/expr/control.rs
    327 radix/crates/radix/src/codegen/go/expr/collection.rs
    326 radix/crates/radix/src/syntax/visit.rs
    326 radix/crates/radix/src/codegen/rust/expr/collection.rs
    322 radix/crates/radix/src/codegen/go/expr/option.rs
    320 radix/crates/radix/src/codegen/names.rs
    319 radix/crates/radix/src/codegen/mod.rs
    308 radix/crates/radix/src/codegen/go/expr/call.rs
    307 radix/crates/radix/src/codegen/faber/stmt.rs
    306 radix/crates/radix/src/codegen/rust/expr/verte.rs
```

Remaining large files are outside this phase set, already listed as deferred, or are dispatch/orchestration files.

**Second-Pass Verification**:

- `cargo check --manifest-path radix/Cargo.toml -p radix`: PASS
- `bun run ci`: PASS
- `bun run lint`: PASS
- `bunx eslint .`: PASS
- `bun run prettier:check`: PASS
- `bun run build:radix`: PASS
- Representative emission comparison against `8632a5ab` for `faber`, `go`, and `rust`: PASS, no diffs.

**Second-Pass Gate Result**: PASS.

## Second-Pass Commit Record

- Hash: recorded by the commit containing this ledger update
- Message: `refactor: complete expression module split`
- Files: Go/Rust expression submodule completion plus factory ledger and plan updates.
