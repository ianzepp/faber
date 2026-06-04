# Phase 3 Delivery: Visibility And Modifier Taxonomy

**Status**: complete  
**Date**: 2026-06-04

## Scope

Import visibility in syntax AST:

- `Visibility::Private` → `Privata`
- `Visibility::Public` → `Publica`

Parser, Faber import codegen, and test fixtures updated. Structured
`AnnotationKind::{Privata,Publica,...}` were already source-shaped and unchanged.

## Out of scope

- Declaration type names (`ClassDecl`, etc.)
- Function modifier enums (already Latin)
- Keyword or parser behavior changes