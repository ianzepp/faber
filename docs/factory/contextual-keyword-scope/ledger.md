# Contextual Keyword Scope Ledger

**Parent Plan**: `docs/factory/contextual-keyword-scope/plan.md`
**Phase**: 0 baseline inventory
**Last Updated**: 2026-05-21

## Summary

The first migration family is `cura` kind vocabulary:

| Word | Current token | Intended owner | Current parser site | Recovery note |
| ---- | ------------- | -------------- | ------------------- | ------------- |
| `arena` | `TokenKind::Arena` | `KeywordOwner::CuraKind` | `parser/stmt.rs::parse_cura_stmt` | Do not depend on `Arena` as a recovery boundary; parent statement boundary is `TokenKind::Cura`. |
| `page` | identifier text | `KeywordOwner::CuraKind` | `parser/stmt.rs::parse_cura_stmt` | Already contextual; keep recovery anchored to the parent `cura` statement. |

Phase 0 found no current recovery boundary that depends on `arena`, `page`, function modifiers, member modifiers, rest markers, or spread markers. Statement recovery is anchored to statement starts in `parser/mod.rs::is_recovery_boundary()`.

## Ad Hoc Contextual Sites

| Site | Current behavior | Phase target |
| ---- | ---------------- | ------------ |
| `LexerMode::Annotation` | After `@`, normal keywords lex as identifiers for the rest of the line. | Keep as annotation scope metadata; do not collapse into normal keyword table. |
| `LexerMode::Section` | After `§`, normal keywords lex as identifiers for the rest of the line. | Keep as section scope metadata. |
| `parse_member_ident()` | Allows `cape`, `inter`, `tempta`, `scribe`, `vide`, `mone`, `nota`, `lege` as member names. | Replace with contextual identifier helper in Phase 2. |
| `parse_annotation_name()` | Allows selected keyword tokens as annotation names. | Replace with contextual identifier helper in Phase 2. |
| `eat_annotation_ident()` | Checks annotation modifier names by identifier text. | Replace with contextual grammar helper after registry is available. |
| `parse_cura_stmt()` | Matches `arena` as a keyword and `page` as identifier text. | First behavior-changing migration in Phase 3. |

## Planned Migration Families

| Family | Words | Owner | Parser call site | Recovery note |
| ------ | ----- | ----- | ---------------- | ------------- |
| `cura` kind | `arena`, `page` | `CuraKind` | `parser/stmt.rs::parse_cura_stmt` | Recover at parent `cura`; no keyword-specific restart needed. |
| entry/function modifiers | `argumenta`, `exitus`, `curata`, `errata`, `optiones`, `immutata`, `iacit` | `EntryModifier`, `FunctionModifier` | `parser/stmt.rs::parse_incipit_stmt`, `parser/decl.rs::parse_func_modifiers` | Neither site is a statement boundary; keep diagnostics local to modifier parsing. |
| genus header/member | `sub`, `implet`, `abstractus`, `generis`, `nexum` | `GenusHeader`, `GenusMember` | `parser/decl.rs::parse_class_decl_inner`, `parse_abstract_class_decl`, `parse_class_member` | `abstractus` currently starts a statement and is a recovery boundary; migrate only with explicit lookahead/recovery replacement. |
| rest/spread | `ceteri`, `sparge` | `RestPattern`, `SpreadExpression` | `parse_param_list`, binding/import/extract parsers, `parser/expr.rs` literal/call parsers | No recovery boundary today; keep failures local to the owning list/literal parser. |
| annotations | `futura`, `cursor`, annotation names/modifiers | `AnnotationName`, `AnnotationModifier` | `parser/decl.rs::parse_annotations` and annotation subparsers | Annotation lexer mode already makes names identifiers; helper should validate owner membership. |
| member identifier exceptions | `cape`, `inter`, `tempta`, `scribe`, `vide`, `mone`, `nota`, `lege` | `MemberIdentifier` | `parser/mod.rs::parse_member_ident` | Identifier-position helper must not alter statement recovery. |

## Identifier-Safety Matrix

These words must become or remain legal ordinary identifiers outside their owner contexts after the relevant migration phase:

