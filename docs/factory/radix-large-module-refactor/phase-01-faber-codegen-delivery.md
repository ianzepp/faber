# Phase 1 Delivery Spec: Split Faber Codegen

**Phase**: 1 of 6 in the now-deleted `radix-large-module-refactor-factory-plan.md`
**Goal**: Reshape `codegen/faber/mod.rs` (1804 LOC) into the established multi-file pattern used by Go/Rust/TS codegen backends. No behavior change.
**Owner**: Factory (main agent + slice + subagents)
**Inputs**: Current `radix/crates/radix/src/codegen/faber/mod.rs`, `mod_test.rs`, usages in `codegen/mod.rs`
**Outputs**: New directory tree per target shape, `mod.rs` reduced to orchestration, all tests and generation identical.
**Out of Scope**: Changing public `FaberCodegen` API surface, any HIR or semantic changes, other codegen backends, docs updates (Phase 5).

## Interpreted Problem (Scoped to Phase 1)

The Faber canonical pretty-printer lives in a single 1804-line `mod.rs`. It mixes:
- Orchestration (`impl Codegen`, `generate`, name collection)
- Item/decl rendering (functions, structs, enums, interfaces, consts, imports, typus)
- Stmt and control flow (blocks, si/sin/secus, itera, fac, cura, tempta/cape, dum, elige/discerne)
- Expr dispatch with precedence (`write_expr`, `write_expr_prec`)
- Pattern, type, literal, name, and op helpers

This matches the "large module" anti-pattern identified in housekeeping. Other backends (see `codegen/go/`, `codegen/rust/`, `codegen/ts/`) already use `mod.rs` + `decl.rs`/`stmt.rs`/`expr.rs`/etc submodules. Faber is the outlier.

**Success**: `mod.rs` drops below ~400 LOC of pure orchestration + re-exports; each new file owns one cohesive responsibility; `cargo test -p radix` and round-trip Faber emission unchanged.

## Normalized Spec

1. Introduce `radix/crates/radix/src/codegen/faber/{decl,stmt,expr,pattern,types,literal,names,ops}.rs`
2. Move cohesive method groups into `impl FaberCodegen` blocks inside each submodule (or free fns + use).
3. `mod.rs`:
   - Keeps `pub struct FaberCodegen`, `pub fn new()`, `impl Default`, `impl Codegen for FaberCodegen { generate }`
   - Declares `mod decl; mod stmt; ...`
   - Re-exports or keeps necessary items pub(crate) for test compatibility via `super::`
   - Owns top-level `collect_names`, `generate_item` dispatch (thin), `generate_function` etc if orchestration, or delegates.
4. Preserve exact output bytes for all existing tests in `mod_test.rs` and any snapshot tests.
5. Use `pub(super)` for cross-submodule items inside the faber module.
6. Prefer method-anchored moves over raw line ranges. If `slice` is used, move complete methods only after confirming attached docs, attributes, signatures, and closing braces are included.
7. After split: run full validation gate + Faber-specific smoke (e.g. `cargo run -p radix -- emit -t faber examples/...`).

## Restart Lessons and Extraction Rules

The first Phase 1 attempt was abandoned before commit. It kept behavior mostly intact, but it left empty target modules, stale imports, orphaned comments, and a failing lint/format gate. The failure mode was not semantic complexity; it was brittle line-range movement around Rust method boundaries.

For the restart, use this discipline:

- Re-anchor every move on method names, not remembered line numbers.
- Treat `///` doc comments and `#[allow(...)]` attributes as part of the method they describe.
- Move one complete method or one tightly coupled helper cluster at a time.
- After each move, immediately remove stale imports from the source file and add only required imports to the target file.
- Run `cargo fmt --manifest-path radix/Cargo.toml --all` after each batch before interpreting compiler output.
- Run `cargo check --manifest-path radix/Cargo.toml -p radix` after each target module lands.
- Run `bun run lint` before any checkpoint; warnings are blockers in this repo.
- Do not leave empty target modules with speculative imports. Create a target file when the first real method lands there.

If a method has a clippy allowance such as `#[allow(clippy::too_many_arguments)]` or `#[allow(clippy::only_used_in_recursion)]`, move the allowance with that method. If moving the method removes the original lint trigger from `mod.rs`, also remove any now-stale allowance or comment from `mod.rs`.

## Repo-Aware Baseline (Current State)

**Files**:
- `radix/crates/radix/src/codegen/faber/mod.rs`: 1804 LOC (single `impl FaberCodegen` ~1720 LOC of methods)
- `radix/crates/radix/src/codegen/faber/mod_test.rs`: ~550 LOC of unit tests using `super::*`
- `radix/crates/radix/src/codegen/mod.rs:119`: `faber::FaberCodegen::new()`

**Key call sites / public surface** (must remain stable):
- `FaberCodegen::new()`
- `impl Codegen for FaberCodegen` (the `generate` method)
- Internals are private; no external crates depend on non-pub details.

**Current responsibilities mapping** (from source inspection):
- Lines ~57-100+: item dispatch + generate_function, generate_struct, generate_enum, generate_interface, type aliases, consts, imports
- Stmt logic: write_block, write_stmt, si/sin/secus, loops (itera/fac), cura, tempta, dum, elige, discerne
- Expr: write_expr, write_expr_prec + precedence table, many expr kind arms
- Helpers: type_to_faber, symbol_to_string, collect_names, write_literal, write_pattern, write_object_field, binary/unary/assign ops, precedence

