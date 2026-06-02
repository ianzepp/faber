//! Statement and block emission for the Go backend.
//!
//! Statement codegen owns Go control-flow shape: blocks, local declarations,
//! explicit returns, loops, `ad` statements, and statement-position `discerne`.
//! Expression codegen still renders expression values; this file decides when a
//! Faber expression must become a Go statement, a block tail return, or a
//! fail-closed codegen error because the statement form has no Phase 9 Go
//! representation.
//!
//! CONTRACTS
//! =========
//! - Tail expressions become `return` only in value/function block helpers that
//!   receive an expected result type.
//! - Entry-point blocks are emitted without braces because `mod.rs` owns the
//!   surrounding `func main()`.
//! - Local `nil` initialization uses `var` with the best available type because
//!   Go cannot infer a type from `nil`.
//! - Unsupported structured `cape` handlers, mixed pattern shapes, and missing
//!   variant metadata return [`CodegenError`] instead of emitting guessed Go.
//! - `ad` statements preserve catch/body control flow around a generated
//!   `radixAd` call, but the actual dispatch implementation is a backend stub.

use super::expr::generate_expr_for_go_type;
use super::types::type_to_go;
use super::{expr::generate_expr, CodeWriter, CodegenError, GoCodegen};
use crate::hir::{HirBlock, HirExprKind, HirPattern, HirStmt, HirStmtKind};
use crate::semantic::{TypeId, TypeTable};

fn nil_init_type(expr: &crate::hir::HirExpr) -> Option<crate::semantic::TypeId> {
    // `nil` cannot stand alone in Go short declarations. Conversions around
    // nil carry the target type needed to write a typed `var x T = nil`.
    match &expr.kind {
        HirExprKind::Literal(crate::hir::HirLiteral::Nil) => expr.ty,
        HirExprKind::Verte { source, target, .. }
            if matches!(source.kind, HirExprKind::Literal(crate::hir::HirLiteral::Nil)) =>
        {
            Some(*target)
        }
        HirExprKind::Conversio { source, target, .. }
            if matches!(source.kind, HirExprKind::Literal(crate::hir::HirLiteral::Nil)) =>
        {
            Some(*target)
        }
        _ => None,
    }
}

pub(super) fn generate_prefixed_block<P>(
    codegen: &GoCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    skip_stmts: usize,
    result_ty: Option<crate::semantic::TypeId>,
    prelude: P,
) -> Result<(), CodegenError>
where
    P: FnOnce(&mut CodeWriter) -> Result<(), CodegenError>,
{
    // This helper is the shared "block with optional synthetic leading
    // statements" path. It is used for catch bindings, `ad` result bindings,
    // and value blocks that must turn a tail expression into `return`.
    w.writeln("{");
    let mut result = Ok(());
    let mut prelude = Some(prelude);
    w.indented(|w| {
        if let Some(prelude) = prelude.take() {
            result = prelude(w);
        }
        for stmt in block.stmts.iter().skip(skip_stmts) {
            if result.is_err() {
                return;
            }
            result = generate_stmt(codegen, stmt, types, w, result_ty.or_else(|| codegen.current_return_ty()));
        }
        if result.is_ok() {
            if let Some(expr) = &block.expr {
                w.write("return ");
                if let Some(result_ty) = result_ty.or_else(|| codegen.current_return_ty()) {
                    result = generate_expr_for_go_type(codegen, expr, result_ty, types, w);
                } else {
                    result = generate_expr(codegen, expr, types, w);
                }
                w.newline();
            }
        }
    });
    result?;
    w.write("}");
    Ok(())
}

pub fn generate_block<F>(
    codegen: &GoCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    prelude: F,
) -> Result<(), CodegenError>
where
    F: FnOnce(&mut CodeWriter),
{
    generate_prefixed_block(codegen, block, types, w, 0, None, |w| {
        prelude(w);
        Ok(())
    })
}

