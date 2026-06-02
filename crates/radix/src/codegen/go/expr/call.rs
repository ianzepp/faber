//! Function and method-call lowering for the Go backend.
//!
//! Calls are the densest interop point between Faber semantics and Go surface
//! syntax. This file handles variant constructors before ordinary call syntax,
//! recovers tuple-like spread calls when semantic types make that safe, maps
//! builtin intrinsics to Go runtime functions, and translates selected stdlib
//! collection methods into Go slices.
//!
//! INVARIANTS
//! ==========
//! - Receiver shape is explicit: struct-definition paths become `self`, and
//!   non-addressable struct values are copied into a temporary so pointer
//!   receiver methods can still be called.
//! - Method translations only fire when receiver type and arity match known
//!   contracts; otherwise emission falls back to a capitalized Go method call.
//! - Spread-call recovery is type-driven and only applies to one array argument
//!   feeding a multi-parameter function.

use super::*;
use crate::hir::HirCallArg;

pub(super) fn generate_call_expr(
    codegen: &GoCodegen<'_>,
    callee: &HirExpr,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    if let HirExprKind::Path(def_id) = callee.kind {
        if codegen.is_variant_def(def_id) {
            // Variant names in callee position construct tagged values rather
            // than call ordinary Go functions.
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
    if let HirExprKind::Path(def_id) = callee.kind {
        if let Some(params) = codegen.function_params(def_id) {
            generate_direct_call_args_with_optional_params(codegen, params, args, types, w)?;
        } else {
            generate_call_args(codegen, args, types, w)?;
        }
    } else {
        generate_call_args(codegen, args, types, w)?;
    }
    w.write(")");
    Ok(())
}

pub(super) fn generate_method_call_expr(
    codegen: &GoCodegen<'_>,
    receiver: &HirExpr,
    method: crate::lexer::Symbol,
    args: &[HirCallArg],
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
        generate_expr(codegen, &arg.expr, types, w)?;
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
            // Inside generated methods, a Faber struct path denotes the active
            // receiver value.
            w.write("self");
            return Ok(());
        }
    }

    let receiver_ty = receiver
        .ty
        .map(|ty| normalize_receiver_type(types.get(ty), types));
    if matches!(receiver_ty, Some(Type::Struct(_))) && !receiver_is_addressable(receiver) {
        // Go pointer receiver methods require an addressable receiver. For
        // computed structs, materialize a local copy and call through its
        // address instead of changing method signatures.
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
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    // These translations are stdlib/runtime contracts, not general method
    // dispatch. Keep them guarded by receiver kind and arity so unknown methods
    // still lower through the normal Go receiver path.
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
        generate_expr(codegen, &args[0].expr, types, w)?;
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
        generate_expr_for_go_type(codegen, &args[0].expr, elem_ty, types, w)?;
        w.write("); return out }()");
        return Ok(true);
    }

    if matches!(method_name, "map" | "mappata") && args.len() == 1 && is_lista {
        let Some(out_ty) = args[0]
            .expr
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
        generate_expr(codegen, &args[0].expr, types, w)?;
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
        generate_expr(codegen, &args[0].expr, types, w)?;
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
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    // Parser/HIR may present a spread-like call as one array argument. When the
    // callee type proves a fixed multi-parameter function, recover the intended
    // positional call by indexing the array.
    let Some(callee_ty) = callee.ty else {
        return Ok(false);
    };
    let Type::Func(sig) = normalize_receiver_type(types.get(callee_ty), types) else {
        return Ok(false);
    };
    if args.len() != 1 || !args[0].spread || sig.params.len() <= 1 {
        return Ok(false);
    }
    let Some(arg_ty) = args[0].expr.ty else {
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
        generate_expr(codegen, &args[0].expr, types, w)?;
        w.write("[");
        w.write(&idx.to_string());
        w.write("]");
    }
    w.write(")");
    Ok(true)
}

fn generate_direct_call_args_with_optional_params(
    codegen: &GoCodegen<'_>,
    params: &[crate::codegen::go::FunctionParamInfo<'_>],
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    for (idx, param) in params.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }

        if let Some(arg) = args.get(idx) {
            if param.optional && param.default.is_none() {
                generate_option_wrapped_expr(codegen, &arg.expr, param.ty, types, w)?;
            } else {
                generate_expr_for_go_type(codegen, &arg.expr, param.ty, types, w)?;
            }
        } else if let Some(default) = param.default {
            generate_expr_for_go_type(codegen, default, param.ty, types, w)?;
        } else if param.optional {
            w.write("nil");
        }
    }
    Ok(())
}

fn generate_call_args(
    codegen: &GoCodegen<'_>,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, &arg.expr, types, w)?;
    }
    Ok(())
}

pub(super) fn try_generate_intrinsic_call(
    codegen: &GoCodegen<'_>,
    callee: &HirExpr,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    // Intrinsics are target shims for compiler-known functions. They are kept
    // here instead of in ordinary name resolution because their Go spellings may
    // include packages or pre-bound arguments such as stderr.
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
            generate_expr(codegen, &arg.expr, types, w)?;
        }
        w.write(")");
    } else {
        w.write(target);
        w.write("(");
        for (idx, arg) in args.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            generate_expr(codegen, &arg.expr, types, w)?;
        }
        w.write(")");
    }
    Ok(true)
}
