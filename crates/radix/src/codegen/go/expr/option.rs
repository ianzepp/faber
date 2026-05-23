//! Optional-value lowering for Go expressions.
//!
//! Faber optionals lower to Go pointers. This file owns the pointer contract at
//! expression boundaries: wrapping non-optional values, preserving expressions
//! that already produce native option pointers, coalescing through nil checks,
//! and lowering optional/non-null chains for fields, indexes, and calls.
//!
//! INVARIANTS
//! ==========
//! - `nil` is the empty optional value.
//! - Non-optional expressions assigned to an optional target are wrapped by
//!   taking the address of a local temporary.
//! - Optional chains return `nil` on absent values, failed dynamic map checks,
//!   missing map keys, or out-of-range indexes.
//! - Non-null chains are assertions by omission: this backend emits direct Go
//!   access and lets Go panic if the value is invalid.

use super::*;

pub(super) fn expr_is_native_option_value(expr: &HirExpr, types: &TypeTable) -> bool {
    // These expression forms already emit a pointer-shaped optional when their
    // semantic type is `Option`; wrapping them again would create **T.
    if !matches!(
        expr.ty
            .map(|ty| normalize_receiver_type(types.get(ty), types)),
        Some(Type::Option(_))
    ) {
        return false;
    }

    matches!(
        expr.kind,
        HirExprKind::Path(_)
            | HirExprKind::Field(_, _)
            | HirExprKind::Index(_, _)
            | HirExprKind::OptionalChain(_, _)
            | HirExprKind::NonNull(_, _)
            | HirExprKind::Call(_, _)
            | HirExprKind::MethodCall(_, _, _)
            | HirExprKind::Verte { .. }
            | HirExprKind::Conversio { .. }
            | HirExprKind::Si { .. }
            | HirExprKind::Block(_)
            | HirExprKind::Handled { .. }
    )
}
pub(super) fn generate_coalesce_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    lhs: &HirExpr,
    rhs: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // `a ?? b` is a nil check against the pointer representation. If the whole
    // expression still returns an option, keep the pointer; otherwise deref the
    // present value before falling through to the fallback.
    match lhs
        .ty
        .map(|ty| normalize_receiver_type(types.get(ty), types))
    {
        Some(Type::Option(_)) => {
            let ret_ty = expr_return_type(expr, types, codegen);
            let returns_option = matches!(
                expr.ty
                    .map(|ty| normalize_receiver_type(types.get(ty), types)),
                Some(Type::Option(_))
            );
            w.write("func() ");
            w.write(&ret_ty);
            w.write(" { v := ");
            generate_expr(codegen, lhs, types, w)?;
            w.write("; if v != nil { return ");
            if returns_option {
                w.write("v");
            } else {
                w.write("*v");
            }
            w.write(" }; return ");
            if let Some(ret_ty) = expr.ty {
                generate_expr_for_go_type(codegen, rhs, ret_ty, types, w)?;
            } else {
                generate_expr(codegen, rhs, types, w)?;
            }
            w.write(" }()");
        }
        Some(Type::Primitive(Primitive::Nihil)) => {
            generate_expr(codegen, rhs, types, w)?;
        }
        _ => {
            generate_expr(codegen, lhs, types, w)?;
        }
    }
    Ok(())
}
pub(super) fn field_type_is_option(
    codegen: &GoCodegen<'_>,
    object_ty: crate::semantic::TypeId,
    field: crate::lexer::Symbol,
    types: &TypeTable,
) -> bool {
    // Sponte fields and explicit option-typed fields both use pointer-shaped Go
    // storage, so optional chaining must not add another address layer.
    match normalize_receiver_type(types.get(object_ty), types) {
        Type::Struct(def_id) => {
            codegen.struct_field_is_sponte(*def_id, field)
                || codegen
                    .struct_field_type(*def_id, field)
                    .is_some_and(|field_ty| {
                        matches!(normalize_receiver_type(types.get(field_ty), types), Type::Option(_))
                    })
        }
        _ => false,
    }
}

