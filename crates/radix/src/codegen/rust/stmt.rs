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
//! - `ad` lowers through a temporary unresolved-capability dispatcher. The
//!   Rust path compiles permissively, then fails explicitly at runtime unless
//!   a later host/provider path supplies a real route.
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
use super::expr::{generate_expr, generate_expr_unwrapped, ExprEmitPolicy, ExprEmitter};
use super::type_shape::{option_inner_or_self, resolve_type, type_id_is_option};
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
            if let HirExprKind::Cede(value) = &expr.kind {
                if codegen.current_generator_yield_ty().is_some() {
                    w.write("__faber_yielded.push(");
                    generate_expr_unwrapped(
                        codegen,
                        value,
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    )?;
                    w.writeln(");");
                    return Ok(());
                }
            }
            generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.writeln(";");
        }
        HirStmtKind::Ad(ad) => {
            generate_ad_stmt(codegen, ad, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
        w.write(&local_storage_type_to_rust(codegen, local, ty, types));
    }

    if let Some(init) = &local.init {
        w.write(" = ");
        if let Some(value_ty) = local_optional_value_type(codegen, local, types) {
            generate_optional_target_expr(
                codegen,
                init,
                value_ty,
                types,
                w,
                ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
            )?;
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

fn generate_ad_stmt(
    codegen: &RustCodegen<'_>,
    ad: &HirAd,
    types: &TypeTable,
    writer: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    // Non-strict capability calls compile without provider metadata. The
    // temporary dispatcher never constructs the success value, but the generic
    // result type keeps the source-declared success binding meaningful.
    let binding_ty = match &ad.binding {
        Some(binding) => binding.ty.ok_or_else(|| CodegenError {
            message: "ad capability calls with a success binding require an explicit result type".to_owned(),
        })?,
        None => types.primitive(Primitive::Vacuum),
    };
    writer.write("match ");
    ExprEmitter::new(
        codegen,
        types,
        writer,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    )
    .ad_dispatch(ad, binding_ty)?;
    writer.writeln(" {");
    writer.indented(|writer| {
        writer.write("Ok(__faber_result) => ");
        let mut result = if let Some(body) = &ad.body {
            generate_ad_success_block(
                codegen,
                ad,
                body,
                types,
                writer,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )
        } else {
            writer.writeln("{");
            writer.indented(|writer| {
                if ad.binding.is_some() {
                    writer.writeln("let _ = __faber_result;");
                }
            });
            writer.write("}");
            Ok(())
        };
        if result.is_ok() {
            writer.writeln(",");
            writer.write("Err(__faber_err) => ");
            result = match &ad.catch {
                Some(catch) => generate_ad_error_block(
                    codegen,
                    catch,
                    types,
                    writer,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                ),
                None => {
                    writer.writeln("{");
                    writer.indented(|writer| {
                        if in_failable_fn && !in_entry && !suppress_error_propagation {
                            writer.writeln("return Err(__faber_err);");
                        } else {
                            writer.writeln("panic!(\"{}\", __faber_err);");
                        }
                    });
                    writer.write("}");
                    Ok(())
                }
            };
        }
        if result.is_ok() {
            writer.writeln(",");
        }
    });
    writer.writeln("}");
    Ok(())
}

fn generate_ad_success_block(
    codegen: &RustCodegen<'_>,
    ad: &HirAd,
    body: &HirBlock,
    types: &TypeTable,
    writer: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let Some(binding) = &ad.binding else {
        return generate_ad_plain_block(
            codegen,
            body,
            0,
            types,
            writer,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        );
    };
    let Some(HirStmt { kind: HirStmtKind::Local(binding_local), .. }) = body.stmts.first() else {
        return generate_ad_plain_block(
            codegen,
            body,
            0,
            types,
            writer,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        );
    };

    let alias_local = if binding.alias.is_some() {
        match body.stmts.get(1) {
            Some(HirStmt { kind: HirStmtKind::Local(local), .. }) => Some(local),
            _ => None,
        }
    } else {
        None
    };
    let skip = 1 + usize::from(alias_local.is_some());

    generate_ad_prefixed_block(
        codegen,
        body,
        skip,
        types,
        writer,
        in_failable_fn,
        in_entry,
        suppress_error_propagation,
        |writer| {
            writer.write("let ");
            writer.write(codegen.resolve_symbol(binding_local.name));
            writer.writeln(" = __faber_result;");
            if let Some(alias_local) = alias_local {
                writer.write("let ");
                writer.write(codegen.resolve_symbol(alias_local.name));
                writer.write(" = ");
                writer.write(codegen.resolve_symbol(binding.name));
                writer.writeln(".clone();");
            }
            Ok(())
        },
    )
}

fn generate_ad_error_block(
    codegen: &RustCodegen<'_>,
    catch: &HirBlock,
    types: &TypeTable,
    writer: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let Some(HirStmt { kind: HirStmtKind::Local(local), .. }) = catch.stmts.first() else {
        return generate_ad_plain_block(
            codegen,
            catch,
            0,
            types,
            writer,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        );
    };

    generate_ad_prefixed_block(
        codegen,
        catch,
        1,
        types,
        writer,
        in_failable_fn,
        in_entry,
        suppress_error_propagation,
        |writer| {
            writer.write("let ");
            writer.write(codegen.resolve_symbol(local.name));
            writer.writeln(" = __faber_err;");
            Ok(())
        },
    )
}

fn generate_ad_plain_block(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    skip: usize,
    types: &TypeTable,
    writer: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    generate_ad_prefixed_block(
        codegen,
        block,
        skip,
        types,
        writer,
        in_failable_fn,
        in_entry,
        suppress_error_propagation,
        |_| Ok(()),
    )
}

fn generate_ad_prefixed_block<F>(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    skip: usize,
    types: &TypeTable,
    writer: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
    prefix: F,
) -> Result<(), CodegenError>
where
    F: FnOnce(&mut CodeWriter) -> Result<(), CodegenError>,
{
    writer.writeln("{");
    let mut result = Ok(());
    writer.indented(|writer| {
        result = prefix(writer);
        if result.is_err() {
            return;
        }
        for stmt in block.stmts.iter().skip(skip) {
            result = generate_stmt(
                codegen,
                stmt,
                types,
                writer,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            );
            if result.is_err() {
                return;
            }
        }
        if let Some(expr) = &block.expr {
            result = generate_expr(
                codegen,
                expr,
                types,
                writer,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            );
            if result.is_ok() {
                writer.writeln(";");
            }
        }
    });
    result?;
    writer.write("}");
    Ok(())
}

fn local_storage_type_to_rust(codegen: &RustCodegen<'_>, local: &HirLocal, ty: TypeId, types: &TypeTable) -> String {
    if local
        .init
        .as_ref()
        .is_some_and(|init| matches!(init.kind, HirExprKind::Literal(HirLiteral::Nil)))
        && matches!(resolve_type(ty, types), Type::Primitive(Primitive::Nihil))
    {
        return "Option<()>".to_owned();
    }

    type_to_rust(codegen, ty, types)
}

fn generate_optional_target_expr(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    value_ty: TypeId,
    types: &TypeTable,
    writer: &mut CodeWriter,
    policy: ExprEmitPolicy,
) -> Result<(), CodegenError> {
    let mut emitter = ExprEmitter::new(codegen, types, writer, policy);
    emitter.expr_as_optional_target(expr, value_ty)
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
    if let Some(value_ty) = return_optional_value_type(codegen, types) {
        generate_optional_target_expr(
            codegen,
            expr,
            value_ty,
            types,
            w,
            ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
        )?;
        return Ok(());
    }

    if return_value_requires_option_unwrap(codegen, expr) {
        generate_expr_unwrapped(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(".clone().unwrap()");
        return Ok(());
    }

    generate_expr_unwrapped(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

fn return_optional_value_type(codegen: &RustCodegen<'_>, types: &TypeTable) -> Option<TypeId> {
    let return_ty = codegen.current_return_ty()?;
    if !type_id_is_option(return_ty, types) {
        return None;
    }
    Some(option_inner_or_self(return_ty, types))
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

fn local_optional_value_type(codegen: &RustCodegen<'_>, local: &HirLocal, types: &TypeTable) -> Option<TypeId> {
    let local_ty = local.ty?;

    if !type_to_rust(codegen, local_ty, types).starts_with("Option<") {
        return None;
    }

    Some(option_inner_or_self(local_ty, types))
}
