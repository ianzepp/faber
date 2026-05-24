//! Rust statement emission inside generated function bodies.
//!
//! This module owns the statement-level boundary between HIR control flow and
//! Rust syntax. Declarations decide whether a function returns plain `T` or
//! `Result<T, String>`; expression lowering decides how calls, throws, and
//! target constructs render. Statement emission threads that context through
//! local bindings, expression statements, explicit `redde`, loop control, and
//! unsupported statement forms.
//!
//! INVARIANTS
//! ==========
//! - `redde` wraps values in `Ok(...)` only when the surrounding declaration is
//!   failable and is not the generated entry context.
//! - Expression statements and local initializers preserve the current error
//!   propagation mode so failable calls can append `?` where expression
//!   lowering permits it.
//! - `ad` remains an explicit Rust backend error. This module does not invent a
//!   target mapping for behavior the Rust backend cannot yet represent.
//! - Entry contexts suppress Result propagation because entry throws are
//!   rendered as panic paths by expression lowering.
//!
//! TYPE CONTRACTS
//! ==============
//! Local annotations are emitted from the semantic type table. The local
//! initializer helper only bridges Rust's `Option<T>` constructor syntax for
//! source forms that are already known to target optional storage; it does not
//! repair missing type information.

use super::super::CodeWriter;
use super::expr::{generate_expr, generate_expr_unwrapped};
use super::types::type_to_rust;
use super::{CodegenError, RustCodegen};
use crate::hir::*;
use crate::semantic::{Primitive, Type, TypeId, TypeTable};

