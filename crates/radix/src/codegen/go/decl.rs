//! Declaration emission for the Go backend.
//!
//! This file maps top-level HIR declarations to Go declarations while relying
//! on `mod.rs` for backend-wide catalogs and `types.rs` for type spelling. It
//! handles the places where Faber declaration semantics have to choose a Go
//! representation: exported struct fields, pointer receiver methods, interface
//! method sets, enum variants as sealed interface implementors, aliases,
//! constants, and source imports.
//!
//! TARGET CONTRACTS
//! ================
//! - Function and method bodies are emitted from already-analyzed HIR; this
//!   layer adds Go-specific preludes for unused parameters and tail returns.
//! - Struct and enum fields are exported because generated expressions and
//!   pattern bindings may cross declaration boundaries inside the generated
//!   package.
//! - Methods use pointer receivers so mutation-oriented Faber methods can share
//!   one Go representation with read-only methods.
//! - Faber stdlib imports such as `norma/*` are consumed by the compiler and do
//!   not currently correspond to emitted Go packages.

use super::stmt::generate_stmt;
use super::types::type_to_go;
use super::{
    expr::{generate_expr, generate_expr_for_go_type},
    CodeWriter, CodegenError, GoCodegen,
};
use crate::hir::*;
use crate::semantic::{Primitive, TypeTable};

pub fn generate_function(
    codegen: &GoCodegen<'_>,
    func: &HirFunction,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("func ");
    w.write(&go_function_name(codegen.resolve_symbol(func.name)));
    generate_type_params(codegen, &func.type_params, w);
    generate_params(codegen, &func.params, types, w);

    if let Some(ret_ty) = func.ret_ty {
        let ret = type_to_go(codegen, ret_ty, types);
        if !ret.is_empty() {
            w.write(" ");
            w.write(&ret);
        }
    }

    if let Some(body) = &func.body {
        w.write(" ");
        let previous_return_ty = codegen.replace_current_return_ty(func.ret_ty);
        let result = generate_block_with_prelude(codegen, body, types, w, &func.params, func.ret_ty);
        codegen.replace_current_return_ty(previous_return_ty);
        result?;
    }
    w.newline();
    Ok(())
}

/// Emit a Faber `genus` as a Go struct plus pointer-receiver methods.
///
/// Fields are capitalized for package-local export. `sponte` fields are written
/// as pointers so nil can represent absence without adding a runtime option
/// wrapper that Go code would not naturally expect.
pub fn generate_struct(
    codegen: &GoCodegen<'_>,
    strukt: &HirStruct,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("type ");
    w.write(codegen.resolve_symbol(strukt.name));
    generate_type_params(codegen, &strukt.type_params, w);
    w.writeln(" struct {");
    w.indented(|w| {
        for field in &strukt.fields {
            // WHY: Go exports fields via uppercase. Generated constructors,
            // conversions, and pattern helpers all assume this field spelling.
            w.write(&capitalize(codegen.resolve_symbol(field.name)));
            w.write(" ");
            if field.sponte {
                w.write("*");
            }
            w.writeln(&type_to_go(codegen, field.ty, types));
        }
    });
    w.writeln("}");

    // Pointer receivers preserve one representation for methods that mutate
    // `self` and methods that only inspect it.
    for method in &strukt.methods {
        w.newline();
        let receiver_name = codegen.resolve_symbol(strukt.name);
        w.write("func (self *");
        w.write(receiver_name);
        w.write(") ");
        w.write(&capitalize(codegen.resolve_symbol(method.func.name)));
        generate_type_params(codegen, &method.func.type_params, w);
        generate_params(codegen, &method.func.params, types, w);

        if let Some(ret_ty) = method.func.ret_ty {
            let ret = type_to_go(codegen, ret_ty, types);
            if !ret.is_empty() {
                w.write(" ");
                w.write(&ret);
            }
        }

        if let Some(body) = &method.func.body {
            w.write(" ");
            let mut prelude_params = Vec::with_capacity(method.func.params.len() + 1);
            prelude_params.push((None, "self"));
            prelude_params.extend(
                method
                    .func
                    .params
                    .iter()
                    .map(|param| (Some(param.def_id), codegen.resolve_symbol(param.name))),
            );
            let previous_return_ty = codegen.replace_current_return_ty(method.func.ret_ty);
            let result =
                generate_block_with_custom_prelude(codegen, body, types, w, &prelude_params, method.func.ret_ty);
            codegen.replace_current_return_ty(previous_return_ty);
            result?;
        }
        w.newline();
    }

    Ok(())
}