| Word | Owner context | Must be legal outside owner in |
| ---- | ------------- | ------------------------------ |
| `arena` | immediate `cura` kind slot | local bindings, function names, type names, member names, parameter names |
| `page` | immediate `cura` kind slot | local bindings, function names, type names, member names, parameter names |
| `argumenta` | entry/function modifier slots | locals, params, functions, fields, imports |
| `exitus` | entry/function modifier slots | locals, params, functions, fields, imports |
| `curata` | function modifier slot | locals, params, functions, fields, imports |
| `errata` | function modifier slot | locals, params, functions, fields, imports |
| `optiones` | function modifier slot | locals, params, functions, fields, imports |
| `immutata` | function modifier slot | locals, params, functions, fields, imports |
| `iacit` | function modifier slot | locals, params, functions, fields, imports |
| `generis` | genus member field modifier | locals, params, functions, non-genus type/member names |
| `nexum` | genus member field modifier | locals, params, functions, non-genus type/member names |
| `sub` | genus header inheritance | locals, params, functions, fields |
| `implet` | genus header implementation list | locals, params, functions, fields |
| `abstractus` | `abstractus genus` declaration prefix | ordinary identifiers only if statement dispatch/recovery is redesigned |
| `ceteri` | rest positions | locals, params, functions, fields when not in rest slot |
| `sparge` | spread positions | locals, params, functions, fields when not in spread slot |

## Current Keyword Inventory

Legend:

- `Scope`: `global`, `contextual`, `alias`, `backlog`, or `test-owned`.
- `Stmt`: starts a statement in `parse_statement`.
- `Rec`: is a recovery boundary in `is_recovery_boundary`.
- `Expr`: participates in expression parsing as operator, literal, or form.
- `Parent`: only meaningful under a parent construct.
- `Docs`: current docs/examples teach or exercise the spelling as active syntax.