/// Emit one Rust statement from HIR.
///
/// The three context flags are declaration-owned policy threaded through
/// statement and expression emission: whether this function has a `Result`
/// signature, whether the statement lives under the entry wrapper, and whether
/// expression-level error propagation is temporarily disabled by a handled form.
pub fn generate_stmt(
    codegen: &RustCodegen<'_>,
    stmt: &HirStmt,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match &stmt.kind {
        HirStmtKind::Local(local) => {
            generate_local(codegen, local, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirStmtKind::Expr(expr) => {
            generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.writeln(";");
        }
        HirStmtKind::Ad(_) => {
            // `ad` has no Rust backend contract yet. Returning an explicit
            // codegen error keeps unsupported syntax visible instead of
            // producing misleading Rust.
            return Err(CodegenError { message: "ad is not supported for Rust codegen".to_owned() });
        }
        HirStmtKind::Redde(value) => {
            if let Some(expr) = value {
                w.write("return ");
                if in_failable_fn && !in_entry {
                    // `redde` returns the success value of the generated
                    // `Result`; expression lowering still decides whether any
                    // nested failable calls need `?`.
                    w.write("Ok(");
                    generate_return_value_expr(
                        codegen,
                        expr,
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    )?;
                    w.write(")");
                } else {
                    generate_return_value_expr(
                        codegen,
                        expr,
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    )?;
                }
                w.writeln(";");
            } else if in_failable_fn && !in_entry {
                // Bare `redde` from a failable function is the unit success
                // path, matching `Result<(), String>`.
                w.writeln("return Ok(());");
            } else {
                w.writeln("return;");
            }
        }
        HirStmtKind::Rumpe => {
            w.writeln("break;");
        }
        HirStmtKind::Perge => {
            w.writeln("continue;");
        }
        HirStmtKind::Tacet => {
            w.writeln("{ /* tacet: explicit noop */ }");
        }
    }
    Ok(())
}

fn generate_local(
    codegen: &RustCodegen<'_>,
    local: &HirLocal,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    // Mutability is the only local-binding policy owned here. Borrow modes,
    // option shape, and collection details come from the type table through
    // `type_to_rust`.
    w.write("let ");
    if local.mutable {
        w.write("mut ");
    }
    w.write(codegen.resolve_symbol(local.name));

    if let Some(ty) = local.ty {
        w.write(": ");
        w.write(&type_to_rust(codegen, ty, types));
    }

    if let Some(init) = &local.init {
        w.write(" = ");
        if local_init_requires_some_wrapper(codegen, local, init, types) {
            w.write("Some(");
            generate_expr(codegen, init, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            if matches!(init.kind, HirExprKind::Literal(HirLiteral::String(_))) {
                w.write(".to_string()");
            }
            w.write(")");
        } else {
            generate_expr(codegen, init, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            if local_init_clones_indexed_owned_value(local, init, types) {
                w.write(".clone()");
            }
        }
    }

    w.writeln(";");
    Ok(())
}

fn generate_return_value_expr(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if return_value_requires_some_wrapper(codegen, expr, types) {
        w.write("Some(");
        generate_expr_unwrapped(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(")");
        return Ok(());
    }

    if return_value_requires_option_unwrap(codegen, expr) {
        generate_expr_unwrapped(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(".clone().unwrap()");
        return Ok(());
    }

    generate_expr_unwrapped(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

fn return_value_requires_some_wrapper(codegen: &RustCodegen<'_>, expr: &HirExpr, types: &TypeTable) -> bool {
    let Some(return_ty) = codegen.current_return_ty() else {
        return false;
    };
    if !type_id_is_option(return_ty, types) {
        return false;
    }

    !return_value_may_already_produce_option(codegen, expr, types)
}

fn return_value_may_already_produce_option(codegen: &RustCodegen<'_>, expr: &HirExpr, types: &TypeTable) -> bool {
    match &expr.kind {
        HirExprKind::Literal(HirLiteral::Nil) | HirExprKind::OptionalChain(_, _) => true,
        HirExprKind::Path(def_id) => codegen
            .binding_type(*def_id)
            .or(expr.ty)
            .is_some_and(|ty| matches!(resolve_type(ty, types), Type::Option(_) | Type::Primitive(Primitive::Nihil))),
        HirExprKind::Call(_, _)
        | HirExprKind::MethodCall(_, _, _)
        | HirExprKind::Field(_, _)
        | HirExprKind::Index(_, _)
        | HirExprKind::NonNull(_, _) => expr
            .ty
            .is_some_and(|ty| matches!(resolve_type(ty, types), Type::Option(_) | Type::Primitive(Primitive::Nihil))),
        _ => false,
    }
}

fn return_value_requires_option_unwrap(codegen: &RustCodegen<'_>, expr: &HirExpr) -> bool {
    match &expr.kind {
        HirExprKind::Path(def_id) => codegen.binding_stores_option(*def_id),
        _ => false,
    }
}

fn local_init_clones_indexed_owned_value(local: &HirLocal, init: &HirExpr, types: &TypeTable) -> bool {
    if !matches!(init.kind, HirExprKind::Index(_, _)) {
        return false;
    }

    local
        .ty
        .is_some_and(|ty| matches!(resolve_type(ty, types), Type::Array(_) | Type::Primitive(Primitive::Textus)))
}

fn resolve_type(type_id: TypeId, types: &TypeTable) -> Type {
    match types.get(type_id) {
        Type::Alias(_, resolved) => resolve_type(*resolved, types),
        ty => ty.clone(),
    }
}

fn type_id_is_option(type_id: TypeId, types: &TypeTable) -> bool {
    match types.get(type_id) {
        Type::Option(_) => true,
        Type::Alias(_, resolved) => type_id_is_option(*resolved, types),
        _ => false,
    }
}

fn local_init_requires_some_wrapper(
    codegen: &RustCodegen<'_>,
    local: &HirLocal,
    init: &HirExpr,
    types: &TypeTable,
) -> bool {
    let Some(local_ty) = local.ty else {
        return false;
    };

    if !type_to_rust(codegen, local_ty, types).starts_with("Option<") {
        return false;
    }

    if let HirExprKind::Verte { target, .. } = &init.kind {
        if type_id_is_option(*target, types) {
            return false;
        }
    }

    // Expressions that may already produce `Option<T>` are left alone. Literal
    // and constructed values need Rust's `Some(...)` wrapper to satisfy an
    // optional local annotation.
    !matches!(
        &init.kind,
        HirExprKind::Literal(HirLiteral::Nil)
            | HirExprKind::OptionalChain(_, _)
            | HirExprKind::Path(_)
            | HirExprKind::Call(_, _)
            | HirExprKind::MethodCall(_, _, _)
            | HirExprKind::Field(_, _)
            | HirExprKind::Index(_, _)
            | HirExprKind::Binary(_, _, _)
            | HirExprKind::Si { .. }
    )
}
