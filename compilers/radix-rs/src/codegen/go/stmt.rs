use super::types::type_to_go;
use super::{expr::generate_expr, CodeWriter, CodegenError, GoCodegen};
use crate::hir::{HirBlock, HirExprKind, HirPattern, HirStmt, HirStmtKind};
use crate::semantic::TypeTable;

pub(super) fn generate_prefixed_block<P>(
    codegen: &GoCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    skip_stmts: usize,
    prelude: P,
) -> Result<(), CodegenError>
where
    P: FnOnce(&mut CodeWriter) -> Result<(), CodegenError>,
{
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
            result = generate_stmt(codegen, stmt, types, w);
        }
        if result.is_ok() {
            if let Some(expr) = &block.expr {
                w.write("return ");
                result = generate_expr(codegen, expr, types, w);
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
    generate_prefixed_block(codegen, block, types, w, 0, |w| {
        prelude(w);
        Ok(())
    })
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
        generate_stmt(codegen, stmt, types, w)?;
    }
    if let Some(expr) = &block.expr {
        generate_expr(codegen, expr, types, w)?;
        w.newline();
    }
    Ok(())
}

pub fn generate_stmt(
    codegen: &GoCodegen<'_>,
    stmt: &HirStmt,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match &stmt.kind {
        HirStmtKind::Local(local) => {
            // WHY: Go uses := for short variable declarations with inferred types,
            // and var for explicit types without initializers.
            if let Some(init) = &local.init {
                let name = codegen.resolve_symbol(local.name);
                if matches!(init.kind, HirExprKind::Literal(crate::hir::HirLiteral::Nil)) {
                    w.write("var ");
                    w.write(name);
                    if let Some(ty) = local.ty {
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
                    generate_expr(codegen, init, types, w)?;
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
            generate_expr_stmt(codegen, expr, types, w)?;
        }
        HirStmtKind::Ad(ad) => {
            generate_ad_stmt(codegen, ad, types, w)?;
        }
        HirStmtKind::Redde(expr) => {
            if let Some(expr) = expr {
                w.write("return ");
                generate_expr(codegen, expr, types, w)?;
                w.newline();
            } else {
                w.writeln("return");
            }
        }
        HirStmtKind::Rumpe => w.writeln("break"),
        HirStmtKind::Perge => w.writeln("continue"),
    }
    Ok(())
}

fn generate_ad_stmt(
    codegen: &GoCodegen<'_>,
    ad: &crate::hir::HirAd,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let binding_ty = ad
        .binding
        .as_ref()
        .and_then(|binding| binding.ty)
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
    let Some(crate::hir::HirStmt { kind: HirStmtKind::Local(local), .. }) = block.stmts.first() else {
        return generate_block(codegen, block, types, w, |_| {});
    };

    generate_prefixed_block(codegen, block, types, w, 1, |w| {
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

    generate_prefixed_block(codegen, block, types, w, skip, |w| {
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
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Block(block) => {
            generate_block(codegen, block, types, w, |_| {})?;
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
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
            w.newline();
            Ok(())
        }
        HirExprKind::Si(cond, then_block, else_block) => {
            w.write("if ");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ");
            generate_block(codegen, then_block, types, w, |_| {})?;
            if let Some(else_block) = else_block {
                w.write(" else ");
                generate_block(codegen, else_block, types, w, |_| {})?;
            }
            w.newline();
            Ok(())
        }
        HirExprKind::Loop(block) => {
            w.write("for ");
            generate_block(codegen, block, types, w, |_| {})?;
            w.newline();
            Ok(())
        }
        HirExprKind::Dum(cond, block) => {
            w.write("for ");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ");
            generate_block(codegen, block, types, w, |_| {})?;
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
            generate_block(codegen, block, types, w, |_| {})?;
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
    if scrutinees.len() != 1 {
        return Err(CodegenError {
            message: "multi-scrutinee discerne is not yet supported for Go codegen".to_owned(),
        });
    }

    let mut first = true;
    for arm in arms {
        let mut wrote_branch = false;
        for pattern in &arm.patterns {
            match pattern {
                HirPattern::Wildcard => {
                    if first {
                        w.write("{");
                    } else {
                        w.write(" else {");
                    }
                    w.newline();
                    w.indented(|w| {
                        let _ = generate_expr_stmt(codegen, &arm.body, types, w);
                    });
                    w.write("}");
                    wrote_branch = true;
                }
                HirPattern::Literal(lit) => {
                    if first {
                        w.write("if ");
                    } else {
                        w.write(" else if ");
                    }
                    generate_expr(codegen, &scrutinees[0], types, w)?;
                    w.write(" == ");
                    write_literal(codegen, lit, w);
                    w.write(" {");
                    w.newline();
                    w.indented(|w| {
                        let _ = generate_expr_stmt(codegen, &arm.body, types, w);
                    });
                    w.write("}");
                    wrote_branch = true;
                }
                _ => {}
            }
            if wrote_branch {
                break;
            }
        }
        if wrote_branch {
            first = false;
        }
    }
    w.newline();
    Ok(())
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