| Word | TokenKind | Scope | Owner | Parser owner | Stmt | Rec | Expr | Parent | Alias | Docs |
| ---- | --------- | ----- | ----- | ------------ | ---- | --- | ---- | ------ | ----- | ---- |
| `fixum` | `Fixum` | global | - | var declarations, binding sites | yes | yes | no | no | - | yes |
| `varia` | `Varia` | global | - | var declarations, binding sites | yes | yes | no | no | - | yes |
| `functio` | `Functio` | global | - | function/interface method declarations | yes | yes | no | no | - | yes |
| `genus` | `Genus` | global | - | class declarations | yes | yes | no | no | - | yes |
| `pactum` | `Pactum` | global | - | interface declarations | yes | yes | no | no | - | yes |
| `typus` | `Typus` | global | - | type aliases/type params/annotations | yes | yes | no | sometimes | - | yes |
| `ordo` | `Ordo` | global | - | enum declarations | yes | yes | no | no | - | yes |
| `discretio` | `Discretio` | global | - | tagged union declarations | yes | yes | no | no | - | yes |
| `importa` | `Importa` | global | - | import declarations | yes | yes | no | no | - | yes |
| `abstractus` | `Abstractus` | contextual | `GenusHeader` | `parse_abstract_class_decl` | yes | yes | no | yes | - | yes |
| `generis` | `Generis` | contextual | `GenusMember` | `parse_class_member` | no | no | no | yes | - | yes |
| `nexum` | `Nexum` | contextual | `GenusMember` | `parse_class_member` | no | no | no | yes | - | yes |
| `publica` | `Publica` | contextual | annotation/import visibility | `parse_import_decl`, `parse_annotation_name` | no | no | no | yes | - | yes |
| `privata` | `Privata` | contextual | annotation/import visibility | `parse_import_decl`, `parse_annotation_name` | no | no | no | yes | - | yes |
| `protecta` | `Protecta` | contextual | annotation name | `parse_annotation_name` | no | no | no | yes | - | no |
| `prae` | `Prae` | contextual | type parameter | `parse_param_list`, `try_parse_type_params` | no | no | no | yes | - | yes |
| `ceteri` | `Ceteri` | contextual | `RestPattern` | params, destructuring, extraction, annotations | no | no | no | yes | - | yes |
| `immutata` | `Immutata` | contextual | `FunctionModifier` | `parse_func_modifiers` | no | no | no | yes | - | yes |
| `iacit` | `Iacit` | contextual | `FunctionModifier` | `parse_func_modifiers` | no | no | no | yes | - | yes |
| `curata` | `Curata` | contextual | `FunctionModifier` | `parse_func_modifiers` | no | no | no | yes | - | yes |
| `errata` | `Errata` | contextual | `FunctionModifier` | `parse_func_modifiers` | no | no | no | yes | - | yes |
| `exitus` | `Exitus` | contextual | `EntryModifier`, `FunctionModifier` | `parse_incipit_stmt`, `parse_func_modifiers` | no | no | no | yes | - | yes |
| `optiones` | `Optiones` | contextual | `FunctionModifier` | `parse_func_modifiers` | no | no | no | yes | - | yes |
| `si` | `Si` | global | - | if statements, optional params | yes | yes | no | no | - | yes |
| `sic` | `Sic` | global | - | ternary expression tail | no | no | yes | yes | - | yes |
| `sin` | `Sin` | global | - | if statement else-if clause | no | no | no | yes | - | yes |
| `secus` | `Secus` | global | - | if statement else clause | no | no | no | yes | - | yes |
| `dum` | `Dum` | global | - | while statements | yes | yes | no | no | - | yes |
| `itera` | `Itera` | global | - | iteration statements | yes | yes | no | no | - | yes |
| `elige` | `Elige` | global | - | switch statements | yes | yes | no | no | - | yes |
| `casu` | `Casu` | global | - | switch/match arms | no | no | no | yes | - | yes |
| `ceterum` | `Ceterum` | global | - | default switch/match arms | no | no | no | yes | - | yes |
| `discerne` | `Discerne` | global | - | match statements | yes | yes | no | no | - | yes |
| `custodi` | `Custodi` | global | - | guard statements | yes | yes | no | no | - | yes |
| `fac` | `Fac` | global | - | do-while statements | yes | yes | no | no | - | yes |
| `ergo` | `Ergo` | global | - | single-statement bodies | no | no | no | yes | - | yes |
| `redde` | `Redde` | global | - | return statements | yes | yes | no | no | - | yes |
| `rumpe` | `Rumpe` | global | - | break statements | yes | yes | no | no | - | yes |
| `perge` | `Perge` | global | - | continue statements | yes | yes | no | no | - | yes |
| `tacet` | `Tacet` | global | - | no-op statements | yes | yes | no | no | - | yes |
| `tempta` | `Tempta` | global | - | try statements/member identifiers | yes | yes | no | no | - | yes |
| `cape` | `Cape` | global | - | catch clauses/member identifiers | no | no | no | yes | - | yes |
| `demum` | `Demum` | global | - | finally clauses | no | no | no | yes | - | yes |
| `iace` | `Iace` | global | - | throw statements | yes | yes | no | no | - | yes |
| `mori` | `Mori` | global | - | panic statements | yes | yes | no | no | - | yes |
| `adfirma` | `Adfirma` | global | - | assert statements | yes | yes | no | no | - | yes |
| `clausura` | `Clausura` | global | - | closure expressions | no | no | yes | no | - | yes |
| `cede` | `Cede` | global | - | await/yield expressions | no | no | yes | no | - | yes |
| `verum` | `Verum` | global | - | boolean literal | no | no | yes | no | - | yes |
| `falsum` | `Falsum` | global | - | boolean literal | no | no | yes | no | - | yes |
| `nihil` | `Nihil` | global | - | nil literal | no | no | yes | no | - | yes |
| `et` | `Et` | global | - | logical operator | no | no | yes | no | - | yes |
| `aut` | `Aut` | global | - | logical operator | no | no | yes | no | - | yes |
| `non` | `Non` | global | - | logical/unary operator | no | no | yes | no | - | yes |
| `vel` | `Vel` | global | - | nullish/default operator | no | no | yes | no | - | yes |
| `est` | `Est` | global | - | identity operator | no | no | yes | no | - | yes |
| `ego` | `Ego` | global | - | self expression | no | no | yes | no | - | yes |
| `finge` | `Finge` | global | - | variant construction | no | no | yes | no | - | yes |
| `sub` | `Sub` | contextual | `GenusHeader` | `parse_class_decl_inner` | no | no | no | yes | - | yes |
| `implet` | `Implet` | contextual | `GenusHeader` | `parse_class_decl_inner` | no | no | no | yes | - | yes |
| `scribe` | `Scribe` | alias | - | diagnostic output/member identifiers | yes | yes | yes | no | `nota` | yes |
| `vide` | `Vide` | backlog | - | diagnostic output/member identifiers | yes | yes | yes | no | - | yes |
| `mone` | `Mone` | backlog | - | diagnostic output/member identifiers | yes | yes | yes | no | - | yes |
| `nota` | `Nota` | global | - | diagnostic output/member identifiers | yes | yes | yes | no | - | yes |
| `incipit` | `Incipit` | global | - | sync entry point | yes | yes | no | no | - | yes |
| `incipiet` | `Incipiet` | global | - | async entry point | yes | yes | no | no | - | yes |
| `argumenta` | `Argumenta` | contextual | `EntryModifier`, `FunctionModifier` | `parse_incipit_stmt`, `parse_func_modifiers` | no | no | no | yes | - | yes |
| `cura` | `Cura` | global | - | resource statements | yes | yes | no | no | - | yes |
| `arena` | `Arena` | contextual | `CuraKind` | `parse_cura_stmt` | no | no | no | yes | - | yes |
| `page` | identifier | contextual | `CuraKind` | `parse_cura_stmt` | no | no | no | yes | - | yes |
| `ad` | `Ad` | global | - | endpoint statements | yes | yes | no | no | - | yes |
| `ex` | `Ex` | global | - | extraction statements, modes, imports | yes | yes | no | sometimes | - | yes |
| `de` | `De` | global | - | borrow/type/param/iteration modes | no | no | no | yes | - | yes |
| `in` | `In` | global | - | mutable borrow/type/param modes | no | no | no | yes | - | yes |
| `ut` | `Ut` | global | - | aliases and pattern binding | no | no | no | yes | - | yes |
| `pro` | `Pro` | global | - | iteration and endpoint binding mode | no | no | no | yes | - | yes |
| `omnia` | `Omnia` | global | - | exhaustive matching/test hooks | no | no | no | yes | - | yes |
| `sparge` | `Sparge` | contextual | `SpreadExpression` | call/array/object spread parsers | no | no | yes | yes | - | yes |
| `praefixum` | `Praefixum` | backlog | - | comptime expression | no | no | yes | no | - | yes |
| `scriptum` | `Scriptum` | backlog | - | formatted string expression | no | no | yes | no | - | yes |
| `lege` | `Lege` | backlog | - | input expression/member identifiers | no | no | yes | no | - | yes |
| `lineam` | `Lineam` | backlog | - | line input expression | no | no | yes | no | - | yes |
| `sed` | `Sed` | backlog | - | regex expression | no | no | yes | no | - | yes |
| `ante` | `Ante` | backlog | - | range operator | no | no | yes | no | - | yes |
| `usque` | `Usque` | backlog | - | range operator | no | no | yes | no | - | yes |
| `per` | `Per` | backlog | - | range step | no | no | yes | yes | - | yes |
| `intra` | `Intra` | backlog | - | range membership operator | no | no | yes | no | - | yes |
| `inter` | `Inter` | backlog | - | between operator/member identifiers | no | no | yes | no | - | yes |
| `ab` | `Ab` | backlog | - | collection query expression | no | no | yes | no | - | yes |
| `ubi` | `Ubi` | backlog | - | collection query filter | no | no | no | yes | - | yes |
| `prima` | `Prima` | backlog | - | collection query terminal | no | no | no | yes | - | yes |
| `ultima` | `Ultima` | backlog | - | collection query terminal | no | no | no | yes | - | yes |
| `summa` | `Summa` | backlog | - | collection query terminal | no | no | no | yes | - | yes |
| `nulla` | `Nulla` | backlog | - | predicate unary operator | no | no | yes | no | - | yes |
| `nonnulla` | `Nonnulla` | backlog | - | predicate unary operator | no | no | yes | no | - | yes |
| `nonnihil` | `Nonnihil` | backlog | - | predicate unary operator | no | no | yes | no | - | yes |
| `negativum` | `Negativum` | backlog | - | predicate unary operator | no | no | yes | no | - | yes |
| `positivum` | `Positivum` | backlog | - | predicate unary operator | no | no | yes | no | - | yes |

