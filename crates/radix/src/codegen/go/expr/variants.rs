//! Variant constructor lowering for Go enum and interface representations.
//!
//! Faber variants are emitted as concrete Go struct values. Those structs are
//! the runtime payloads that can satisfy generated enum or interface surfaces,
//! so this module keeps construction separate from later target assertions.
//! Conversion code can then recognize constructor-shaped expressions and avoid
//! wrapping or asserting values that are already in their concrete variant form.
//!
//! TARGET CONTRACTS
//! ================
//! - A bare variant path or a direct call to a variant path is considered a
//!   variant value expression.
//! - Constructor arguments are assigned positionally to generated, exported Go
//!   fields when field metadata is available.
//! - Missing field metadata does not synthesize names; the constructor emits the
//!   empty composite literal body for that variant shape.

use super::*;
use crate::hir::HirCallArg;

pub(super) fn variant_value_expr(expr: &HirExpr, codegen: &GoCodegen<'_>) -> bool {
    match &expr.kind {
        HirExprKind::Path(def_id) => codegen.is_variant_def(*def_id),
        HirExprKind::Call(callee, _) => match callee.kind {
            HirExprKind::Path(def_id) => codegen.is_variant_def(def_id),
            _ => false,
        },
        _ => false,
    }
}
pub(super) fn generate_variant_constructor(
    codegen: &GoCodegen<'_>,
    def_id: crate::hir::DefId,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write(codegen.resolve_def(def_id));
    w.write("{");
    if let Some(fields) = codegen.variant_fields(def_id) {
        // INVARIANT: Variant field order is the ABI between HIR constructor
        // arguments and the generated Go struct payload.
        for (idx, arg) in args.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            if let Some(field) = fields.get(idx) {
                w.write(&capitalize(codegen.resolve_symbol(*field)));
                w.write(": ");
            }
            generate_expr(codegen, &arg.expr, types, w)?;
        }
    }
    w.write("}");
    Ok(())
}