pub(super) fn generate_value_block(
    codegen: &GoCodegen<'_>,
    block: &HirBlock,
    result_ty: crate::semantic::TypeId,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // A value block has a known result type, so optional wrapping and nil
    // handling are delegated through expression emission for that Go type.
    generate_prefixed_block(codegen, block, types, w, 0, Some(result_ty), |_| Ok(()))
}

/// Emit only the statements inside a block (no braces).
///
/// WHY: Used for the entry-point `main()` body where the braces are
/// already emitted by the caller.
pub fn generate_block_stmts(
    codegen: &GoCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    for stmt in &block.stmts {
        generate_stmt(codegen, stmt, types, w, None)?;
    }
    if let Some(expr) = &block.expr {
        generate_expr(codegen, expr, types, w)?;
        w.newline();
    }
    Ok(())
}

fn generate_stmt_block(
    codegen: &GoCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    current_return_ty: Option<TypeId>,
) -> Result<(), CodegenError> {
    // Statement-position blocks keep tail expressions as expression statements.
    // Function/method blocks use declaration helpers when tails must return.
    w.writeln("{");
    let mut result = Ok(());
    w.indented(|w| {
        for stmt in &block.stmts {
            if result.is_err() {
                return;
            }
            result = generate_stmt(codegen, stmt, types, w, current_return_ty);
        }
        if result.is_ok() {
            if let Some(expr) = &block.expr {
                result = generate_expr_stmt(codegen, expr, types, w, current_return_ty);
            }
        }
    });
    result?;
    w.write("}");
    Ok(())
}

pub fn generate_stmt(
    codegen: &GoCodegen<'_>,
    stmt: &HirStmt,
    types: &TypeTable,
    w: &mut CodeWriter,
    current_return_ty: Option<TypeId>,
) -> Result<(), CodegenError> {
    match &stmt.kind {
        HirStmtKind::Local(local) => {
            // WHY: Go uses := for short variable declarations with inferred types,
            // and var for explicit types without initializers or nil values.
            if let Some(init) = &local.init {
                let name = codegen.resolve_symbol(local.name);
                let nil_init_ty = nil_init_type(init);
                if matches!(init.kind, HirExprKind::Literal(crate::hir::HirLiteral::Nil)) || nil_init_ty.is_some() {
                    w.write("var ");
                    w.write(name);
                    if let Some(ty) = local.ty.or(init.ty).or(nil_init_ty) {
                        w.write(" ");
                        w.write(&type_to_go(codegen, ty, types));
                    } else {
                        w.write(" any");
                    }
                    w.write(" = nil");
                    w.newline();
                } else {
                    w.write(name);
                    w.write(" := ");
                    if let Some(ty) = local.ty {
                        generate_expr_for_go_type(codegen, init, ty, types, w)?;
                    } else {
                        generate_expr(codegen, init, types, w)?;
                    }
                    w.newline();
                }
                if !codegen.is_used(local.def_id) {
                    w.write("_ = ");
                    w.writeln(name);
                }
            } else {
                w.write("var ");
                w.write(codegen.resolve_symbol(local.name));
                if let Some(ty) = local.ty {
                    w.write(" ");
                    w.write(&type_to_go(codegen, ty, types));
                }
                w.newline();
                if !codegen.is_used(local.def_id) {
                    w.write("_ = ");
                    w.writeln(codegen.resolve_symbol(local.name));
                }
            }
        }
        HirStmtKind::Expr(expr) => {
            generate_expr_stmt(codegen, expr, types, w, current_return_ty)?;
        }
        HirStmtKind::Ad(ad) => {
            generate_ad_stmt(codegen, ad, types, w)?;
        }
        HirStmtKind::Redde(expr) => {
            if let Some(expr) = expr {
                w.write("return ");
                if let Some(return_ty) = current_return_ty.or_else(|| codegen.current_return_ty()) {
                    generate_expr_for_go_type(codegen, expr, return_ty, types, w)?;
                } else {
                    generate_expr(codegen, expr, types, w)?;
                }
                w.newline();
            } else {
                w.writeln("return");
            }
        }
        HirStmtKind::Rumpe => w.writeln("break"),
        HirStmtKind::Perge => w.writeln("continue"),
        HirStmtKind::Tacet => w.writeln("{ /* tacet: explicit noop */ }"),
    }
    Ok(())
}

