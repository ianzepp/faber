use super::*;
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
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write(codegen.resolve_def(def_id));
    w.write("{");
    if let Some(fields) = codegen.variant_fields(def_id) {
        for (idx, arg) in args.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            if let Some(field) = fields.get(idx) {
                w.write(&capitalize(codegen.resolve_symbol(*field)));
                w.write(": ");
            }
            generate_expr(codegen, arg, types, w)?;
        }
    }
    w.write("}");
    Ok(())
}