**Existing module pattern in siblings** (for consistency):
- Look at `codegen/go/mod.rs` structure, `go/decl.rs`, `go/stmt.rs`, `go/expr.rs` etc. for how `pub(super)` and re-use of CodeWriter works.

**Test surface**: All 20+ tests in mod_test.rs must continue to pass without modification to the test file itself (only super imports may need `pub use` in mod.rs).

**Risks**:
- Precedence and associativity bugs in expr split (high)
- Name collection (DefId map) must be available to all submodules
- CodeWriter usage is shared; keep in parent or re-export
- Any `self.` method calls that cross boundaries need `pub(super)` or moved together

## Stage Graph / Workstreams for Phase 1

1. **Exploration & boundary definition** (read-only subagent or main): map methods, attached attributes/docs, and internal call dependencies for each target module. Produce method-name extraction lists, not only line ranges.
2. **Target module implementation**: create each target `.rs` file only when moving its first real method. Do not create placeholder modules with speculative imports.
3. **Method-anchored extractions**:
   - `ops.rs`: `expr_precedence`, `binop_to_faber`, `binop_precedence`, `assignop_to_faber`, `unop_to_faber`
   - `types.rs`: `type_to_faber`, `flatten_option`
   - `literal.rs`: `write_literal`, `write_object_field`, quoted text/symbol helpers if they are not better kept with names
   - `names.rs`: `collect_names`, name collectors, `name_for_def`, `symbol_to_string`
   - `pattern.rs`: `write_pattern`, `write_match_arms`/match-arm helpers if present
   - `expr.rs`: `write_expr`, `write_expr_prec`, expression-only helpers
   - `stmt.rs`: `write_block`, `write_stmt`, control-flow statement helpers, branch body helpers, loop-shape helpers, `reddit_expr` if used by statement rendering
   - `decl.rs`: `generate_item`, declarations, functions, proba functions, structs, enums, interfaces, `is_synthetic_proba_function`
4. **Glue edits**:
   - Add `mod decl; mod stmt; ...` declarations
   - Adjust visibility on moved items (`pub(super)`)
   - Add necessary `use` in submodules and in mod.rs
   - Ensure `CodeWriter`, `CodegenError`, HIR types visible (they come from super:: or crate::)
5. **Compile + test loop**:
   - After each method batch: `cargo fmt --manifest-path radix/Cargo.toml --all`
   - After each target module: `cargo check --manifest-path radix/Cargo.toml -p radix`
   - Before checkpoint: `bun run lint`
   - Full `cargo test -p radix --test faber_codegen` or the mod_test
   - Fix any import/visibility/scope issues immediately
6. **Faber-specific smoke**: Use CLI `cargo run -p radix -- emit -t faber <example.fab>` and roundtrip check if available.
7. **Full validation gate**: per master plan.

**Parallelism**: Feasible, but only with clear ownership. Subagents may own target files or read-only method maps. The parent agent serializes edits to `mod.rs` and owns the final integration order.

Suggested 5.4-mini ownership:

- Agent A: `ops.rs`, `types.rs`, `literal.rs` method map and implementation proposal
- Agent B: `names.rs` and `pattern.rs` method map and implementation proposal
- Agent C: `expr.rs` method map, dependencies, and post-move verification checklist
- Agent D: `stmt.rs` method map, dependencies, and post-move verification checklist
- Agent E: `decl.rs` method map, dependencies, and final public-surface verification

Each agent must report exact method names, attached attributes/docs, imports needed in its target file, and cross-file method calls. Do not let multiple agents delete from `mod.rs` concurrently.

## Checkpoints & Gate

**Phase Checkpoint** (from master plan):
- no behavior changes intended
- `codegen/faber/mod.rs` becomes orchestration-sized (< ~400 LOC ideal)
- Faber codegen tests still pass
- full validation gate passes (bun run ci, lint, eslint, prettier:check, build:radix)

**Specific smoke for this phase**:
- `cargo test -p radix -- --quiet 2>&1 | tail -10` (focus on faber tests)
- Manual: `cargo run -p radix -- emit -t faber examples/exempla/salve-munde.fab`

**Poker-face target**: >= 90% completion against this spec (no missing submodule, no output diff in tests, no public API breakage).

## Companion Skills

- `slice`: primary for all bulk moves of method bodies and helper fns
- `poker-face`: post-impl completion audit before commit
- `check` (or verifier subagent): for final diff + gate review if desired
- Explorer subagents: for initial line-range identification and call-graph of internal methods

## Open Questions for Phase 1

- Exact line ranges for each extraction (will be determined live with nl/rg before first slice)
- Whether some small helpers (e.g. symbol_to_string) stay in mod.rs or move to names.rs
- Whether `generate_item` dispatch stays in mod.rs or moves to decl.rs (plan says mod.rs keeps top-level orchestration)
- Any private structs/types that need to be hoisted or duplicated? (Expect none; use pub(super))

## Spec Revisions

- 2026-05-20: Added restart lessons after the abandoned first Phase 1 attempt. The restart should use method-anchored moves, keep attributes/docs attached to methods, avoid placeholder modules, and use parallel subagents only for target-file ownership or read-only mapping while `mod.rs` integration remains serialized.

**Status**: Ready for implementation. Delivery spec persisted before any edits.