## Test-Owned Words

These words are explicitly out of scope for this contextual keyword migration except for keeping metadata complete:

| Word | TokenKind | Parser owner |
| ---- | --------- | ------------ |
| `probandum` | `Probandum` | test suite declarations |
| `proba` | `Proba` | test case declarations |
| `praepara` | `Praepara` | test setup hooks |
| `praeparabit` | `Praeparabit` | test setup hooks |
| `postpara` | `Postpara` | test teardown hooks |
| `postparabit` | `Postparabit` | test teardown hooks |
| `omitte` | `Omitte` | test modifiers / annotation names |
| `futurum` | `Futurum` | test modifiers |
| `solum` | `Solum` | test modifiers / annotation names |
| `tag` | `Tag` | test tags / identifier exception |
| `temporis` | `Temporis` | test timeout modifier |
| `metior` | `Metior` | benchmark/test modifier / annotation names |
| `repete` | `Repete` | repeat test modifier |
| `fragilis` | `Fragilis` | flaky test modifier |
| `requirit` | `Requirit` | test requirement modifier |
| `solum_in` | `SolumIn` | environment-specific test modifier |

## Annotation And Section Vocabulary

`LexerMode::Annotation` and `LexerMode::Section` deliberately turn words into identifiers after `@` and `§`. The first registry should still include known annotation-owned words such as `futura`, `cursor`, `cli`, `imperium`, `optio`, `operandus`, `brevis`, `longum`, `descriptio`, `ubique`, and annotation modifier reuse such as `vel`.

## Alias And Removed-Alias Notes

`scribe` remains a current compatibility spelling for neutral diagnostic output and should be represented as an alias of canonical `nota`.

The old compile-time cast spellings `qua`, `innatum`, and `novum` no longer appear in `keyword_or_ident()` and currently lex as identifiers. They should not be restored as keywords by this migration. If future docs need a removed-alias registry, it should be separate from current active keyword behavior.