fn generate_block_with_prelude(
    codegen: &GoCodegen<'_>,
    body: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    params: &[HirParam],
    ret_ty: Option<crate::semantic::TypeId>,
) -> Result<(), CodegenError> {
    let prelude_params: Vec<(Option<DefId>, &str)> = params
        .iter()
        .map(|param| (Some(param.def_id), codegen.resolve_symbol(param.name)))
        .collect();
    generate_block_with_custom_prelude(codegen, body, types, w, &prelude_params, ret_ty)
}

fn generate_block_with_custom_prelude(
    codegen: &GoCodegen<'_>,
    body: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    params: &[(Option<DefId>, &str)],
    ret_ty: Option<crate::semantic::TypeId>,
) -> Result<(), CodegenError> {
    // Go rejects unused parameters and locals. The prelude records intentional
    // non-use after the backend's simple path-use pass; it does not prove
    // semantic liveness.
    let needs_nil_return = ret_ty
        .map(|ret_ty| matches!(types.get(ret_ty), crate::semantic::Type::Primitive(Primitive::Nihil)))
        .unwrap_or(false);

    w.writeln("{");
    let mut result = Ok(());
    w.indented(|w| {
        for (def_id, name) in params {
            if def_id
                .map(|def_id| codegen.is_used(def_id))
                .unwrap_or(false)
            {
                continue;
            }
            w.write("_ = ");
            w.writeln(name);
        }
        for stmt in &body.stmts {
            if result.is_err() {
                return;
            }
            result = generate_stmt(codegen, stmt, types, w, ret_ty);
        }
        if result.is_ok() {
            if let Some(expr) = &body.expr {
                // Faber block tail expressions become Go returns at function
                // and method boundaries. Statement-position blocks use the
                // statement emitter instead.
                w.write("return ");
                if let Some(ret_ty) = ret_ty {
                    result = generate_expr_for_go_type(codegen, expr, ret_ty, types, w);
                } else {
                    result = generate_expr(codegen, expr, types, w);
                }
                w.newline();
            } else if needs_nil_return {
                // `nihil` returns are represented as `any` in Go, so an empty
                // Faber body needs an explicit nil return.
                w.writeln("return nil");
            }
        }
    });
    result?;
    w.write("}");
    Ok(())
}

/// Emit a Faber trait/interface as a Go interface method set.
///
/// Method names are exported for the same reason struct fields are exported:
/// generated package-local values should satisfy and call the interface without
/// depending on unexported names from another generated declaration.
pub fn generate_interface(
    codegen: &GoCodegen<'_>,
    interface: &HirInterface,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("type ");
    w.write(codegen.resolve_symbol(interface.name));
    generate_type_params(codegen, &interface.type_params, w);
    w.writeln(" interface {");
    w.indented(|w| {
        for method in &interface.methods {
            w.write(&capitalize(codegen.resolve_symbol(method.name)));
            w.write("(");
            for (idx, param) in method.params.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(codegen.resolve_symbol(param.name));
                w.write(" ");
                w.write(&type_to_go(codegen, param.ty, types));
            }
            w.write(")");
            if let Some(ret_ty) = method.ret_ty {
                let ret = type_to_go(codegen, ret_ty, types);
                if !ret.is_empty() {
                    w.write(" ");
                    w.write(&ret);
                }
            }
            w.newline();
        }
    });
    w.writeln("}");
    Ok(())
}

/// Emit an enum as a Go interface + variant structs.
///
/// WHY: Go lacks algebraic data types. The idiomatic pattern is an unexported
/// marker-method interface with one struct per variant, matching how the Go
/// standard library models sum types (e.g., ast.Expr, ast.Stmt). Variant fields
/// are exported so constructors and `discerne` bindings can read them after a
/// type switch.
pub fn generate_enum(
    codegen: &GoCodegen<'_>,
    enum_item: &HirEnum,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let enum_name = codegen.resolve_symbol(enum_item.name);

    // The marker method keeps unrelated structs from satisfying the enum
    // interface accidentally within generated code.
    w.write("type ");
    w.write(enum_name);
    generate_type_params(codegen, &enum_item.type_params, w);
    w.writeln(" interface {");
    w.indented(|w| {
        // Marker method — lowercase to keep it unexported
        w.write("is");
        w.write(enum_name);
        w.writeln("()");
    });
    w.writeln("}");

    // Each variant is a concrete Go value. Payload field order is also recorded
    // by `GoCodegen` so positional Faber variant constructors can be emitted.
    for variant in &enum_item.variants {
        let variant_name = codegen.resolve_symbol(variant.name);
        w.newline();
        w.write("type ");
        w.write(variant_name);
        w.writeln(" struct {");
        w.indented(|w| {
            for field in &variant.fields {
                w.write(&capitalize(codegen.resolve_symbol(field.name)));
                w.write(" ");
                w.writeln(&type_to_go(codegen, field.ty, types));
            }
        });
        w.writeln("}");

        w.write("func (");
        w.write(variant_name);
        w.write(") is");
        w.write(enum_name);
        w.writeln("() {}");
    }

    Ok(())
}