pub(super) fn field_type_is_option_through_options(
    codegen: &GoCodegen<'_>,
    mut object_ty: crate::semantic::TypeId,
    field: crate::lexer::Symbol,
    types: &TypeTable,
) -> bool {
    loop {
        match normalize_receiver_type(types.get(object_ty), types) {
            Type::Option(inner) => object_ty = *inner,
            _ => return field_type_is_option(codegen, object_ty, field, types),
        }
    }
}
pub(super) fn generate_optional_chain_expr(
    codegen: &GoCodegen<'_>,
    object: &HirExpr,
    chain: &crate::hir::HirOptionalChainKind,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Optional chain lowering is intentionally local and fail-soft: each step
    // checks the Go shape it needs and returns nil when the value is absent or
    // dynamic access cannot be proven.
    let ret_ty = expr_return_type(expr, types, codegen);
    w.write("func() ");
    w.write(&ret_ty);
    w.write(" { ");
    match chain {
        crate::hir::HirOptionalChainKind::Member(field) => {
            w.write("v := ");
            generate_expr(codegen, object, types, w)?;
            if let Some(object_ty) = object.ty {
                match normalize_receiver_type(types.get(object_ty), types) {
                    Type::Option(inner) => match normalize_receiver_type(types.get(*inner), types) {
                        Type::Map(_, value_ty) => {
                            w.write("; if v == nil { return nil }; ");
                            w.write("base := *v; ");
                            w.write("value, ok := base[");
                            w.write(&format!("{:?}", codegen.resolve_symbol(*field)));
                            w.write("]; if !ok { return nil }; ");
                            if let Some(assert_ty) = asserted_map_value_type(*value_ty, expr.ty, types) {
                                w.write("typed := value.(");
                                w.write(&types::type_to_go(codegen, assert_ty, types));
                                w.write("); ");
                                if matches!(
                                    expr.ty
                                        .map(|ty| normalize_receiver_type(types.get(ty), types)),
                                    Some(Type::Option(_))
                                ) {
                                    w.write("return &typed");
                                } else {
                                    w.write("return typed");
                                }
                            } else if matches!(
                                expr.ty
                                    .map(|ty| normalize_receiver_type(types.get(ty), types)),
                                Some(Type::Option(_))
                            ) {
                                w.write("wrapped := value; return &wrapped");
                            } else {
                                w.write("return value");
                            }
                        }
                        _ => {
                            w.write("; if v == nil { return nil }; ");
                            let field_name = capitalize(codegen.resolve_symbol(*field));
                            if field_type_is_option_through_options(codegen, *inner, *field, types) {
                                w.write("return v.");
                                w.write(&field_name);
                            } else if matches!(
                                expr.ty
                                    .map(|ty| normalize_receiver_type(types.get(ty), types)),
                                Some(Type::Option(_))
                            ) {
                                w.write("return &v.");
                                w.write(&field_name);
                            } else {
                                w.write("return v.");
                                w.write(&field_name);
                            }
                        }
                    },
                    other => match other {
                        Type::Map(_, value_ty) => {
                            w.write("; ");
                            w.write("value, ok := v[");
                            w.write(&format!("{:?}", codegen.resolve_symbol(*field)));
                            w.write("]; if !ok { return nil }; ");
                            if let Some(assert_ty) = asserted_map_value_type(*value_ty, expr.ty, types) {
                                w.write("typed := value.(");
                                w.write(&types::type_to_go(codegen, assert_ty, types));
                                w.write("); ");
                                if matches!(
                                    expr.ty
                                        .map(|ty| normalize_receiver_type(types.get(ty), types)),
                                    Some(Type::Option(_))
                                ) {
                                    w.write("return &typed");
                                } else {
                                    w.write("return typed");
                                }
                            } else if matches!(
                                expr.ty
                                    .map(|ty| normalize_receiver_type(types.get(ty), types)),
                                Some(Type::Option(_))
                            ) {
                                w.write("wrapped := value; return &wrapped");
                            } else {
                                w.write("return value");
                            }
                        }
                        Type::Primitive(Primitive::Ignotum) | Type::Primitive(Primitive::Nihil) => {
                            w.write("; if v == nil { return nil }; ");
                            w.write("m, ok := v.(map[string]any); if !ok { return nil }; ");
                            w.write("value, ok := m[");
                            w.write(&format!("{:?}", codegen.resolve_symbol(*field)));
                            w.write("]; if !ok { return nil }; ");
                            if matches!(
                                expr.ty
                                    .map(|ty| normalize_receiver_type(types.get(ty), types)),
                                Some(Type::Option(_))
                            ) {
                                w.write("wrapped := value; return &wrapped");
                            } else {
                                w.write("return value");
                            }
                        }
                        _ => {
                            w.write("; ");
                            let field_name = capitalize(codegen.resolve_symbol(*field));
                            if field_type_is_option(codegen, object_ty, *field, types) {
                                w.write("return v.");
                                w.write(&field_name);
                            } else {
                                w.write("return &v.");
                                w.write(&field_name);
                            }
                        }
                    },
                }
            } else {
                w.write("return nil");
            }
        }
        crate::hir::HirOptionalChainKind::Index(index) => {
            w.write("items := ");
            generate_expr(codegen, object, types, w)?;
            w.write("; idx := ");
            generate_expr(codegen, index, types, w)?;
            w.write("; if idx < 0 || idx >= len(items) { return nil }; return &items[idx]");
        }
        crate::hir::HirOptionalChainKind::Call(args) => {
            w.write("fn := ");
            generate_expr(codegen, object, types, w)?;
            w.write("; if fn == nil { return nil }; result := fn(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write("); ");
            if matches!(
                expr.ty
                    .map(|ty| normalize_receiver_type(types.get(ty), types)),
                Some(Type::Option(_))
            ) {
                w.write("return &result");
            } else {
                w.write("return result");
            }
        }
    }
    w.write(" }()");
    Ok(())
}
pub(super) fn generate_non_null_expr(
    codegen: &GoCodegen<'_>,
    object: &HirExpr,
    chain: &crate::hir::HirNonNullKind,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // WHY: Go has no non-null assertion. Emit plain access so invalid values
    // fail at the same point a hand-written Go dereference or lookup would.
    generate_expr(codegen, object, types, w)?;
    match chain {
        crate::hir::HirNonNullKind::Member(field) => {
            if matches!(
                object
                    .ty
                    .map(|ty| normalize_receiver_type(types.get(ty), types)),
                Some(Type::Map(_, _))
            ) {
                w.write("[");
                w.write(&format!("{:?}", codegen.resolve_symbol(*field)));
                w.write("]");
            } else {
                w.write(".");
                w.write(&capitalize(codegen.resolve_symbol(*field)));
            }
        }
        crate::hir::HirNonNullKind::Index(index) => {
            w.write("[");
            generate_expr(codegen, index, types, w)?;
            w.write("]");
        }
        crate::hir::HirNonNullKind::Call(args) => {
            w.write("(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
    }
    Ok(())
}

pub(super) fn generate_option_wrapped_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    inner_ty: crate::semantic::TypeId,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Only wrap plain values. Nil literals and expressions that already produce
    // the native pointer optional representation are emitted unchanged.
    if matches!(expr.kind, HirExprKind::Literal(HirLiteral::Nil)) || expr_is_native_option_value(expr, types) {
        return generate_expr(codegen, expr, types, w);
    }

    let inner_go_ty = types::type_to_go(codegen, inner_ty, types);
    w.write("func() *");
    w.write(&inner_go_ty);
    w.write(" { v := ");
    generate_expr(codegen, expr, types, w)?;
    w.write("; return &v }()");
    Ok(())
}