fn generate_ad_stmt(
    codegen: &GoCodegen<'_>,
    ad: &crate::hir::HirAd,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // `ad` has statement-level control flow even though Go has no backend
    // transport here. Calls target the generated `radixAd` stub; caught errors
    // enter the catch block, uncaught errors panic, and successful results bind
    // into the optional body.
    let binding_ty = ad
        .binding
        .as_ref()
        .map(|binding| binding.ty)
        .map(|ty| type_to_go(codegen, ty, types))
        .filter(|ty| !ty.is_empty())
        .unwrap_or_else(|| "any".to_owned());

    w.write("if ");
    if ad.binding.is_some() {
        w.write("__radixResult, ");
    } else {
        w.write("_, ");
    }
    w.write("__radixErr := radixAd[");
    w.write(&binding_ty);
    w.write("](");
    write_ad_call_args(codegen, ad, types, w)?;
    w.write("); __radixErr != nil ");

    match &ad.catch {
        Some(catch) => generate_error_binding_block(codegen, catch, "__radixErr", types, w)?,
        None => {
            w.writeln("{");
            w.indented(|w| w.writeln("panic(__radixErr)"));
            w.write("}");
        }
    }

    if let Some(body) = &ad.body {
        w.write(" else ");
        if ad.binding.is_some() {
            generate_ad_body_block(codegen, ad, body, types, w)?;
        } else {
            generate_block(codegen, body, types, w, |_| {})?;
        }
    } else if ad.binding.is_some() {
        w.write(" else ");
        w.writeln("{");
        w.indented(|w| w.writeln("_ = __radixResult"));
        w.write("}");
    }

    w.newline();
    Ok(())
}

fn write_ad_call_args(
    codegen: &GoCodegen<'_>,
    ad: &crate::hir::HirAd,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write(&format!("{:?}", codegen.resolve_symbol(ad.path)));
    for arg in &ad.args {
        w.write(", ");
        generate_expr(codegen, arg, types, w)?;
    }
    Ok(())
}

pub(super) fn generate_error_binding_block(
    codegen: &GoCodegen<'_>,
    block: &HirBlock,
    value_expr: &str,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Lowered catch blocks begin with a local binding when the source named the
    // error. Consume that synthetic first statement and reintroduce it from the
    // Go error expression so the rest of the block emits normally.
    let Some(crate::hir::HirStmt { kind: HirStmtKind::Local(local), .. }) = block.stmts.first() else {
        return generate_block(codegen, block, types, w, |_| {});
    };

    generate_prefixed_block(codegen, block, types, w, 1, None, |w| {
        w.write(codegen.resolve_symbol(local.name));
        w.write(" := ");
        w.writeln(value_expr);
        if !codegen.is_used(local.def_id) {
            w.write("_ = ");
            w.writeln(codegen.resolve_symbol(local.name));
        }
        Ok(())
    })
}

fn generate_ad_body_block(
    codegen: &GoCodegen<'_>,
    ad: &crate::hir::HirAd,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Successful `ad` bodies can start with compiler-lowered locals for the
    // result binding and optional alias. Replace those locals with assignments
    // from `__radixResult` so the generated body has one source of truth.
    let Some(binding) = &ad.binding else {
        return generate_block(codegen, block, types, w, |_| {});
    };

    let Some(crate::hir::HirStmt { kind: HirStmtKind::Local(binding_local), .. }) = block.stmts.first() else {
        return generate_block(codegen, block, types, w, |_| {});
    };

    let alias_local = if binding.alias.is_some() {
        match block.stmts.get(1) {
            Some(crate::hir::HirStmt { kind: HirStmtKind::Local(local), .. }) => Some(local),
            _ => None,
        }
    } else {
        None
    };
    let skip = 1 + usize::from(alias_local.is_some());

    generate_prefixed_block(codegen, block, types, w, skip, None, |w| {
        w.write(codegen.resolve_symbol(binding_local.name));
        w.write(" := __radixResult");
        w.newline();
        if !codegen.is_used(binding_local.def_id) {
            w.write("_ = ");
            w.writeln(codegen.resolve_symbol(binding_local.name));
        }
        if let Some(alias_local) = alias_local {
            w.write(codegen.resolve_symbol(alias_local.name));
            w.write(" := ");
            w.writeln(codegen.resolve_symbol(binding.name));
            if !codegen.is_used(alias_local.def_id) {
                w.write("_ = ");
                w.writeln(codegen.resolve_symbol(alias_local.name));
            }
        }
        Ok(())
    })
}

