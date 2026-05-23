# Radix Documentation Factory Plan

## Objective

Apply the Faber documentation methodology to `crates/radix` while excluding test
files. The goal is not comment volume. The goal is to make Radix easier to
understand, maintain, and modify by documenting the semantic load-bearing parts
of the compiler: file-level abstracts, public and crate-facing contracts,
cross-phase invariants, diagnostic policy, codegen trade-offs, and edge cases.

Use the freshly documented `crates/faber` crate as the live style reference,
especially:

- `crates/faber/src/package.rs` for file headers that read like abstracts.
- `crates/faber/src/explain.rs` for public data contracts and taxonomy docs.
- `crates/faber/src/library.rs` for boundary/invariant-focused module docs.
- `crates/faber/src/explain_render.rs` for separating data policy from rendering
  policy.

## Documentation Method

Use the `docs` skill.

Rules to enforce in every phase:

- Documentation density follows semantic weight, not line count.
- File headers should explain the file's story: purpose, architecture role,
  why it exists as its own file, implementation decisions, invariants, hazards,
  and compatibility constraints.
- Public and crate-facing types/functions should describe contracts,
  invariants, error behavior, and phase/system context.
- Private helpers stay compact unless they encode policy, edge cases, or
  non-obvious trade-offs.
- Comments explain why, constraints, or recovery context. They do not narrate
  simple Rust mechanics.
- Put doc comments before field-specific attributes.
- Use blank lines between documented fields/variants; keep private schemas
  compact when they have no field docs.
- Do not touch test files: exclude `*_test.rs`, `*test.rs`, and `/tests/`.
- Preserve behavior. This is a documentation and formatting pass only.

## Factory Ledger

- Phase Set Source: this plan.
- Target Repo: `/Users/ianzepp/work/ianzepp/faber`.
- Target Tree: `crates/radix`.
- Delivery Spec Directory: `docs/radix-docs-factory/`.
- Current Phase: phase 6 pending.
- Completed Phases: phases 1-5.
- Pending Phases: phases 6-10 below.
- Checkpoint Policy: each phase must have a focused diff, `cargo fmt --check`,
  `cargo test -p radix`, and `git diff --check` passing before commit.
- Commit Policy: commit after each completed phase.
- Agent Policy: use bounded subagents on small disjoint file groups inside the
  active phase. Subagents must use the `docs` skill and Faber crate as reference,
  must not touch tests, and must not stage/commit.
- Correctness Policy: docs-only changes; correctness review checks behavior is
  unchanged and comments do not assert unsupported facts.
- Poker Face Policy: before each phase commit, compare changed files against the
  phase boundary and require at least 85% completion.
- Granularity Policy: phases are grouped by compiler subsystem and should not
  mix unrelated ownership boundaries.
- Open Questions: none.

## Standard Subagent Prompt

Use this prompt shape for implementation agents, filling in the phase and file
list:

```text
In /Users/ianzepp/work/ianzepp/faber, apply the updated docs methodology to the
listed crates/radix files only. You are not alone in the codebase; other workers
may be editing different files, so do not modify or revert files outside your
ownership. Omit test files.

Use the docs skill perspective and use crates/faber as the live reference:
- package.rs for strong file-header abstracts and invariants
- explain.rs for public data contracts and taxonomy docs
- library.rs for boundary/invariant module docs

Focus on semantic density and file-header quality. Add or improve module/file
headers where they do not tell the file's story. Document public or
crate-facing contracts and high-semantic-weight types/functions. Keep obvious
private helpers compact. Put doc comments before attributes. Use blank lines
between documented fields/variants, not between every private field by default.

Keep behavior unchanged. Do not touch tests. Run cargo fmt --check if feasible.
Do not run git stage/commit commands. Final response: list changed files,
summarize documentation changes, and report validation run.
```

## Phase 1: Crate Surface, CLI, Driver, Diagnostics

Goal: document the user/developer command surface and diagnostic boundary before
inner compiler phases.

Files:

- `crates/radix/src/lib.rs`
- `crates/radix/src/bin/radix.rs`
- `crates/radix/src/cli.rs`
- `crates/radix/src/tool.rs`
- `crates/radix/src/driver/mod.rs`
- `crates/radix/src/driver/source.rs`
- `crates/radix/src/driver/session.rs`
- `crates/radix/src/diagnostics/mod.rs`
- `crates/radix/src/diagnostics/diagnostic.rs`
- `crates/radix/src/diagnostics/catalog.rs`
- `crates/radix/src/diagnostics/render.rs`

Checkpoint: a reader can understand the public Radix API boundary, CLI IR,
driver/session ownership, and diagnostic rendering/catalog policy from headers
and public contract docs.

## Phase 2: Lexer, Parser, Syntax

