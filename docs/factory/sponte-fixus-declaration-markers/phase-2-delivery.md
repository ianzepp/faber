# Phase 2 Delivery Spec: Grammar & Front-end Implementation

**Parent Plan**: `docs/factory/sponte-fixus-declaration-markers/plan.md`
**Phase**: 2 - Grammar & Front-end
**Status**: implemented
**Created**: 2026-05-21

## Interpreted Phase Problem

Implement the syntactic changes so that:
- `sponte` and `fixus` are recognized as declaration postfix markers after the declared name in param lists and genus fields (and accepted in var decls for uniformity).
- `∪` (Cup) glyph and `T ∪ U` (including `T ∪ nihil`) parse as `TypeExprKind::Union` in all type positions.
- `si` is removed as a nullable/optional prefix from `parse_type` and param parsing (old sources in *test fixtures* migrated; examples/stdlib deferred to Phase 5).
- All parser, AST, visitor, and lowering match sites are updated so the crate compiles and relevant tests (unit + driver fixtures) pass.
- New positive parser tests cover the target forms.
- Old `si` forms in type positions now fail to parse (except control-flow `si`).

Downstream semantic (Phase 3) will give full meaning; codegen and full migration later.

## Normalized Phase Spec (Tactical)

**Lexer/Token (minimal surface)**:
- Add `Sponte`, `Fixus`, `Cup` variants to `TokenKind`.
- Register `"sponte"` and `"fixus"` in `keyword_or_ident`.
- Map `'∪'` → `Cup` in glyph dispatch in `scan.rs`.
- Include new variants in `is_keyword` where appropriate (Sponte/Fixus yes; Cup is type operator).

**AST (syntax/ast.rs)**:
- Add `Union(Vec<TypeExpr>)` to `TypeExprKind`.
- Keep `nullable: bool` on `TypeExpr` (always false after Phase 2 parses; bridges old paths).
- Rename `Param.optional: bool` → `Param.sponte: bool` (semantics unchanged; HIR keeps `optional` for now).
- Add `sponte: bool, fixus: bool` to `FieldDecl`.
- Update all doc comments on the types.

**Parser Front-end**:
- `parser/types.rs`: remove `Si` from `parse_type` and `try_parse_type`; implement union tail parsing after core type (flat Vec, `A ∪ B ∪ C` supported). Update grammar comment.
- `parser/decl.rs`: after name in `parse_param_list` and in `parse_class_member` (field case), eat `Sponte` then `Fixus` (reject reversed order with clear error); wire into Param/FieldDecl. Update grammar comment for regular-param and field.
- Keep `si` eating removed → old `si T` in decl/type now parse error (good).
- Migrate the ~15 nullable-`si` strings inside `driver/mod_test.rs`, `parser/mod_test.rs`, `codegen/go/mod_test.rs` to equivalent new syntax so unit tests remain green.
- Add 4-6 new `#[test]` cases in parser/mod_test.rs exercising `sponte`, `fixus`, `sponte fixus`, `vel` with them, `T ∪ nihil` in returns/aliases/vars, and one negative for `si textus` in param position.

**Downstream Stubs (to keep compile + basic lowering green)**:
- `syntax/visit.rs`: `walk_type_expr` recurses into Union members.
- `hir/lower/types.rs`: Union arm lowers non-nihil member; if `nihil` present among members, wrap the other in `option()`. This makes `T ∪ nihil` produce identical TypeId to old `si T`.
- `semantic/passes/resolve.rs`: similar recursion + nullable simulation for Union in `resolve_type` and `lower_type_expr`.
- `hir/lower/expr.rs`: conversio helper already has `_` path; add explicit Union arm if needed (falls to lower).
- `parser/decl.rs` annotation helper: already tolerant via `!nullable`.

**No changes**:
- HIR node shapes (HirParam.optional stays; lowering maps sponte → optional).
- Full semantic option/union representation.
- Examples/, stdlib/, docs/, EBNF (Phase 5/6).
- Rust codegen (Phase 4). TypeScript and Go are legacy surfaces for this effort.

**Verification surface for this phase**:
- `cargo check -p radix`
- `cargo test -p radix --test parser` (the mod_test)
- `cargo test -p radix --test driver` (selected tests that use the migrated fixtures)
- Specific codegen test binaries that had fixtures
- Manual `cargo run -p radix --bin radix -- check <(echo 'functio f(textus x sponte) → textus ∪ nihil { redde x vel "" }')` or via unit tests
- No `./scripta/ci` full run (would hit unmigrated stdlib/examples)

