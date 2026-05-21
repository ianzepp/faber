# Sponte/Fixus Declaration Markers — Phase 1 Inventory Ledger

**Parent Plan**: `docs/factory/sponte-fixus-declaration-markers/plan.md`
**Phase**: 1 - Inventory
**Last Updated**: 2026-05-21
**Status**: complete (for phases 0-2 scope)

## Summary

This ledger classifies every site where `si` currently functions as a nullable/optional marker (declaration optionality or value-type nullability) rather than the control-flow keyword. `si` as `if`/`sin`/`secus` is explicitly out of scope for migration and must continue to parse and execute unchanged.

**Total affected source surfaces** (excluding generated target/, plan itself, and pure control-flow uses):
- 14 example .fab files (some files contain both control and nullable uses)
- 7 stdlib .fab files (HAL contracts + json/processus)
- 4 documentation files
- 5 Rust test / driver / codegen-test files containing embedded Faber source
- Internal AST + 3 compiler passes that consume the `nullable` flag

## Classification of `si` Nullable/Optional Uses

### 1. Function / Method Parameter Optionality (`si` before type or mode in param list)

| File | Lines | Form | Notes |
|------|-------|------|-------|
| crates/radix/src/driver/mod_test.rs | 706, 1983, 1990, 1994 | `si bivalens formal`, `si textus titulus`, `si numerus pagina vel 1`, `de si numerus depth` | Parser tests + driver tests. Mixed with `de`, `vel`, `ut`. |
| crates/radix/src/parser/mod_test.rs | 517 | `si de textus nomen ut alias vel "anon"` | Complex param with pre + mode + alias + default. |
| examples/exempla/functio/optionalis.fab | 10,18,23,31 | `si textus titulus`, `si numerus pagina vel 1`, `de si numerus depth`, `si numerus aetas vel 0`, `si bivalens activus vel verum` | Teaching example for optional params. |
| examples/exempla/si/ergo-redde.fab | 17,32 | `→ si numerus` (return, not param) | See returns below. |
| stdlib/norma/json.fab | 27 | `si numerus indentum` | @verte param on pange. |
| stdlib/norma/hal/caelum.fab | 25,29,37,41,67,71 | `si textus cert`, `si textus key`, `si bivalens tute vel falsum`, `si numerus n` | @externa/@futura pactum methods; some with vel defaults. |
| stdlib/norma/hal/processus.fab | 45,46,62,63 | `si textus directorium, si tabula<...> ambitus` | Inside functio decls (likely params). |
| stdlib/norma/hal/http.fab | 106,131,134 | returns only (see §3) | params here are non-si. |
| stdlib/norma/hal/pressura.fab | 23,35 | `si numerus nivel` (param + return) | Compressor HAL. |
| stdlib/norma/hal/thesaurus.fab | 24 | return only | |

Migration action: After name, emit `sponte` (and keep `vel` where present). `de si` ordering becomes `de T name sponte`.

### 2. Return Type Nullability (`→ si T`)

| File | Lines | Form | Notes |
|------|-------|------|-------|
| examples/exempla/si/ergo-redde.fab | 17,32 | `→ si numerus` | Return of divide / findFirst. |
| examples/exempla/functio/optionalis.fab | 10,18,23,31 | returns are non-nullable in this file | - |
| docs/grammatica/functiones.md | 76,92,417 | `→ si numerus`, `→ textus` (with si params) | Duplicates the optinalis examples. |
| docs/grammatica/structurae.md | 197 | `→ si numerus` | In sequens example. |
| stdlib/norma/hal/thesaurus.fab | 24 | `→ si textus` | capiet |
| stdlib/norma/hal/http.fab | 106,131,134 | `→ si textus` | caput / param on Rogatio & Pactum |
| stdlib/norma/hal/pressura.fab | 35 | `→ Compressor` (non-si return of si param fn) | - |
| stdlib/norma/hal/caelum.fab | 25+ | returns non-si | - |

Migration action: `→ T ∪ nihil` (pure type position). Also update EBNF and grammatica.

### 3. Local Variable Declarations (`fixum`/`varia si T name`)

| File | Lines | Form | Notes |
|------|-------|------|-------|
| crates/radix/src/driver/mod_test.rs | 1007,1031,1032,1051,1052,1372,1395,1722,1748,1909 | `fixum si textus maybe ← nihil`, `fixum si User maybeUser ← nihil`, `fixum si numerus maybe ← nihil` | Many "maybe" patterns testing nihil/vel/optionals. |
| examples/exempla/vel/vel.fab | 12,17,36-38 | same maybe patterns | vel operator teaching file. |
| examples/exempla/binarius/binarius.fab | 231,236,237 | `fixum si textus maybeName ← nihil` etc. | Binarius (binary?) examples. |
| examples/exempla/unarius/unarius.fab | 23,27 | `fixum si textus maybe ← nihil` | Unarius examples. |
| examples/exempla/est/est.fab | 9,24 | `fixum si numerus maybeValue ← nihil`, `fixum si textus name ← nihil` | est operator file. |
| examples/exempla/qua/qua.fab | 33 | `num ⇢ si numerus` (postfix conversion, not var decl) | See §5. |
| examples/exempla/ternarius/ternarius.fab | 29 | `nihil ⇢ si textus` (postfix) | See §5. |

Migration action: For nullable locals use `fixum (textus ∪ nihil) name ← nihil` or `fixum textus ∪ nihil name ← nihil`. `sponte` on locals is not semantically meaningful (declaration is always performed); parser may still accept `name sponte` for uniformity or reject it.