fn generate_expr_stmt(
    codegen: &GoCodegen<'_>,
    expr: &crate::hir::HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    current_return_ty: Option<TypeId>,
) -> Result<(), CodegenError> {
    // Some Faber expressions become Go statements when their result is not
    // consumed. Keep these rewrites here so expression emission can stay focused
    // on value contexts.
    match &expr.kind {
        HirExprKind::Block(block) => {
            generate_stmt_block(codegen, block, types, w, current_return_ty)?;
            w.newline();
            Ok(())
        }
        HirExprKind::MethodCall(receiver, method, args)
            if matches!(receiver.kind, HirExprKind::Path(_))
                && matches!(codegen.resolve_symbol(*method), "appende" | "adde") =>
        {
            let HirExprKind::Path(def_id) = receiver.kind else {
                unreachable!()
            };
            let name = codegen.resolve_def(def_id);
            w.write(name);
            w.write(" = append(");
            w.write(name);
            for arg in args {
                w.write(", ");
                generate_expr(codegen, &arg.expr, types, w)?;
            }
            w.write(")");
            w.newline();
            Ok(())
        }
        HirExprKind::MethodCall(receiver, method, _args)
            if matches!(receiver.kind, HirExprKind::Path(_))
                && matches!(codegen.resolve_symbol(*method), "inverte") =>
        {
            let HirExprKind::Path(def_id) = receiver.kind else {
                unreachable!()
            };
            let name = codegen.resolve_def(def_id);
            w.write("for i, j := 0, len(");
            w.write(name);
            w.write(")-1; i < j; i, j = i+1, j-1 { ");
            w.write(name);
            w.write("[i], ");
            w.write(name);
            w.write("[j] = ");
            w.write(name);
            w.write("[j], ");
            w.write(name);
            w.write("[i] }");
            w.newline();
            Ok(())
        }
        HirExprKind::Si { cond, then_block, then_catch: None, else_block } => {
            w.write("if ");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ");
            generate_stmt_block(codegen, then_block, types, w, current_return_ty)?;
            if let Some(else_block) = else_block {
                w.write(" else ");
                generate_stmt_block(codegen, else_block, types, w, current_return_ty)?;
            }
            w.newline();
            Ok(())
        }
        HirExprKind::Si { then_catch: Some(_), .. } | HirExprKind::Handled { .. } => {
            Err(crate::codegen::CodegenError {
                message: "structured cape handlers are not emitted by Go codegen in Phase 5C".to_owned(),
            })
        }
        HirExprKind::Loop(block) => {
            w.write("for ");
            generate_stmt_block(codegen, block, types, w, current_return_ty)?;
            w.newline();
            Ok(())
        }
        HirExprKind::Dum(cond, block) => {
            w.write("for ");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ");
            generate_stmt_block(codegen, block, types, w, current_return_ty)?;
            w.newline();
            Ok(())
        }
        HirExprKind::Itera(mode, def_id, _binding_name, iter, block) => {
            w.write("for ");
            match mode {
                crate::hir::HirIteraMode::De => {
                    w.write(codegen.resolve_def(*def_id));
                    w.write(" := range ");
                }
                crate::hir::HirIteraMode::Ex | crate::hir::HirIteraMode::Pro => {
                    w.write("_, ");
                    w.write(codegen.resolve_def(*def_id));
                    w.write(" := range ");
                }
            }
            generate_expr(codegen, iter, types, w)?;
            w.write(" ");
            generate_stmt_block(codegen, block, types, w, current_return_ty)?;
            w.newline();
            Ok(())
        }
        HirExprKind::Discerne(scrutinees, arms) => generate_discerne_stmt(codegen, scrutinees, arms, types, w),
        _ => {
            generate_expr(codegen, expr, types, w)?;
            w.newline();
            Ok(())
        }
    }
}

