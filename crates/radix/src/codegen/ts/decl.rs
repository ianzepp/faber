//! TypeScript declaration emission for top-level and member HIR items.
//!
//! This file owns the shape of generated TypeScript declarations: functions,
//! classes, interfaces, discriminated-union enums, aliases, constants, and
//! imports. Expression and statement bodies are delegated to sibling modules so
//! declaration code can concentrate on signatures, member layout, and the
//! target-level compromises needed to represent Faber constructs in TypeScript.
//!
//! GENERATED-CODE TRADE-OFFS
//! =========================
//! The backend emits readable, direct TypeScript rather than a runtime-heavy
//! encoding. For example, Faber enum variants lower to tagged object unions,
//! optional parameters use TypeScript's `?` syntax, and imports are printed as
//! ES module bindings when they still matter after stdlib lowering. These
//! choices preserve the checked program's shape without pretending TypeScript
//! can enforce every semantic guarantee Radix already verified.
//!
//! INVARIANTS
//! ==========
//! - `TypeTable` entries are expected to come from semantic typechecking; this
//!   module translates those types but does not repair missing information.
//! - Name spelling goes through `TsCodegen` so declarations and references use
//!   one target name policy.
//! - Built-in stdlib imports that lower to native TypeScript facilities are
//!   intentionally elided here instead of becoming unresolved module imports.

use super::stmt::generate_block;
use super::types::type_to_ts;
use super::{expr::generate_expr, CodeWriter, CodegenError, TsCodegen};
use crate::hir::*;
use crate::semantic::TypeTable;

/// Emits a TypeScript function declaration or signature from a HIR function.
///
/// Body-less functions are preserved as declarations because interface-like or
/// external surfaces may describe callable contracts without carrying a body in
/// the current compilation unit.
pub fn generate_function(
    codegen: &TsCodegen<'_>,
    func: &HirFunction,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    if func.is_async {
        w.write("async ");
    }
    w.write("function ");
    w.write(codegen.resolve_symbol(func.name));
    generate_type_params(codegen, &func.type_params, w);
    generate_params(codegen, &func.params, types, w);
    if let Some(ret_ty) = func.ret_ty {
        w.write(": ");
        w.write(&type_to_ts(codegen, ret_ty, types));
    }

    if let Some(body) = &func.body {
        w.write(" ");
        generate_block(codegen, body, types, w)?;
    } else {
        w.writeln(";");
    }
    Ok(())
}

/// Emits a HIR struct as a TypeScript class declaration.
///
/// Faber receiver analysis has already decided whether methods are instance or
/// static members. This emitter reflects that decision directly and leaves
/// object-layout validity to earlier semantic passes.
pub fn generate_class(
    codegen: &TsCodegen<'_>,
    strukt: &HirStruct,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("class ");
    w.write(codegen.resolve_symbol(strukt.name));
    generate_type_params(codegen, &strukt.type_params, w);
    if let Some(parent) = strukt.extends {
        w.write(" extends ");
        w.write(codegen.resolve_def(parent));
    }
    if !strukt.implements.is_empty() {
        w.write(" implements ");
        for (idx, iface) in strukt.implements.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            w.write(codegen.resolve_def(*iface));
        }
    }
    w.writeln(" {");
    let mut result = Ok(());
    w.indented(|w| {
        for field in &strukt.fields {
            if result.is_err() {
                return;
            }
            if field.is_static {
                w.write("static ");
            }
            w.write(codegen.resolve_symbol(field.name));
            w.write(": ");
            w.write(&type_to_ts(codegen, field.ty, types));
            if let Some(init) = &field.init {
                w.write(" = ");
                result = generate_expr(codegen, init, types, w);
            }
            w.writeln(";");
        }

        for method in &strukt.methods {
            if result.is_err() {
                return;
            }
            if matches!(method.receiver, HirReceiver::None) {
                w.write("static ");
            }
            if method.func.is_async {
                w.write("async ");
            }
            w.write(codegen.resolve_symbol(method.func.name));
            generate_type_params(codegen, &method.func.type_params, w);
            generate_params(codegen, &method.func.params, types, w);
            if let Some(ret_ty) = method.func.ret_ty {
                w.write(": ");
                w.write(&type_to_ts(codegen, ret_ty, types));
            }
            if let Some(body) = &method.func.body {
                w.write(" ");
                result = generate_block(codegen, body, types, w);
            } else {
                w.writeln(";");
            }
        }
    });
    result?;
    w.writeln("}");
    Ok(())
}