### 4. Genus Field Declarations (`si T name` inside genus)

| File | Lines | Form | Notes |
|------|-------|------|-------|
| examples/exempla/optionalis/optionalis.fab | 11,16 | `si textus state`, `si Address address` | Classic "optional field" example for optional chaining. |

Migration action: `textus state sponte`, `Address address sponte`. This is the motivating visual-alignment case from the plan.

### 5. Type Alias / Typus Nullables

| File | Lines | Form | Notes |
|------|-------|------|-------|
| examples/exempla/typus/typus.fab | 16 | `typus OptionalName = si textus` | Explicit optional alias teaching. |

Migration action: `typus OptionalName = textus ∪ nihil`

### 6. Postfix Type Conversion / Cast Sites (`⇢ si T`)

| File | Lines | Form | Notes |
|------|-------|------|-------|
| examples/exempla/qua/qua.fab | 33 | `num ⇢ si numerus` | Tests qua/innatum/novum removal surface (now ⇢). |
| examples/exempla/ternarius/ternarius.fab | 29 | `maybe ← nihil ⇢ si textus` | Ternary / conversion example. |
| crates/radix/src/codegen/go/mod_test.rs | 347,360 | same `⇢ si textus` | Go codegen fixtures. |

Migration action: `num ⇢ (numerus ∪ nihil)` — the union syntax must be parenthesized or have appropriate precedence in type position after ⇢ .

### 7. Documentation & Grammar Surfaces (non-code)

| File | Context | Migration |
|------|---------|-----------|
| EBNF.md:204-206 | "`si` prefix marks nullable types: `si textus` = nullable string" + combined `si de textus` | Rewrite section for new declaration markers and `∪ nihil` value syntax. |
| docs/grammatica/functiones.md | Multiple copies of optionalis examples + return si | Update examples and prose. |
| docs/grammatica/structurae.md:197 | `sequens() → si numerus` | Update. |
| crates/radix/src/codegen/go/mod_test.rs (also has source strings) | Fixture programs | Will need update when we migrate test sources in Phase 5, or keep as negative "old syntax" tests in Phase 7. |

### 8. Internal Compiler Surfaces (AST + Passes) — Phase 3 Impact

| Location | Use of `si` / nullable |
|----------|------------------------|
| crates/radix/src/syntax/ast.rs:1070 | `TypeExpr { nullable: bool, ... }` — the flag populated by parser today. |
| crates/radix/src/parser/types.rs:81 | `let nullable = self.eat_keyword(TokenKind::Si);` — primary site to delete. |
| crates/radix/src/parser/decl.rs:308 | `let optional = self.eat_keyword(TokenKind::Si);` for params. |
| crates/radix/src/parser/decl.rs:1063 | guard `&& !ty.nullable` (probably in some recovery or sig check). |
| crates/radix/src/hir/lower/expr.rs:638 | copies `nullable` into HIR. |
| crates/radix/src/semantic/passes/resolve.rs:1091 | `if ty.nullable { ... }` — likely turns into Option lowering. |

These will be refactored in Phase 3 (Semantic & Lowering). Phase 2 only ensures the parser no longer produces `nullable: true` via the old spelling.

### 9. Generated / Build Artifacts (ignore for migration)

- target/.../explain_entries.rs (many "si" mentions inside control-flow explain text) — regenerated.
- Any cached build products.

## Identifier Safety & Contextual Notes

- `sponte` and `fixus` do not appear anywhere in the tree today (only in the plan). Safe to reserve.
- After addition as keywords they will be illegal as bare identifiers (variable names, member names, etc.) unless we later introduce contextual scope (see open Q in plan).
- `∪` glyph is completely new; no conflict.

## Migration Family Buckets (for Phase 5)

1. **Param sites** (highest volume in stdlib + examples) — replace prefix `si` + reorder modes, append `sponte` after name.
2. **Return / type-alias / cast sites** — `si T` → `T ∪ nihil` (parenthesize after ⇢ when needed).
3. **Genus field sites** — `si T name` → `T name sponte`.
4. **Local var "maybe" patterns** — `fixum si T x ← nihil` → `fixum T ∪ nihil x ← nihil`.
5. **Docs + EBNF** — narrative + example updates.
6. **Negative test corpus** — add cases that `si T` now fails to parse in type positions with helpful message.

## Parser Grammar Sites That Must Change (Phase 2)

- `parse_type` (types.rs)
- `parse_param_list` (decl.rs)
- `parse_class_member` (decl.rs) — field path
- `parse_var_decl` / binding (decl.rs) — accept optional sponte/fixus after ident for uniformity (low priority)
- `try_parse_type`
- Any place that does direct `eat_keyword(Si)` for types
- TokenKind + keyword table (new Sponte, Fixus, Cup)
- TypeExpr struct + TypeExprKind (add Union, remove or ignore nullable)

## Validation That Inventory Is Complete

- All `si ` + type-word occurrences in non-target tree were captured via the searches above.
- Control-flow `si` statements (hundreds) were manually excluded by requiring a following type keyword or `de`/`in`.
- No `si` appears inside macro or scripta/ files for Faber source.

**Phase 1 complete.** This ledger is the authoritative map for the migration batches in Phase 5 and for writing the negative tests in Phase 7.

Next: Phase 2 executes the grammar/front-end changes using this inventory only as reference (no bulk migration yet).
