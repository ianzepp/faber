//! Rust declaration generation

use super::super::CodeWriter;
use super::types::type_to_rust;
use super::CodegenError;
use crate::hir::*;
use crate::semantic::TypeTable;

pub fn generate_function(
    func: &HirFunction,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Async modifier
    if func.is_async {
        w.write("async ");
    }

    w.write("fn ");
    // TODO: Write function name from symbol
    w.write("todo_func_name");

    // Type parameters
    if !func.type_params.is_empty() {
        w.write("<");
        for (i, param) in func.type_params.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            // TODO: Write type param name
            w.write("T");
        }
        w.write(">");
    }

    // Parameters
    w.write("(");
    for (i, param) in func.params.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        // TODO: Write param name and type
        w.write("param: ()");
    }
    w.write(")");

    // Return type
    if let Some(ret_ty) = func.ret_ty {
        w.write(" -> ");
        w.write(&type_to_rust(ret_ty, types));
    }

    // Body
    if let Some(body) = &func.body {
        w.write(" ");
        generate_block(body, types, w)?;
    } else {
        w.write(";");
    }

    w.newline();
    Ok(())
}

pub fn generate_struct(
    s: &HirStruct,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub struct ");
    // TODO: Write struct name
    w.write("TodoStruct");

    if !s.type_params.is_empty() {
        w.write("<");
        for (i, _param) in s.type_params.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            w.write("T");
        }
        w.write(">");
    }

    w.writeln(" {");
    w.indented(|w| {
        for field in &s.fields {
            if !field.is_static {
                w.write("pub ");
                // TODO: Write field name and type
                w.writeln("field: (),");
            }
        }
    });
    w.writeln("}");

    // Generate impl block for methods
    if !s.methods.is_empty() {
        w.newline();
        w.write("impl ");
        w.write("TodoStruct");
        w.writeln(" {");
        w.indented(|w| {
            for method in &s.methods {
                let _ = generate_function(&method.func, types, w);
            }
        });
        w.writeln("}");
    }

    Ok(())
}

pub fn generate_enum(
    e: &HirEnum,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub enum ");
    // TODO: Write enum name
    w.write("TodoEnum");

    if !e.type_params.is_empty() {
        w.write("<");
        for (i, _param) in e.type_params.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            w.write("T");
        }
        w.write(">");
    }

    w.writeln(" {");
    w.indented(|w| {
        for variant in &e.variants {
            // TODO: Write variant name
            w.write("Variant");
            if !variant.fields.is_empty() {
                w.writeln(" {");
                w.indented(|w| {
                    for field in &variant.fields {
                        // TODO: Write field
                        w.writeln("field: (),");
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
    i: &HirInterface,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub trait ");
    // TODO: Write trait name
    w.write("TodoTrait");

    if !i.type_params.is_empty() {
        w.write("<");
        for (idx, _param) in i.type_params.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            w.write("T");
        }
        w.write(">");
    }

    w.writeln(" {");
    w.indented(|w| {
        for method in &i.methods {
            w.write("fn ");
            // TODO: Write method name
            w.write("method");
            w.write("(");
            w.write("&self");
            for param in &method.params {
                w.write(", ");
                // TODO: Write param
                w.write("param: ()");
            }
            w.write(")");
            if let Some(ret) = method.ret_ty {
                w.write(" -> ");
                w.write(&type_to_rust(ret, types));
            }
            w.writeln(";");
        }
    });
    w.writeln("}");

    Ok(())
}

pub fn generate_type_alias(
    a: &HirTypeAlias,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub type ");
    // TODO: Write alias name
    w.write("TodoAlias");
    w.write(" = ");
    w.write(&type_to_rust(a.ty, types));
    w.writeln(";");

    Ok(())
}

pub fn generate_const(
    c: &HirConst,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub const ");
    // TODO: Write const name
    w.write("TODO_CONST");
    w.write(": ");
    if let Some(ty) = c.ty {
        w.write(&type_to_rust(ty, types));
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
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.writeln("{");
    w.indented(|w| {
        for stmt in &block.stmts {
            // TODO: Generate statement
            w.writeln("todo!();");
        }
        if let Some(expr) = &block.expr {
            // TODO: Generate tail expression
            w.writeln("todo!()");
        }
    });
    w.write("}");
    Ok(())
}
