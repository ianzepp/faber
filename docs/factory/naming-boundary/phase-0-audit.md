# Phase 0: Naming Boundary Audit

**Status**: complete  
**Date**: 2026-06-04

## Boundary Summary

| Layer | Role | Naming rule |
| --- | --- | --- |
| `syntax::ast` | Grammar-shaped AST | Latin/Faber for source tokens and prepositions |
| `parser/*` | Parse source into AST | Must emit source-shaped enum variants |
| `hir::nodes` | Lowered program with source spans | Latin when preserving source markers; English when normalized to compiler concepts |
| `semantic::types` | Checked types and passing policy | English compiler vocabulary |
| `mir::nodes` | Backend-neutral IR | English compiler vocabulary |

## `syntax/ast.rs`

| Name | Classification | Action |
| --- | --- | --- |
| `StmtKind::{Si,Dum,Itera,Redde,...}` | source-shaped Latin | keep |
| `IteraMode::{Ex,De,Ab}` | source-shaped Latin | keep (post-`ab` cleanup) |
| `ParamMode::{Owned,Ref,MutRef,Move}` | inconsistent: models `de`/`in`/`ex` | **Phase 1**: `Ref→De`, `MutRef→In`, `Move→Ex`; keep `Owned` (no marker) |
| `TypeMode::{Ref,MutRef}` | inconsistent: models `de`/`in` in type position | **Phase 1**: `Ref→De`, `MutRef→In` |
| `Visibility::{Private,Public}` | inconsistent: models `privata`/`publica` | **Phase 3**: `Private→Privata`, `Public→Publica` |
| `ClassDecl`, `InterfaceDecl`, `EnumDecl`, `UnionDecl` | mixed English on Latin keywords | **Phase 4**: defer |
| `StmtKind::Class/Interface/Enum/Union` | same | defer |
| `RangeKind::{Exclusive,Inclusive}` | compiler-shaped semantic policy | keep |
| `OptionalChainKind`, `NonNullKind` | compiler-shaped | keep |
| `AssignOp::*` | operator taxonomy | keep |
| `AnnotationKind::{Futura,Cursor,Publica,Privata,...}` | source-shaped | keep |
| `FuncModifier::*` | source-shaped Latin | keep |

## `parser/*`

| Surface | Classification | Action |
| --- | --- | --- |
| `parse_itera_stmt` → `IteraMode` | source-shaped | keep |
| `parse_param` → `ParamMode` | source-shaped markers | **Phase 1** align variants |
| `parse_type_expr` → `TypeMode` | source-shaped markers | **Phase 1** align variants |
| `parse_import_decl` → `Visibility` | source-shaped keywords | **Phase 3** align variants |
| Parser tests asserting `ParamMode::Ref` | test debt | update in Phase 1 |

## `hir/nodes.rs`

| Name | Classification | Action |
| --- | --- | --- |
| `HirIteraMode::{Ex,De,Ab}` | source-shaped (mirrors syntax) | keep |
| `HirParamMode::{Owned,Ref,MutRef,Move}` | bridge: 1:1 with syntax markers, used through borrow/typecheck | **Phase 2**: align to `De`/`In`/`Ex`/`Owned` — HIR still preserves source contract before `semantic::ParamMode` |
| `HirExprKind::{Intervallum,Conversio,Clausura,Cede}` | source-shaped | keep |
| `HirStruct`, `HirEnum`, `HirUnion` | compiler/product taxonomy | keep |
| `ImportDecl.visibility: Visibility` | syntax-shaped field on HIR import | follows **Phase 3** |

## `hir/lower/*`

| Mapping | Classification | Action |
| --- | --- | --- |
| `syntax::IteraMode` → `HirIteraMode` | intentional 1:1 bridge | keep |
| `syntax::ParamMode` → `HirParamMode` | English names on Latin semantics | **Phase 2** after syntax rename |
| `syntax::TypeMode` → `Mutability` | compiler normalization | keep English at semantic boundary |
| `param_mode_from_hir` → `semantic::ParamMode` | semantic normalization | keep English |

## `semantic/types.rs`

| Name | Classification | Action |
| --- | --- | --- |
| `ParamMode::{Owned,Ref,MutRef,Move}` | compiler-shaped passing policy | keep |
| `Type::{Array,Map,Struct,...}` | compiler-shaped | keep |
| Uses of `syntax::ParamMode` / `syntax::TypeMode` in `resolve.rs` | cross-boundary checks | update call sites only |

## `mir/nodes.rs`

| Families | Classification | Action |
| --- | --- | --- |
| `MirTerminatorKind`, `MirAggregateKind`, `MirCollectionOp`, … | compiler-shaped | keep (no changes this pass) |

## First Implementation Phase (Phase 1)

**Scope**: `syntax::ParamMode`, `syntax::TypeMode`, parser, parser tests, and syntax-touching resolve/lower call sites.

**Churn**: ~10 files, mechanical renames, no behavior change.

**Validation**:

```bash
cargo test -p radix parser -- --nocapture
cargo test -p radix hir -- --nocapture
cargo test -p radix
./scripta/lint
```

## Deferred

- Declaration taxonomy (`ClassDecl` vs `GenusDecl`): Phase 4 decision only.
- `semantic::ParamMode` and MIR families: explicitly out of scope for rename phases.