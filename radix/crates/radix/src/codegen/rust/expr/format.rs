use super::*;
pub(super) fn rust_format_template(template: &str) -> String {
    let mut out = String::with_capacity(template.len());
    for ch in template.chars() {
        match ch {
            '{' => out.push_str("{{"),
            '}' => out.push_str("}}"),
            '§' => out.push_str("{}"),
            _ => out.push(ch),
        }
    }
    out
}
pub(super) fn rust_scribe_format(expr: &HirExpr, types: &TypeTable) -> &'static str {
    if matches!(
        expr.kind,
        HirExprKind::Literal(HirLiteral::String(_))
            | HirExprKind::Literal(HirLiteral::Int(_))
            | HirExprKind::Literal(HirLiteral::Float(_))
            | HirExprKind::Literal(HirLiteral::Bool(_))
    ) {
        return "{}";
    }

    match expr.ty.map(|ty| resolve_type(ty, types)) {
        Some(Type::Primitive(Primitive::Textus))
        | Some(Type::Primitive(Primitive::Numerus))
        | Some(Type::Primitive(Primitive::Fractus))
        | Some(Type::Primitive(Primitive::Bivalens))
        | Some(Type::Primitive(Primitive::Vacuum))
        | Some(Type::Primitive(Primitive::Nihil)) => "{}",
        _ => "{:?}",
    }
}
