//! Rust declaration generation

use super::super::CodeWriter;
use super::types::type_to_rust;
use super::{CodegenError, RustCodegen};
use crate::hir::*;
use crate::semantic::TypeTable;

pub fn generate_function(
    codegen: &RustCodegen<'_>,
    func: &HirFunction,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
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
    if let Some(ret_ty) = func.ret_ty {
        w.write(" -> ");
        w.write(&type_to_rust(codegen, ret_ty, types));
    }

    // Body
    if let Some(body) = &func.body {
        w.write(" ");
        generate_block(codegen, body, types, w)?;
    } else {
        w.write(";");
    }

    w.newline();
    Ok(())
}

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
                let _ = generate_function(codegen, &method.func, types, w);
            }
        });
        w.writeln("}");
    }

    Ok(())
}

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
    // TODO: Generate const value
    w.write("todo!()");
    w.writeln(";");

    Ok(())
}

fn generate_block(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.writeln("{");
    w.indented(|w| {
        for stmt in &block.stmts {
            // TODO: Generate statement
            let _ = super::stmt::generate_stmt(codegen, stmt, types, w);
        }
        if let Some(expr) = &block.expr {
            // TODO: Generate tail expression
            let _ = super::expr::generate_expr(codegen, expr, types, w);
        }
    });
    w.write("}");
    Ok(())
}