/// Emit a Faber type alias as a Go alias declaration.
///
/// The alias preserves the named surface for downstream generated code; it does
/// not introduce additional runtime validation.
pub fn generate_type_alias(
    codegen: &GoCodegen<'_>,
    alias: &HirTypeAlias,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("type ");
    w.write(codegen.resolve_symbol(alias.name));
    w.write(" = ");
    w.writeln(&type_to_go(codegen, alias.ty, types));
    Ok(())
}

/// Emit a Faber constant declaration.
///
/// The backend currently writes `var` rather than Go `const` because Faber
/// constant initializers can lower to expressions that are not Go compile-time
/// constants.
pub fn generate_const(
    codegen: &GoCodegen<'_>,
    constant: &HirConst,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("var ");
    w.write(codegen.resolve_symbol(constant.name));
    if let Some(ty) = constant.ty {
        w.write(" ");
        w.write(&type_to_go(codegen, ty, types));
    }
    w.write(" = ");
    generate_expr(codegen, &constant.value, types, w)?;
    w.newline();
    Ok(())
}

/// Emit source-level imports that map directly to Go packages.
///
/// Faber stdlib imports are intentionally elided here: the package compiler has
/// already loaded those interfaces as source-level contracts, and this backend
/// does not yet emit one Go package per Faber module.
pub fn generate_import(codegen: &GoCodegen<'_>, import: &HirImport, w: &mut CodeWriter) -> Result<(), CodegenError> {
    let path = codegen.resolve_symbol(import.path);
    // WHY: stdlib imports like norma/* don't map to Go packages yet.
    if path.starts_with("norma/") {
        return Ok(());
    }

    w.write("import ");
    w.write(&format!("{:?}", path));
    w.newline();
    Ok(())
}

fn generate_type_params(codegen: &GoCodegen<'_>, params: &[HirTypeParam], w: &mut CodeWriter) {
    if params.is_empty() {
        return;
    }
    w.write("[");
    for (idx, param) in params.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        w.write(codegen.resolve_symbol(param.name));
        w.write(" any");
    }
    w.write("]");
}

/// Emit Go parameter lists with already-resolved Faber names and Go type text.
fn generate_params(codegen: &GoCodegen<'_>, params: &[HirParam], types: &TypeTable, w: &mut CodeWriter) {
    w.write("(");
    for (idx, param) in params.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        w.write(codegen.resolve_symbol(param.name));
        w.write(" ");
        if param.optional && param.default.is_none() {
            w.write("*");
        }
        w.write(&type_to_go(codegen, param.ty, types));
    }
    w.write(")");
}

/// Capitalize the first character of a string for Go export convention.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
    }
}

/// Sanitize Faber function spellings into Go identifiers.
///
/// Operators and symbolic helper names can reach codegen as function names.
/// Rather than guessing semantic meaning, the backend uses a deterministic
/// declaration-side fallback. Ordinary identifier names pass through unchanged.
fn go_function_name(name: &str) -> String {
    if name
        .chars()
        .enumerate()
        .all(|(idx, ch)| (ch == '_' || ch.is_ascii_alphanumeric()) && (idx > 0 || !ch.is_ascii_digit()))
    {
        return name.to_owned();
    }

    let mut out = String::with_capacity(name.len());
    for (idx, ch) in name.chars().enumerate() {
        if ch == '_' || ch.is_ascii_alphanumeric() {
            if idx == 0 && ch.is_ascii_digit() {
                out.push('_');
            }
            out.push(ch);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }

    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "proba".to_owned()
    } else {
        trimmed.to_owned()
    }
}
