# Radix Large Module Refactor Factory Ledger

**Factory Run**: radix-large-module-refactor per `docs/radix-large-module-refactor-factory-plan.md`
**Started**: 2026-05-20
**Target Repo**: /Users/ianzepp/work/ianzepp/faber (radix primary)
**Phase Set Source**: docs/radix-large-module-refactor-factory-plan.md (factory-approved)
**Delivery Spec Dir**: docs/factory/radix-large-module-refactor/
**Commit Policy**: Commit after each phase completion + validation gate pass
**Agent Policy**: Use subagents for exploration, poker-face, verification, bounded impl where appropriate; main agent supervises and integrates
**Checkpoint Policy**: Full validation gate after every phase; phase-specific smoke tests
**Current Status**: Phase 3 complete in working tree (gate PASS); ready for commit

## Baseline (Phase 0 Intake)

- HEAD: 7619f4f3 (docs: clarify radix refactor extraction strategy)
- Branch: main (ahead of origin/main by 1)
- Working tree: clean
- Validation gate baseline run: in progress (see below)

## Phases

| Phase | Name | Status | Commit | Delivery Spec |
|-------|------|--------|--------|---------------|
| 0 | Preflight and delivery-spec setup | completed | a0e3838c | N/A (ledger + Phase 1 spec) |
| 1 | Split Faber codegen | completed | c4368940 | `phase-01-faber-codegen-delivery.md` |
| 2 | Split typecheck pass | completed | 4ee6a5e8 | `phase-02-typecheck-delivery.md` |
| 3 | Split Go expression codegen | completed | pending | `phase-03-go-expr-delivery.md` |
| 4 | Split Rust expression codegen | pending | pending | `phase-04-rust-expr-delivery.md` |
| 5 | Documentation and final hygiene review | pending | pending | `phase-05-docs-hygiene-delivery.md` |

## Validation Gate Log

**Baseline run started at Phase 0**:

- `bun run check:radix`: PASS (fmt clean)
- `bun run lint`: PASS (0.29s, cached)
- `bun run prettier:check`: PASS
- `bunx eslint .`: PASS (clean)
- `bun run test:radix`: PASS (full suite + hygiene + doctests ok, fast due to cache)
- `bun run build:radix`: PASS (release build cached, 0.03s)
- **FULL BASELINE VALIDATION GATE: PASS** (all components green at HEAD 39707302)
- `bunx eslint .`: not yet
- `bun run prettier:check`: not yet
- `bun run build:radix`: not yet
- `bun run test:radix` (via ci): not yet

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

**Phase 3 Gate Result**: PASS. Ready to commit with message `refactor: split go expression codegen`.
