//! Rust Declaration Generation
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Generates Rust item declarations (functions, structs, enums, traits, type aliases,
//! constants) from HIR. Handles error propagation wrapping, reference mode translation,
//! and async function generation.
//!
//! COMPILER PHASE: Codegen (submodule)
//! INPUT: HIR items (HirFunction, HirStruct, etc.)
//! OUTPUT: Rust declaration source text
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Failable functions return Result<T, String>.
//!   WHY: Faber's `iace` requires error propagation; String is simple error type.
//! - Reference modes map to Rust borrow syntax.
//!   WHY: de -> &T, in -> &mut T, ex -> T (moved).
//! - Async functions use `async fn`.
//!   WHY: Direct mapping to Rust's async/await.

use super::super::CodeWriter;
use super::types::type_to_rust;
use super::{CodegenError, RustCodegen};
use crate::hir::*;
use crate::semantic::TypeTable;

/// Generate a Rust function declaration.
///
/// TRANSFORMS:
///   functio salve(textus n) -> fn salve(n: String)
///   Failable function      -> fn f() -> Result<T, String>
///   futura functio f()     -> async fn f()
///
/// TARGET: Rust-specific Result wrapping for failable functions.
pub fn generate_function(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    func: &HirFunction,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let is_failable = codegen.is_failable_def(def_id);

    // Async modifier
    if func.is_async {
        w.write("async ");
    }

    w.write("fn ");
    w.write(codegen.resolve_symbol(func.name));

    // Type parameters
    if !func.type_params.is_empty() {
        w.write("<");
        for (i, param) in func.type_params.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            w.write(codegen.resolve_symbol(param.name));
        }
        w.write(">");
    }

    // Parameters
    w.write("(");
    for (i, param) in func.params.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        w.write(codegen.resolve_symbol(param.name));
        w.write(": ");
        w.write(&type_to_rust(codegen, param.ty, types));
    }
    w.write(")");

    // Return type
    if is_failable {
        w.write(" -> ");
        w.write("Result<");
        if let Some(ret_ty) = func.ret_ty {
            w.write(&type_to_rust(codegen, ret_ty, types));
        } else {
            w.write("()");
        }
        w.write(", String>");
    } else if let Some(ret_ty) = func.ret_ty {
        w.write(" -> ");
        w.write(&type_to_rust(codegen, ret_ty, types));
    }

    // Body
    if let Some(body) = &func.body {
        w.write(" ");
        generate_block(codegen, body, types, w, is_failable, false, true)?;
    } else {
        w.write(";");
    }

    w.newline();
    Ok(())
}

/// Generate a Rust struct declaration.
///
/// TRANSFORMS:
///   genus Person { textus name } -> pub struct Person { pub name: String }
///
/// TARGET: All fields are pub (Faber has no visibility modifiers).
/// NOTE: Static fields are skipped (not supported in Rust structs).
pub fn generate_struct(
    codegen: &RustCodegen<'_>,
    s: &HirStruct,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub struct ");
    w.write(codegen.resolve_symbol(s.name));

    if !s.type_params.is_empty() {
        w.write("<");
        for (i, param) in s.type_params.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            w.write(codegen.resolve_symbol(param.name));
        }
        w.write(">");
    }

    w.writeln(" {");
    w.indented(|w| {
        for field in &s.fields {
            if !field.is_static {
                w.write("pub ");
                w.write(codegen.resolve_symbol(field.name));
                w.write(": ");
                w.write(&type_to_rust(codegen, field.ty, types));
                w.writeln(",");
            }
        }
    });
    w.writeln("}");

    // Generate impl block for methods
    if !s.methods.is_empty() {
        w.newline();
        w.write("impl ");
        w.write(codegen.resolve_symbol(s.name));
        w.writeln(" {");
        w.indented(|w| {
            for method in &s.methods {
                let _ = generate_function(codegen, method.def_id, &method.func, types, w);
            }
        });
        w.writeln("}");
    }

    Ok(())
}

