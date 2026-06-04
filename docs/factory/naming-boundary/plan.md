# Compiler Naming Boundary Factory Plan

**Status**: complete  
**Created**: 2026-06-04  
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`  
**Factory Artifact Dir**: `docs/factory/naming-boundary/`  
**Mode**: staged compiler vocabulary consistency pass  
**Commit Policy**: commit after each completed phase and validation gate pass

## Interpreted Problem

The compiler has an implicit naming boundary. On the early side, AST and much
of HIR preserve source grammar and should use Faber/Latin vocabulary when the
name is directly modeling a source word or source mode. On the later side,
semantic analysis, MIR helpers, validators, and target backends may use English
compiler vocabulary for concepts that are not source spellings.

The `itera ab` cleanup exposed the boundary because `IteraMode::Range` and
`HirIteraMode::Range` were English names inside a source preposition taxonomy
whose sibling variants were Latin (`Ex`, `De`). That has now been corrected to
`Ab`, but the same pattern may exist elsewhere.

This plan makes the boundary explicit and then repairs high-confidence naming
inconsistencies without doing a broad aesthetic rewrite.

## Naming Boundary

### Source-Shaped Side

Use source/Faber vocabulary when a node or enum variant represents a grammar
word, source preposition, source modifier, or source annotation family.

Examples:

- `StmtKind::Si`, `Dum`, `Itera`, `Redde`, `Rumpe`, `Perge`, `Iace`.
- `IteraMode::{Ex, De, Ab}`.
- `HirIteraMode::{Ex, De, Ab}`.
- `HirExprKind::Intervallum`, `Conversio`, `Clausura`, `Cede`.

This side includes:

- `crates/radix/src/syntax/ast.rs`;
- parser-facing enums and parser tests;
- HIR nodes that still preserve source-level grammar intent;
- Faber round-trip codegen decisions that print source spelling.

### Compiler-Shaped Side

Use English compiler vocabulary when the type or helper describes semantic
meaning, data-flow shape, runtime behavior, MIR control flow, target ABI, or
backend implementation strategy rather than a source token.

Examples:

- `RangeIteraLowering` after typecheck proves `ab` contains an interval.
- `MirTerminatorKind::Branch`, `Switch`, `Unreachable`.
- `MirAggregateKind::Struct`, `Array`, `Map`.
- `MirCollectionOp::Append`, `Length`, `Contains`.
- `Type::Array`, `Type::Map`, `Type::Struct`.

English is also acceptable in comments and diagnostics when it explains the
semantic meaning of a Faber source form.

## Non-Goals

- Do not rename every English type in AST/HIR merely because it is English.
- Do not rename user-visible diagnostics unless the diagnostic currently exposes
  the wrong source vocabulary.
- Do not rename MIR node families that already represent backend-neutral
  compiler concepts.
- Do not churn target backend helper names unless they encode source modes.
- Do not combine this with syntax changes, keyword changes, or compatibility
  policy changes.

## Candidate Inconsistency Inventory

These require audit before implementation:

| Surface | Current Naming | Concern | Initial Direction |
| --- | --- | --- | --- |
| Iterator modes | `Ab` after cleanup | now consistent | keep |
| Parameter modes in syntax AST | `ParamMode::{Owned, Ref, MutRef, Move}` | source markers are `de`, `in`, `ex`, plus default | audit first |
| Type modes in syntax AST | `TypeMode::{Ref, MutRef}` | source markers are `de`, `in` in type position | likely rename to source-shaped variants |
| Semantic parameter modes | `semantic::ParamMode::{Owned, Ref, MutRef, Move}` | may be semantic, not syntax | probably keep English |
| HIR parameter modes | `HirParamMode::{Owned, Ref, MutRef, Move}` | HIR may have crossed into semantic passing mode | audit with borrow/typecheck/codegen users |
| Visibility in syntax AST | `Visibility::{Private, Public}` | source words are `privata`, `publica` | audit; may be semantic enough to keep |
| Declaration taxonomy in AST | `Class`, `Interface`, `Enum`, `Union` | source words are `genus`, `pactum`, `ordo`, `discretio` | defer; high churn |
| Declaration structs | `ClassDecl`, `InterfaceDecl`, `EnumDecl`, `UnionDecl` | mixed with Latin statement variants | defer; high churn |
| Range endpoint policy | `RangeKind::{Exclusive, Inclusive}` | source has `ante/usque` and glyphs | keep English semantic policy |
| Optional/non-null chains | `OptionalChainKind`, `NonNullKind` | source operators are symbolic, semantic English is clearer | keep |
| Assignment ops | `AssignOp::AddAssign`, etc. | operator taxonomy, not source word | keep |

## Phase Set

### Phase 0: Boundary Audit And Classification

Write `phase-0-audit.md` with an inventory of source-shaped and
compiler-shaped names in:

- `crates/radix/src/syntax/ast.rs`;
- `crates/radix/src/parser/*`;
- `crates/radix/src/hir/nodes.rs`;
- `crates/radix/src/hir/lower/*`;
- `crates/radix/src/semantic/types.rs`;
- `crates/radix/src/mir/nodes.rs`.

For every candidate, classify it as:

- source-shaped Latin/Faber name;
- compiler-shaped English name;
- intentional bridge;
- inconsistent and safe to rename;
- inconsistent but too broad for this pass.

Checkpoint:

- No code changes.
- The audit identifies a first implementation phase with low churn and clear
  validation.

### Phase 1: Source Mode Enum Cleanup

Target only parser/HIR enums that model source preposition or source marker
modes.

Likely candidates after audit:

- `syntax::ParamMode`;
- `syntax::TypeMode`;
- maybe parser-only aliases in tests or helper names.

Possible direction:

- Replace parser-level `Ref`/`MutRef`/`Move` variants with source-shaped names
  such as `De`, `In`, `Ex` where the enum is truly syntax-level.
- Preserve semantic/HIR passing-mode names if those variants mean compiler
  ownership behavior rather than spelling.

Checkpoint:

- Parser tests prove source mode parsing still works.
- Semantic/typecheck/codegen tests prove downstream behavior is unchanged.
- Public diagnostics still name source forms such as `de`, `in`, and `ex`.

### Phase 2: HIR Bridge Review

Decide whether HIR should carry source-shaped mode names or semantic passing
mode names for parameters and reference-like type positions.

Rules:

- If HIR preserves source syntax so later passes can validate it, use Latin.
- If HIR has already normalized to semantic ownership/reference meaning, use
  English.
- If both are needed, split the concepts rather than overloading one enum.

Checkpoint:

- No missing type facts or backend guesses are introduced.
- MIR and LLVM/Wasm follow-up work still see target-neutral semantics.

### Phase 3: Visibility And Modifier Taxonomy

Audit syntax-level `Visibility`, function modifiers, annotations, and test
modifiers.

Likely approach:

- Keep structured annotation variants in Faber names where they are annotation
  names (`Futura`, `Cursor`, `Publica`, `Privata`).
- Decide whether `Visibility::{Private, Public}` is semantic enough to keep or
  should become source-shaped.
- Avoid renaming declaration types in this phase.

Checkpoint:

- Explain/docs coverage still matches source spelling.
- No keyword or parser behavior changes.

### Phase 4: Declaration Taxonomy Decision

This is a decision phase, not an automatic implementation phase.

Question:

Should syntax AST declaration variants and structs use Faber source taxonomy
(`GenusDecl`, `PactumDecl`, `OrdoDecl`, `DiscretioDecl`) or compiler-standard
taxonomy (`ClassDecl`, `InterfaceDecl`, `EnumDecl`, `UnionDecl`)?

Reasons to defer:

- This touches many files and tests.
- Some English names may intentionally reflect semantic categories after
  lowering, especially HIR `HirStruct` and semantic `Type::Struct`.
- The cost may exceed the readability gain unless there are recurring bugs.

Checkpoint:

- Produce a recommendation and, if warranted, separate delivery specs for each
  declaration family.

## Validation Gates

For any implementation phase:

```bash
cargo test -p radix parser -- --nocapture
cargo test -p radix hir -- --nocapture
cargo test -p radix mir -- --nocapture
cargo test -p radix llvm -- --nocapture
cargo test -p radix wasm -- --nocapture
cargo test -p radix
./scripta/lint
```

Run narrower focused tests first, but do not commit a code rename without at
least `cargo test -p radix` and lint unless the phase explicitly records a
justified deferral.

## Completion Criteria

The consistency pass can stop successfully when:

- the naming boundary is documented and cited by follow-up plans;
- source-mode enum inconsistencies are corrected or explicitly classified;
- high-churn declaration taxonomy is either deferred with rationale or split
  into separate delivery specs;
- no MIR/Wasm/LLVM behavior changes are introduced by naming-only phases;
- tests prove the renamed surfaces are behavior-preserving.

## Initial Recommendation

Start with Phase 0, then Phase 1 if the audit confirms `ParamMode` and
`TypeMode` are parser/source-shaped rather than semantic. Do not start with
declaration taxonomy. That work is too broad to justify before the mode-level
boundary is clean.