Goal: document source-to-AST front-end phases and their invariants.

Files:

- `crates/radix/src/lexer/mod.rs`
- `crates/radix/src/lexer/scan.rs`
- `crates/radix/src/lexer/cursor.rs`
- `crates/radix/src/lexer/token.rs`
- `crates/radix/src/lexer/keywords.rs`
- `crates/radix/src/parser/mod.rs`
- `crates/radix/src/parser/decl.rs`
- `crates/radix/src/parser/expr.rs`
- `crates/radix/src/parser/stmt.rs`
- `crates/radix/src/parser/types.rs`
- `crates/radix/src/parser/pattern.rs`
- `crates/radix/src/parser/error.rs`
- `crates/radix/src/syntax/mod.rs`
- `crates/radix/src/syntax/ast.rs`
- `crates/radix/src/syntax/span.rs`
- `crates/radix/src/syntax/visit.rs`

Checkpoint: headers and comments describe tokenization, parse recovery/error
strategy, AST shape, span/node-id invariants, and visitor traversal role without
over-commenting grammar mechanics.

## Phase 3: HIR Model and Lowering

Goal: document the AST-to-HIR boundary and what semantic assumptions begin at
HIR.

Files:

- `crates/radix/src/hir/mod.rs`
- `crates/radix/src/hir/nodes.rs`
- `crates/radix/src/hir/visit.rs`
- `crates/radix/src/hir/lower/mod.rs`
- `crates/radix/src/hir/lower/decl.rs`
- `crates/radix/src/hir/lower/expr.rs`
- `crates/radix/src/hir/lower/stmt.rs`
- `crates/radix/src/hir/lower/types.rs`
- `crates/radix/src/hir/lower/pattern.rs`

Checkpoint: the phase boundary from syntactic AST to compiler-friendly HIR is
recoverable from file headers and load-bearing type/function docs.

## Phase 4: Semantic Core and Non-Typecheck Passes

Goal: document semantic pipeline orchestration, shared state, diagnostics, and
non-typecheck passes.

Files:

- `crates/radix/src/semantic/mod.rs`
- `crates/radix/src/semantic/error.rs`
- `crates/radix/src/semantic/scope.rs`
- `crates/radix/src/semantic/types.rs`
- `crates/radix/src/semantic/passes/mod.rs`
- `crates/radix/src/semantic/passes/collect.rs`
- `crates/radix/src/semantic/passes/resolve.rs`
- `crates/radix/src/semantic/passes/borrow.rs`
- `crates/radix/src/semantic/passes/exhaustive.rs`
- `crates/radix/src/semantic/passes/lint.rs`

Checkpoint: semantic pass order, pass dependencies, error behavior, type-table
role, scope invariants, and warnings/errors policy are clear at high-value
points.

## Phase 5: Semantic Typecheck

Goal: document the typecheck subsystem as a high-semantic-weight hot path.

Files:

- `crates/radix/src/semantic/passes/typecheck/mod.rs`
- `crates/radix/src/semantic/passes/typecheck/collect.rs`
- `crates/radix/src/semantic/passes/typecheck/infer.rs`
- `crates/radix/src/semantic/passes/typecheck/lookup.rs`
- `crates/radix/src/semantic/passes/typecheck/ops.rs`
- `crates/radix/src/semantic/passes/typecheck/call.rs`
- `crates/radix/src/semantic/passes/typecheck/item.rs`
- `crates/radix/src/semantic/passes/typecheck/expr.rs`
- `crates/radix/src/semantic/passes/typecheck/stmt.rs`
- `crates/radix/src/semantic/passes/typecheck/control.rs`
- `crates/radix/src/semantic/passes/typecheck/aggregate.rs`
- `crates/radix/src/semantic/passes/typecheck/access.rs`
- `crates/radix/src/semantic/passes/typecheck/convert.rs`
- `crates/radix/src/semantic/passes/typecheck/pattern.rs`
- `crates/radix/src/semantic/passes/typecheck/finalize.rs`

Checkpoint: type inference/checking contracts, unknown/nullability policy,
call/access/control-flow checks, and finalization responsibilities are
documented where future maintainers would otherwise reconstruct behavior.

## Phase 6: MIR

Goal: document the middle IR model, HIR-to-MIR lowering, validation, dumping,
visitor semantics, and temporary Rust probe boundary.

Files:

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

Checkpoint: MIR's role below typed HIR and above target codegen is clear, with
special attention to storage/order invariants, control-flow representation, and
why the Rust probe is temporary.

## Phase 7: Codegen Shared and TypeScript Backend

Goal: document shared codegen contracts and the TypeScript backend.

Files:

