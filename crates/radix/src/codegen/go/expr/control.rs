//! Control-expression lowering for Go.
//!
//! Faber permits several control forms in expression position. Go does not, so
//! this file emits immediately invoked function expressions to create a value
//! boundary around blocks, branches, loops, assertions, and panic recovery.
//! The wrappers are target compromises: they preserve expression placement and
//! local control flow, while keeping unsupported structured handler forms in
//! the dispatcher fail-closed path.
//!
//! TARGET COMPROMISES
//! ==================
//! - `tempta` is represented with `defer`/`recover`, matching Go panic
//!   mechanics rather than a native exception model.
//! - Branches and blocks return through generated helper functions when the HIR
//!   carries a value type.
//! - Faber ranges currently lower to compact `[]any` descriptors; iteration
//!   over them is not expanded here.

use super::*;
use crate::hir::HirIteraMode;

pub(super) fn generate_tempta_expr(
    codegen: &GoCodegen<'_>,
    body: &crate::hir::HirBlock,
    catch: Option<&crate::hir::HirBlock>,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // WHY: Go has no try/catch. `recover` is the only Go-native way to catch
    // panics inside this expression boundary.
    w.write("func() { defer func() { if r := recover(); r != nil { _ = r ");
    if let Some(catch) = catch {
        w.write("; ");
        stmt::generate_error_binding_block(codegen, catch, "r", types, w)?;
    }
    w.write(" } }(); ");
    stmt::generate_block(codegen, body, types, w, |_| {})?;
    w.write(" }()");
    Ok(())
}

pub(super) fn generate_block_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    block: &crate::hir::HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Blocks in expression position need a synthetic function so statement
    // emission can stay shared with top-level block lowering.
    w.write("func() ");
    w.write(&expr_return_type(expr, types, codegen));
    w.write(" ");
    if let Some(result_ty) = expr.ty {
        stmt::generate_value_block(codegen, block, result_ty, types, w)?;
    } else {
        stmt::generate_block(codegen, block, types, w, |_| {})?;
    }
    w.write("()");
    Ok(())
}

pub(super) fn generate_if_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    cond: &HirExpr,
    then_block: &crate::hir::HirBlock,
    else_block: Option<&crate::hir::HirBlock>,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // WHY: Go's if is a statement, not an expression. Wrap it so callers still
    // receive one Go expression.
    w.write("func() ");
    w.write(&expr_return_type(expr, types, codegen));
    w.write(" { if ");
    generate_expr(codegen, cond, types, w)?;
    w.write(" ");
    if let Some(result_ty) = expr.ty {
        stmt::generate_value_block(codegen, then_block, result_ty, types, w)?;
    } else {
        stmt::generate_block(codegen, then_block, types, w, |_| {})?;
    }
    if let Some(else_block) = else_block {
        w.write(" else ");
        if let Some(result_ty) = expr.ty {
            stmt::generate_value_block(codegen, else_block, result_ty, types, w)?;
        } else {
            stmt::generate_block(codegen, else_block, types, w, |_| {})?;
        }
    } else {
        w.write(" else { return nil }");
    }
    w.write(" }()");
    Ok(())
}

pub(super) fn generate_loop_expr(
    codegen: &GoCodegen<'_>,
    block: &crate::hir::HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("func() { for ");
    stmt::generate_block(codegen, block, types, w, |_| {})?;
    w.write(" }()");
    Ok(())
}

pub(super) fn generate_while_expr(
    codegen: &GoCodegen<'_>,
    cond: &HirExpr,
    block: &crate::hir::HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Faber `itera de` binds map/object keys; `itera ex` and range mode bind values.
    w.write("func() { for ");
    generate_expr(codegen, cond, types, w)?;
    w.write(" ");
    stmt::generate_block(codegen, block, types, w, |_| {})?;
    w.write(" }()");
    Ok(())
}

pub(super) fn generate_for_expr(
    codegen: &GoCodegen<'_>,
    mode: HirIteraMode,
    def_id: crate::hir::DefId,
    iter: &HirExpr,
    block: &crate::hir::HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("func() { for ");
    match mode {
        HirIteraMode::De => {
            w.write(codegen.resolve_def(def_id));
            w.write(", _ := range ");
        }
        HirIteraMode::Ex | HirIteraMode::Ab => {
            w.write("_, ");
            w.write(codegen.resolve_def(def_id));
            w.write(" := range ");
        }
    }
    generate_expr(codegen, iter, types, w)?;
    w.write(" ");
    stmt::generate_block(codegen, block, types, w, |_| {})?;
    w.write(" }()");
    Ok(())
}

pub(super) fn generate_range_expr(
    codegen: &GoCodegen<'_>,
    start: &HirExpr,
    end: &HirExpr,
    step: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // WHY: Go has no range literals. This descriptor is intentionally small and
    // does not expand to a generated sequence at expression emission time.
    w.write("[]any{");
    generate_expr(codegen, start, types, w)?;
    w.write(", ");
    generate_expr(codegen, end, types, w)?;
    if let Some(step) = step {
        w.write(", ");
        generate_expr(codegen, step, types, w)?;
    }
    w.write("}");
    Ok(())
}

pub(super) fn generate_assert_expr(
    codegen: &GoCodegen<'_>,
    cond: &HirExpr,
    message: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("func() { if !(");
    generate_expr(codegen, cond, types, w)?;
    w.write(") { panic(");
    if let Some(message) = message {
        w.write("fmt.Sprint(");
        generate_expr(codegen, message, types, w)?;
        w.write(")");
    } else {
        w.write("\"assertion failed\"");
    }
    w.write(") } }()");
    Ok(())
}