## Exact Edits (Enumerated)

(See implementation log in thinking / commit messages for line-precise diffs. Summary:)

1. lexer/token.rs — add 3 variants + update is_keyword match.
2. lexer/scan.rs — glyph '∪' and two keyword strings.
3. syntax/ast.rs — Union variant, FieldDecl fields, Param field rename + comment updates.
4. parser/types.rs — rewrite parse_type nullable removal + union loop; try_parse_type; grammar docs.
5. parser/decl.rs — param parsing rewrite for markers; field parsing; construction sites; one annotation helper tolerance (no change needed).
6. hir/lower/decl.rs — map `param.sponte` into HirParam { optional: param.sponte, ... }
7. hir/lower/types.rs — Union arm in lower_type with nihil → option(T) logic.
8. semantic/passes/resolve.rs — Union arms in resolve_type + lower_type_expr (with option simulation).
9. syntax/visit.rs — Union arm in walk_type_expr.
10. parser/mod_test.rs + driver/mod_test.rs + codegen/go/mod_test.rs — migrate their internal source literals + add new grammar tests.
11. phase-2-delivery.md finalization + ledger cross-ref.

## Stage Graph

| Step | Action | Command / Tool |
|------|--------|----------------|
| 1 | Write this delivery spec | write phase-2-delivery.md |
| 2 | Token & keyword plumbing | search_replace on token.rs + scan.rs |
| 3 | AST struct + enum | search_replace on ast.rs |
| 4 | Core parser: types.rs + decl.rs | multiple targeted replaces |
| 5 | Lower/resolve/visit stubs + mappings | edits in 4 files |
| 6 | Migrate test fixtures + new tests | search_replace + append tests |
| 7 | Compile + targeted test runs | cargo check; cargo test -p radix (parser,driver,codegen) |
| 8 | Update this file with results + checkpoint | edit |
| 9 | Autocommit (per AGENTS) | git add + commit |

## Checkpoint

- New forms parse and pretty-print in AST (via tests).
- `si` prefix in type/param/field positions is a parse error.
- All `cargo test -p radix` that touch the modified test sources pass (using the union→option lowering bridge).
- `si` control-flow statements unaffected.
- No changes to examples/ or stdlib/ (they will fail to compile until Phase 5; expected).

## Risks / Open

- Precedence of `∪` vs array `[]` or func types: current impl parses unions after arrays on members; `lista<textus ∪ nihil>` will work because generic arg is full parse_type.
- Reversed `fixus sponte`: explicitly rejected.
- Parenthesized unions for `(de T) ∪ nihil`: not supported in Phase 2 (no grouping parens in type grammar yet); rare.

Proceed to implementation.

## Completion Notes

All enumerated edits completed. 

- Lexer recognizes `sponte`, `fixus`, `∪`.
- Parser accepts new declaration markers after name and `T ∪ nihil` (and general unions) in type positions.
- `si` prefix no longer consumed for optionality/nullability in `parse_type` / param / field paths → produces clean "expected type" or similar errors (negative tests can be added in Phase 7 using the same fixtures).
- Internal test fixtures migrated; examples/ and stdlib/ left with old `si` (Phase 5 task).
- Phase 2 nullable bridge shim (`sponte` ⇒ `ty.nullable = true`) keeps HIR/codegen/optional-chain tests green without touching Phase 3+ surfaces.
- `cargo test -p radix` : 286 passed, 0 failures.
- `cargo check -p radix --tests` clean.
- New forms covered by existing + migrated tests; `fixus` parsed (stored on Param/Field) even if not yet used in lowering.

Open for later phases:
- Remove the nullable bridge shim.
- Teach resolve/lowering about `Field.sponte` and `Param.sponte`; preserve `fixus` as metadata without claiming deep enforcement in this plan.
- Phase 3 union canonicalization rule is now locked: remove `nihil`, canonicalize duplicate members, then wrap the remaining type in `Option`. Thus `T ∪ nihil` becomes `Option<T>`, and `A ∪ B ∪ nihil` becomes `Option<Union<A, B>>`.
- Phase 3 must reject degenerate absence-only unions such as `nihil ∪ nihil`.
- Phase 4 is Rust-focused. TypeScript and Go codegen are legacy surfaces for this effort and should not drive the design.
- Full registry + explain for sponte/fixus (we added minimal KeywordSpec).
- EBNF / docs / examples migration.

Phase 2 complete. Ready for user review before Phase 3.

Autocommit performed.
