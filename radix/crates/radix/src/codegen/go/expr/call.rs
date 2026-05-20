use super::*;
pub(super) fn generate_call_expr(
    codegen: &GoCodegen<'_>,
    callee: &HirExpr,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    if let HirExprKind::Path(def_id) = callee.kind {
        if codegen.is_variant_def(def_id) {
            return generate_variant_constructor(codegen, def_id, args, types, w);
        }
    }
    if try_generate_spread_call_recovery(codegen, callee, args, types, w)? {
        return Ok(());
    }
    if try_generate_intrinsic_call(codegen, callee, args, types, w)? {
        return Ok(());
    }
    generate_expr(codegen, callee, types, w)?;
    w.write("(");
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, arg, types, w)?;
    }
    w.write(")");
    Ok(())
}

pub(super) fn generate_method_call_expr(
    codegen: &GoCodegen<'_>,
    receiver: &HirExpr,
    method: crate::lexer::Symbol,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    if try_generate_translated_method_call(codegen, receiver, method, args, types, w)? {
        return Ok(());
    }
    write_method_receiver(codegen, receiver, types, w)?;
    w.write(".");
    w.write(&capitalize(codegen.resolve_symbol(method)));
    w.write("(");
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, arg, types, w)?;
    }
    w.write(")");
    Ok(())
}

pub(super) fn write_method_receiver(
    codegen: &GoCodegen<'_>,
    receiver: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    if let HirExprKind::Path(def_id) = receiver.kind {
        if codegen.is_struct_def(def_id) {
            w.write("self");
            return Ok(());
        }
    }

    let receiver_ty = receiver
        .ty
        .map(|ty| normalize_receiver_type(types.get(ty), types));
    if matches!(receiver_ty, Some(Type::Struct(_))) && !receiver_is_addressable(receiver) {
        let go_ty = expr_return_type(receiver, types, codegen);
        w.write("func() *");
        w.write(&go_ty);
        w.write(" { v := ");
        generate_expr(codegen, receiver, types, w)?;
        w.write("; return &v }()");
        return Ok(());
    }

    generate_expr(codegen, receiver, types, w)
}