- `crates/radix/src/codegen/mod.rs`
- `crates/radix/src/codegen/names.rs`
- `crates/radix/src/codegen/writer.rs`
- `crates/radix/src/codegen/ts/mod.rs`
- `crates/radix/src/codegen/ts/decl.rs`
- `crates/radix/src/codegen/ts/expr.rs`
- `crates/radix/src/codegen/ts/stmt.rs`
- `crates/radix/src/codegen/ts/types.rs`

Checkpoint: target abstraction, naming policy, writer behavior, and TS mapping
trade-offs are clear without duplicating every syntax conversion.

## Phase 8: Rust Backend

Goal: document the primary backend's contracts and high-weight expression/type
translation decisions.

Files:

- `crates/radix/src/codegen/rust/mod.rs`
- `crates/radix/src/codegen/rust/cli.rs`
- `crates/radix/src/codegen/rust/decl.rs`
- `crates/radix/src/codegen/rust/stmt.rs`
- `crates/radix/src/codegen/rust/types.rs`
- `crates/radix/src/codegen/rust/failable.rs`
- `crates/radix/src/codegen/rust/expr/mod.rs`
- `crates/radix/src/codegen/rust/expr/access.rs`
- `crates/radix/src/codegen/rust/expr/block.rs`
- `crates/radix/src/codegen/rust/expr/call.rs`
- `crates/radix/src/codegen/rust/expr/collection.rs`
- `crates/radix/src/codegen/rust/expr/control.rs`
- `crates/radix/src/codegen/rust/expr/convert.rs`
- `crates/radix/src/codegen/rust/expr/format.rs`
- `crates/radix/src/codegen/rust/expr/literal.rs`
- `crates/radix/src/codegen/rust/expr/ops.rs`
- `crates/radix/src/codegen/rust/expr/option.rs`
- `crates/radix/src/codegen/rust/expr/pattern.rs`
- `crates/radix/src/codegen/rust/expr/verte.rs`

Checkpoint: Rust backend docs emphasize failable propagation, ownership/borrow
mapping, CLI emission, optional/null handling, stdlib translation, and where
the backend is intentionally target-specific.

## Phase 9: Go Backend

Goal: document Go target trade-offs and how Faber semantics are approximated.

Files:

- `crates/radix/src/codegen/go/mod.rs`
- `crates/radix/src/codegen/go/decl.rs`
- `crates/radix/src/codegen/go/stmt.rs`
- `crates/radix/src/codegen/go/types.rs`
- `crates/radix/src/codegen/go/expr/mod.rs`
- `crates/radix/src/codegen/go/expr/access.rs`
- `crates/radix/src/codegen/go/expr/call.rs`
- `crates/radix/src/codegen/go/expr/collection.rs`
- `crates/radix/src/codegen/go/expr/control.rs`
- `crates/radix/src/codegen/go/expr/convert.rs`
- `crates/radix/src/codegen/go/expr/literal.rs`
- `crates/radix/src/codegen/go/expr/ops.rs`
- `crates/radix/src/codegen/go/expr/option.rs`
- `crates/radix/src/codegen/go/expr/variants.rs`

Checkpoint: Go backend comments explain target compromises such as error
returns, pointer optionals, interface-based variants, lack of borrow semantics,
and control-flow lowering.

## Phase 10: Faber Canonical Backend

Goal: document the source-preserving/canonical pretty-printer backend.

Files:

- `crates/radix/src/codegen/faber/mod.rs`
- `crates/radix/src/codegen/faber/names.rs`
- `crates/radix/src/codegen/faber/ops.rs`
- `crates/radix/src/codegen/faber/decl.rs`
- `crates/radix/src/codegen/faber/expr.rs`
- `crates/radix/src/codegen/faber/stmt.rs`
- `crates/radix/src/codegen/faber/types.rs`
- `crates/radix/src/codegen/faber/literal.rs`
- `crates/radix/src/codegen/faber/pattern.rs`

Checkpoint: docs explain why canonical Faber generation exists, what it can and
cannot preserve, grammar-valid output policy, and how names/types/operators are
kept inside real Faber syntax.

## Phase Execution Template

For each phase:

1. Save a delivery spec at
   `docs/radix-docs-factory/phase-XX-<slug>-delivery.md`.
2. Spawn bounded subagents only for disjoint file groups in that phase.
3. Review the diff manually from the factory supervisor context.
4. Remove unsupported or decorative comments.
5. Run:

   ```bash
   cargo fmt --check
   cargo test -p radix
   git diff --check
   ```

6. Run a poker-face completion check against the phase checkpoint.
7. Commit with a phase-specific message.

## Completion Definition

The factory run is complete when all non-test Rust files under `crates/radix`
have been reviewed under this methodology, every phase has a saved delivery
spec, every phase checkpoint has passed, and all phase commits are present in
git history.