/// Emits a structural interface with method signatures.
///
/// Interfaces carry only contracts, so method bodies are deliberately absent
/// and optional parameters are translated into TypeScript's parameter syntax.
pub fn generate_interface(
    codegen: &TsCodegen<'_>,
    interface: &HirInterface,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("interface ");
    w.write(codegen.resolve_symbol(interface.name));
    generate_type_params(codegen, &interface.type_params, w);
    w.writeln(" {");
    w.indented(|w| {
        for method in &interface.methods {
            w.write(codegen.resolve_symbol(method.name));
            w.write("(");
            for (idx, param) in method.params.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(codegen.resolve_symbol(param.name));
                if param.optional {
                    w.write("?");
                }
                w.write(": ");
                w.write(&type_to_ts(codegen, param.ty, types));
            }
            w.write(")");
            if let Some(ret_ty) = method.ret_ty {
                w.write(": ");
                w.write(&type_to_ts(codegen, ret_ty, types));
            }
            w.writeln(";");
        }
    });
    w.writeln("}");
    Ok(())
}

/// Emits a Faber enum as a tagged TypeScript object union.
///
/// TypeScript has native `enum`, but tagged unions preserve payload shape and
/// line up with the exhaustiveness information Radix computes before codegen.
pub fn generate_enum(
    codegen: &TsCodegen<'_>,
    enum_item: &HirEnum,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("type ");
    w.write(codegen.resolve_symbol(enum_item.name));
    generate_type_params(codegen, &enum_item.type_params, w);
    w.write(" = ");
    for (idx, variant) in enum_item.variants.iter().enumerate() {
        if idx > 0 {
            w.write(" | ");
        }
        w.write("{ tag: ");
        w.write(&format!("{:?}", codegen.resolve_symbol(variant.name)));
        for field in &variant.fields {
            w.write(", ");
            w.write(codegen.resolve_symbol(field.name));
            w.write(": ");
            w.write(&type_to_ts(codegen, field.ty, types));
        }
        w.write(" }");
    }
    w.writeln(";");
    Ok(())
}

/// Emits a named TypeScript alias for a checked HIR type alias.
pub fn generate_type_alias(
    codegen: &TsCodegen<'_>,
    alias: &HirTypeAlias,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("type ");
    w.write(codegen.resolve_symbol(alias.name));
    w.write(" = ");
    w.write(&type_to_ts(codegen, alias.ty, types));
    w.writeln(";");
    Ok(())
}

/// Emits a top-level constant declaration.
///
/// The initializer is generated through expression codegen so target-specific
/// intrinsics and collection forms stay centralized with expression policy.
pub fn generate_const(
    codegen: &TsCodegen<'_>,
    constant: &HirConst,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("const ");
    w.write(codegen.resolve_symbol(constant.name));
    if let Some(ty) = constant.ty {
        w.write(": ");
        w.write(&type_to_ts(codegen, ty, types));
    }
    w.write(" = ");
    generate_expr(codegen, &constant.value, types, w)?;
    w.writeln(";");
    Ok(())
}

/// Emits an ES module import when the import still has a TypeScript artifact.
///
/// Some `norma` modules are source-level contracts whose operations lower to
/// native JavaScript/TypeScript constructs. Those imports are elided here so the
/// generated file does not reference modules that the target runtime does not
/// need to load.
pub fn generate_import(codegen: &TsCodegen<'_>, import: &HirImport, w: &mut CodeWriter) -> Result<(), CodegenError> {
    let path = codegen.resolve_symbol(import.path);
    if matches!(path, "norma/mathesis" | "norma/tempus") {
        return Ok(());
    }

    w.write("import { ");
    for (idx, item) in import.items.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        w.write(codegen.resolve_symbol(item.name));
        if let Some(alias) = item.alias {
            w.write(" as ");
            w.write(codegen.resolve_symbol(alias));
        }
    }
    w.write(" } from ");
    w.write(&format!("{:?}", path));
    w.writeln(";");
    Ok(())
}

fn generate_type_params(codegen: &TsCodegen<'_>, params: &[HirTypeParam], w: &mut CodeWriter) {
    if params.is_empty() {
        return;
    }
    w.write("<");
    for (idx, param) in params.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        w.write(codegen.resolve_symbol(param.name));
    }
    w.write(">");
}

fn generate_params(codegen: &TsCodegen<'_>, params: &[HirParam], types: &TypeTable, w: &mut CodeWriter) {
    w.write("(");
    for (idx, param) in params.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        w.write(codegen.resolve_symbol(param.name));
        if param.optional {
            w.write("?");
        }
        w.write(": ");
        w.write(&type_to_ts(codegen, param.ty, types));
    }
    w.write(")");
}