pub(super) fn receiver_is_addressable(receiver: &HirExpr) -> bool {
    matches!(
        receiver.kind,
        HirExprKind::Path(_)
            | HirExprKind::Field(_, _)
            | HirExprKind::Index(_, _)
            | HirExprKind::Deref(_)
            | HirExprKind::NonNull(_, _)
    )
}
pub(super) fn try_generate_translated_method_call(
    codegen: &GoCodegen<'_>,
    receiver: &HirExpr,
    method: crate::lexer::Symbol,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    let method_name = codegen.resolve_symbol(method);
    let receiver_type = receiver
        .ty
        .map(|ty| normalize_receiver_type(types.get(ty), types));
    let is_lista = matches!(receiver_type, Some(Type::Array(_)));
    let is_textus = matches!(receiver_type, Some(Type::Primitive(Primitive::Textus)));
    let list_elem_ty = match receiver_type {
        Some(Type::Array(elem_ty)) => Some(*elem_ty),
        _ => None,
    };

    if method_name == "longitudo" && args.is_empty() && (is_lista || is_textus) {
        w.write("len(");
        generate_expr(codegen, receiver, types, w)?;
        w.write(")");
        return Ok(true);
    }

    if method_name == "accipe" && args.len() == 1 && (is_lista || is_textus) {
        generate_expr(codegen, receiver, types, w)?;
        w.write("[");
        generate_expr(codegen, &args[0], types, w)?;
        w.write("]");
        return Ok(true);
    }

    if method_name == "primus" && args.is_empty() && is_lista {
        generate_expr(codegen, receiver, types, w)?;
        w.write("[0]");
        return Ok(true);
    }

    if method_name == "addita" && args.len() == 1 && is_lista {
        let Some(elem_ty) = list_elem_ty else {
            return Ok(false);
        };
        let elem_go_ty = types::type_to_go(codegen, elem_ty, types);
        w.write("func() []");
        w.write(&elem_go_ty);
        w.write(" { src := ");
        generate_expr(codegen, receiver, types, w)?;
        w.write("; out := append([]");
        w.write(&elem_go_ty);
        w.write("{}, src...); out = append(out, ");
        generate_expr_for_go_type(codegen, &args[0], elem_ty, types, w)?;
        w.write("); return out }()");
        return Ok(true);
    }

    if matches!(method_name, "map" | "mappata") && args.len() == 1 && is_lista {
        let Some(out_ty) = args[0]
            .ty
            .and_then(|ty| match normalize_receiver_type(types.get(ty), types) {
                Type::Func(sig) => Some(sig.ret),
                _ => None,
            })
        else {
            return Ok(false);
        };
        let out_go_ty = types::type_to_go(codegen, out_ty, types);
        w.write("func() []");
        w.write(&out_go_ty);
        w.write(" { mapper := ");
        generate_expr(codegen, &args[0], types, w)?;
        w.write("; src := ");
        generate_expr(codegen, receiver, types, w)?;
        w.write("; out := make([]");
        w.write(&out_go_ty);
        w.write(", len(src)); for i, value := range src { out[i] = mapper(value) }; return out }()");
        return Ok(true);
    }

    if matches!(method_name, "filter" | "filtrata") && args.len() == 1 && is_lista {
        let Some(elem_ty) = list_elem_ty else {
            return Ok(false);
        };
        let elem_go_ty = types::type_to_go(codegen, elem_ty, types);
        w.write("func() []");
        w.write(&elem_go_ty);
        w.write(" { pred := ");
        generate_expr(codegen, &args[0], types, w)?;
        w.write("; src := ");
        generate_expr(codegen, receiver, types, w)?;
        w.write("; out := make([]");
        w.write(&elem_go_ty);
        w.write(
            ", 0, len(src)); for _, value := range src { if pred(value) { out = append(out, value) } }; return out }()",
        );
        return Ok(true);
    }

    if method_name == "inversa" && args.is_empty() && is_lista {
        let Some(elem_ty) = list_elem_ty else {
            return Ok(false);
        };
        let elem_go_ty = types::type_to_go(codegen, elem_ty, types);
        w.write("func() []");
        w.write(&elem_go_ty);
        w.write(" { src := ");
        generate_expr(codegen, receiver, types, w)?;
        w.write("; out := append([]");
        w.write(&elem_go_ty);
        w.write("{}, src...); for i, j := 0, len(out)-1; i < j; i, j = i+1, j-1 { out[i], out[j] = out[j], out[i] }; return out }()");
        return Ok(true);
    }

    if method_name == "ordinata" && args.is_empty() && is_lista {
        let Some(elem_ty) = list_elem_ty else {
            return Ok(false);
        };
        let elem_go_ty = types::type_to_go(codegen, elem_ty, types);
        w.write("func() []");
        w.write(&elem_go_ty);
        w.write(" { src := ");
        generate_expr(codegen, receiver, types, w)?;
        w.write("; out := append([]");
        w.write(&elem_go_ty);
        w.write("{}, src...); sort.Slice(out, func(i, j int) bool { return out[i] < out[j] }); return out }()");
        return Ok(true);
    }

    Ok(false)
}
pub(super) fn try_generate_spread_call_recovery(
    codegen: &GoCodegen<'_>,
    callee: &HirExpr,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    let Some(callee_ty) = callee.ty else {
        return Ok(false);
    };
    let Type::Func(sig) = normalize_receiver_type(types.get(callee_ty), types) else {
        return Ok(false);
    };
    if args.len() != 1 || sig.params.len() <= 1 {
        return Ok(false);
    }
    let Some(arg_ty) = args[0].ty else {
        return Ok(false);
    };
    let Type::Array(_) = normalize_receiver_type(types.get(arg_ty), types) else {
        return Ok(false);
    };

    generate_expr(codegen, callee, types, w)?;
    w.write("(");
    for idx in 0..sig.params.len() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, &args[0], types, w)?;
        w.write("[");
        w.write(&idx.to_string());
        w.write("]");
    }
    w.write(")");
    Ok(true)
}
pub(super) fn try_generate_intrinsic_call(
    codegen: &GoCodegen<'_>,
    callee: &HirExpr,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    let HirExprKind::Path(def_id) = callee.kind else {
        return Ok(false);
    };
    let name = codegen.resolve_def(def_id);
    let mapped = match name {
        "scribe" => Some("fmt.Println"),
        "vide" => Some("fmt.Println"),
        "mone" => Some("fmt.Fprintln(os.Stderr, "),
        _ => None,
    };

    let Some(target) = mapped else {
        return Ok(false);
    };

    if name == "mone" {
        // Special: fmt.Fprintln(os.Stderr, args...)
        w.write("fmt.Fprintln(os.Stderr, ");
        for (idx, arg) in args.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            generate_expr(codegen, arg, types, w)?;
        }
        w.write(")");
    } else {
        w.write(target);
        w.write("(");
        for (idx, arg) in args.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            generate_expr(codegen, arg, types, w)?;
        }
        w.write(")");
    }
    Ok(true)
}