/// Generate a Rust enum declaration.
///
/// TRANSFORMS:
///   discretio Result { Ok { textus val }, Err { textus msg } }
///   -> pub enum Result { Ok { val: String }, Err { msg: String } }
///
/// TARGET: Rust enum with struct-like variants (named fields).
pub fn generate_enum(
    codegen: &RustCodegen<'_>,
    e: &HirEnum,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub enum ");
    w.write(codegen.resolve_symbol(e.name));

    if !e.type_params.is_empty() {
        w.write("<");
        for (i, param) in e.type_params.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            w.write(codegen.resolve_symbol(param.name));
        }
        w.write(">");
    }

    w.writeln(" {");
    w.indented(|w| {
        for variant in &e.variants {
            w.write(codegen.resolve_symbol(variant.name));
            if !variant.fields.is_empty() {
                w.writeln(" {");
                w.indented(|w| {
                    for field in &variant.fields {
                        w.write(codegen.resolve_symbol(field.name));
                        w.write(": ");
                        w.write(&type_to_rust(codegen, field.ty, types));
                        w.writeln(",");
                    }
                });
                w.write("}");
            }
            w.writeln(",");
        }
    });
    w.writeln("}");

    Ok(())
}

/// Generate a Rust trait declaration.
///
/// TRANSFORMS:
///   pactum Display { functio display() } -> pub trait Display { fn display(&self); }
///
/// TARGET: Rust trait with &self receiver (Faber interfaces always have implicit self).
pub fn generate_trait(
    codegen: &RustCodegen<'_>,
    i: &HirInterface,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub trait ");
    w.write(codegen.resolve_symbol(i.name));

    if !i.type_params.is_empty() {
        w.write("<");
        for (idx, param) in i.type_params.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            w.write(codegen.resolve_symbol(param.name));
        }
        w.write(">");
    }

    w.writeln(" {");
    w.indented(|w| {
        for method in &i.methods {
            w.write("fn ");
            w.write(codegen.resolve_symbol(method.name));
            w.write("(");
            w.write("&self");
            for param in &method.params {
                w.write(", ");
                w.write(codegen.resolve_symbol(param.name));
                w.write(": ");
                w.write(&type_to_rust(codegen, param.ty, types));
            }
            w.write(")");
            if let Some(ret) = method.ret_ty {
                w.write(" -> ");
                w.write(&type_to_rust(codegen, ret, types));
            }
            w.writeln(";");
        }
    });
    w.writeln("}");

    Ok(())
}

pub fn generate_type_alias(
    codegen: &RustCodegen<'_>,
    a: &HirTypeAlias,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub type ");
    w.write(codegen.resolve_symbol(a.name));
    w.write(" = ");
    w.write(&type_to_rust(codegen, a.ty, types));
    w.writeln(";");

    Ok(())
}

pub fn generate_const(
    codegen: &RustCodegen<'_>,
    c: &HirConst,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub const ");
    w.write(codegen.resolve_symbol(c.name));
    w.write(": ");
    if let Some(ty) = c.ty {
        w.write(&type_to_rust(codegen, ty, types));
    } else {
        w.write("()");
    }
    w.write(" = ");
    super::expr::generate_expr(codegen, &c.value, types, w, false, false, false)?;
    w.writeln(";");

    Ok(())
}

fn generate_block(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    wrap_tail_ok: bool,
) -> Result<(), CodegenError> {
    w.writeln("{");
    w.indented(|w| {
        for stmt in &block.stmts {
            let _ = super::stmt::generate_stmt(codegen, stmt, types, w, in_failable_fn, in_entry, false);
        }
        if let Some(expr) = &block.expr {
            if wrap_tail_ok && in_failable_fn && !in_entry {
                w.write("Ok(");
                let _ = super::expr::generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, false);
                w.writeln(")");
            } else {
                let _ = super::expr::generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, false);
            }
        }
    });
    w.write("}");
    Ok(())
}
