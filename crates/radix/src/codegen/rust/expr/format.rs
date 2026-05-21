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

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_scribe_expr(
    codegen: &RustCodegen<'_>,
    kind: HirScribeKind,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let macro_name = match kind {
        HirScribeKind::Mone => "eprintln",
        HirScribeKind::Nota | HirScribeKind::Vide | HirScribeKind::Scribe => "println",
    };
    if args.is_empty() {
        w.write(macro_name);
        w.write("!()");
        return Ok(());
    }

    let format = args
        .iter()
        .map(|arg| rust_scribe_format(arg, types))
        .collect::<Vec<_>>()
        .join(" ");
    w.write(macro_name);
    w.write("!(\"");
    w.write(&format);
    w.write("\"");
    for arg in args {
        w.write(", ");
        generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    w.write(")");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_scriptum_expr(
    codegen: &RustCodegen<'_>,
    template: Symbol,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("format!(\"");
    w.write(&rust_format_template(codegen.resolve_symbol(template)));
    w.write("\"");
    for arg in args {
        w.write(", ");
        generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    w.write(")");
    Ok(())
}