fn generate_discerne_stmt(
    codegen: &GoCodegen<'_>,
    scrutinees: &[crate::hir::HirExpr],
    arms: &[crate::hir::HirCasuArm],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Go has separate convenient forms for value switches and type switches.
    // Variant patterns select the type-switch path; literal/wildcard/binding
    // patterns use an ordinary value switch.
    if scrutinees.len() != 1 {
        return Err(CodegenError {
            message: "multi-scrutinee discerne is not yet supported for Go codegen".to_owned(),
        });
    }

    if arms
        .iter()
        .flat_map(|arm| arm.patterns.iter())
        .any(matches_variant_pattern)
    {
        return generate_variant_discerne_stmt(codegen, &scrutinees[0], arms, types, w);
    }

    let scrutinee_name = "__radixMatch";
    w.write("switch ");
    w.write(scrutinee_name);
    w.write(" := ");
    generate_expr(codegen, &scrutinees[0], types, w)?;
    w.write("; ");
    w.write(scrutinee_name);
    w.writeln(" {");

    for arm in arms {
        if arm.patterns.len() != 1 {
            return Err(CodegenError {
                message: "multi-pattern discerne arms are not yet supported for Go codegen".to_owned(),
            });
        }

        let mut result = Ok(());
        w.indented(|w| {
            result = generate_non_variant_discerne_arm(codegen, &arm.patterns[0], &arm.body, types, scrutinee_name, w);
        });
        result?;

        if pattern_is_catch_all(&arm.patterns[0]) {
            break;
        }
    }
    w.write("}");
    w.newline();
    Ok(())
}

fn generate_non_variant_discerne_arm(
    codegen: &GoCodegen<'_>,
    pattern: &HirPattern,
    body: &crate::hir::HirExpr,
    types: &TypeTable,
    scrutinee_name: &str,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match pattern {
        HirPattern::Literal(lit) => {
            w.write("case ");
            write_literal(codegen, lit, w);
            w.writeln(":");
            let mut result = Ok(());
            w.indented(|w| {
                result = generate_expr_stmt(codegen, body, types, w, None);
            });
            result
        }
        HirPattern::Wildcard => {
            w.writeln("default:");
            let mut result = Ok(());
            w.indented(|w| {
                result = generate_expr_stmt(codegen, body, types, w, None);
            });
            result
        }
        HirPattern::Binding(_, name) => {
            w.writeln("default:");
            let mut result = Ok(());
            w.indented(|w| {
                w.write(codegen.resolve_symbol(*name));
                w.write(" := ");
                w.writeln(scrutinee_name);
                result = generate_expr_stmt(codegen, body, types, w, None);
            });
            result
        }
        HirPattern::Alias(_, alias, inner) => {
            generate_non_variant_discerne_header(codegen, inner, w)?;
            let mut result = Ok(());
            w.indented(|w| {
                w.write(codegen.resolve_symbol(*alias));
                w.write(" := ");
                w.writeln(scrutinee_name);
                if let HirPattern::Binding(_, name) = inner.as_ref() {
                    w.write(codegen.resolve_symbol(*name));
                    w.write(" := ");
                    w.writeln(scrutinee_name);
                }
                result = generate_expr_stmt(codegen, body, types, w, None);
            });
            result
        }
        HirPattern::Variant(_, _) => {
            Err(CodegenError { message: "variant discerne pattern reached non-variant Go codegen path".to_owned() })
        }
    }
}

fn generate_non_variant_discerne_header(
    codegen: &GoCodegen<'_>,
    pattern: &HirPattern,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match pattern {
        HirPattern::Literal(lit) => {
            w.write("case ");
            write_literal(codegen, lit, w);
            w.writeln(":");
            Ok(())
        }
        HirPattern::Wildcard | HirPattern::Binding(_, _) => {
            w.writeln("default:");
            Ok(())
        }
        HirPattern::Alias(_, _, inner) => generate_non_variant_discerne_header(codegen, inner, w),
        HirPattern::Variant(_, _) => {
            Err(CodegenError { message: "variant discerne pattern reached non-variant Go codegen path".to_owned() })
        }
    }
}

fn generate_variant_discerne_stmt(
    codegen: &GoCodegen<'_>,
    scrutinee: &crate::hir::HirExpr,
    arms: &[crate::hir::HirCasuArm],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Variant enums lower to concrete structs behind an interface. Matching is
    // therefore a Go type switch, with `__radixCase` introduced only when an arm
    // needs access to the concrete variant value.
    let needs_case_binding = arms
        .iter()
        .flat_map(|arm| arm.patterns.iter())
        .any(pattern_needs_case_binding);
    if needs_case_binding {
        w.write("switch __radixCase := any(");
        generate_expr(codegen, scrutinee, types, w)?;
        w.writeln(").(type) {");
    } else {
        w.write("switch any(");
        generate_expr(codegen, scrutinee, types, w)?;
        w.writeln(").(type) {");
    }
    let mut result = Ok(());
    w.indented(|w| {
        for arm in arms {
            if result.is_err() {
                return;
            }
            if arm.patterns.len() != 1 {
                result = Err(CodegenError {
                    message: "multi-pattern discerne arms are not yet supported for Go codegen".to_owned(),
                });
                return;
            }
            result = generate_variant_discerne_arm(codegen, &arm.patterns[0], &arm.body, types, w);
        }
        if result.is_ok() {
            w.writeln("default:");
            w.indented(|w| w.writeln(r#"panic("non-exhaustive discerne")"#));
        }
    });
    result?;
    w.write("}");
    w.newline();
    Ok(())
}

fn generate_variant_discerne_arm(
    codegen: &GoCodegen<'_>,
    pattern: &HirPattern,
    body: &crate::hir::HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match pattern {
        HirPattern::Wildcard | HirPattern::Binding(_, _) => {
            w.writeln("default:");
            w.indented(|w| {
                let _ = generate_expr_stmt(codegen, body, types, w, None);
            });
            Ok(())
        }
        HirPattern::Alias(_, alias, inner) => match inner.as_ref() {
            HirPattern::Variant(def_id, bindings) => {
                write_variant_case_header(codegen, *def_id, w)?;
                let mut result = Ok(());
                w.indented(|w| {
                    w.write(codegen.resolve_symbol(*alias));
                    w.writeln(" := __radixCase");
                    if result.is_err() {
                        return;
                    }
                    result = write_variant_field_bindings(codegen, *def_id, bindings, w);
                    if result.is_err() {
                        return;
                    }
                    result = generate_expr_stmt(codegen, body, types, w, None);
                });
                result
            }
            _ => Err(CodegenError { message: "alias discerne patterns must wrap a variant in Go codegen".to_owned() }),
        },
        HirPattern::Variant(def_id, bindings) => {
            write_variant_case_header(codegen, *def_id, w)?;
            let mut result = Ok(());
            w.indented(|w| {
                if result.is_err() {
                    return;
                }
                result = write_variant_field_bindings(codegen, *def_id, bindings, w);
                if result.is_err() {
                    return;
                }
                result = generate_expr_stmt(codegen, body, types, w, None);
            });
            result
        }
        HirPattern::Literal(_) => Err(CodegenError {
            message: "literal discerne patterns cannot be mixed with variant patterns in Go codegen".to_owned(),
        }),
    }
}

fn write_variant_case_header(
    codegen: &GoCodegen<'_>,
    def_id: crate::hir::DefId,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("case ");
    w.write(codegen.resolve_def(def_id));
    w.writeln(":");
    Ok(())
}

fn write_variant_field_bindings(
    codegen: &GoCodegen<'_>,
    def_id: crate::hir::DefId,
    bindings: &[HirPattern],
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Variant field metadata is collected before emission. If it is missing or
    // too short, the backend stops instead of guessing field names or silently
    // dropping bindings.
    let Some(fields) = codegen.variant_fields(def_id) else {
        return Err(CodegenError {
            message: format!("missing variant field metadata for {}", codegen.resolve_def(def_id)),
        });
    };
    if bindings.len() > fields.len() {
        return Err(CodegenError {
            message: format!(
                "variant pattern for {} binds {} fields but variant has only {}",
                codegen.resolve_def(def_id),
                bindings.len(),
                fields.len()
            ),
        });
    }

    for (pattern, field) in bindings.iter().zip(fields.iter()) {
        write_variant_binding_pattern(codegen, pattern, *field, w)?;
    }
    Ok(())
}

fn write_variant_binding_pattern(
    codegen: &GoCodegen<'_>,
    pattern: &HirPattern,
    field: crate::lexer::Symbol,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match pattern {
        HirPattern::Wildcard => Ok(()),
        HirPattern::Binding(_, name) => {
            w.write(codegen.resolve_symbol(*name));
            w.write(" := __radixCase.");
            w.writeln(&field_name_to_go(codegen.resolve_symbol(field)));
            Ok(())
        }
        HirPattern::Alias(_, _alias, inner) => {
            write_variant_binding_pattern(codegen, inner, field, w)?;
            Ok(())
        }
        other => Err(CodegenError {
            message: format!("unsupported nested variant binding pattern in Go codegen: {:?}", other),
        }),
    }
}

fn matches_variant_pattern(pattern: &HirPattern) -> bool {
    match pattern {
        HirPattern::Variant(_, _) => true,
        HirPattern::Alias(_, _, inner) => matches_variant_pattern(inner),
        HirPattern::Wildcard | HirPattern::Binding(_, _) | HirPattern::Literal(_) => false,
    }
}

fn pattern_needs_case_binding(pattern: &HirPattern) -> bool {
    match pattern {
        HirPattern::Binding(_, _) => true,
        HirPattern::Alias(_, _, _) => true,
        HirPattern::Variant(_, bindings) => !bindings.is_empty(),
        HirPattern::Wildcard | HirPattern::Literal(_) => false,
    }
}

fn pattern_is_catch_all(pattern: &HirPattern) -> bool {
    match pattern {
        HirPattern::Wildcard | HirPattern::Binding(_, _) => true,
        HirPattern::Alias(_, _, inner) => pattern_is_catch_all(inner),
        HirPattern::Literal(_) | HirPattern::Variant(_, _) => false,
    }
}

fn field_name_to_go(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn write_literal(codegen: &GoCodegen<'_>, literal: &crate::hir::HirLiteral, w: &mut CodeWriter) {
    match literal {
        crate::hir::HirLiteral::Int(v) => w.write(&v.to_string()),
        crate::hir::HirLiteral::Float(v) => w.write(&v.to_string()),
        crate::hir::HirLiteral::String(sym) => w.write(&format!("{:?}", codegen.resolve_symbol(*sym))),
        crate::hir::HirLiteral::Regex(pattern, flags) => {
            w.write("regexp.MustCompile(`");
            w.write(codegen.resolve_symbol(*pattern));
            if let Some(flags) = flags {
                w.write("(?");
                w.write(codegen.resolve_symbol(*flags));
                w.write(")");
            }
            w.write("`)");
        }
        crate::hir::HirLiteral::Bool(v) => w.write(if *v { "true" } else { "false" }),
        crate::hir::HirLiteral::Nil => w.write("nil"),
    }
}
