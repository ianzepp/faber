//! Faber canonical code generation
//!
//! Pretty-prints Faber source in canonical form.
//! Useful for formatting, normalization, and round-tripping.

use super::{Codegen, CodegenError, CodeWriter};
use crate::hir::{HirProgram, HirItem, HirItemKind, HirFunction, HirStruct, HirEnum};
use crate::semantic::{Type, TypeTable, Primitive, TypeId, Mutability};
use crate::FaberOutput;

/// Faber canonical code generator
pub struct FaberCodegen;

impl FaberCodegen {
    pub fn new() -> Self {
        Self
    }

    fn generate_item(
        &self,
        item: &HirItem,
        types: &TypeTable,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        match &item.kind {
            HirItemKind::Function(func) => {
                self.generate_function(func, types, w)?;
            }
            HirItemKind::Struct(s) => {
                self.generate_struct(s, types, w)?;
            }
            HirItemKind::Enum(e) => {
                self.generate_enum(e, types, w)?;
            }
            HirItemKind::Interface(i) => {
                self.generate_interface(i, types, w)?;
            }
            HirItemKind::TypeAlias(a) => {
                w.write("typus ");
                // TODO: Write alias name
                w.write("TodoAlias");
                w.write(" = ");
                w.write(&self.type_to_faber(a.ty, types));
                w.newline();
            }
            HirItemKind::Const(c) => {
                w.write("fixum ");
                if let Some(ty) = c.ty {
                    w.write(&self.type_to_faber(ty, types));
                    w.write(" ");
                }
                // TODO: Write const name
                w.write("TODO_CONST");
                w.write(": ");
                // TODO: Generate expression
                w.writeln("nihil");
            }
            HirItemKind::Import(_) => {
                // TODO: Generate import
            }
        }
        Ok(())
    }

    fn generate_function(
        &self,
        func: &HirFunction,
        types: &TypeTable,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("functio ");
        // TODO: Write function name
        w.write("todo_functio");

        // Type parameters
        if !func.type_params.is_empty() {
            w.write("(");
            for (i, _param) in func.type_params.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                w.write("prae typus T");
            }
            w.write(")");
        } else {
            w.write("(");
        }

        // Parameters
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 || !func.type_params.is_empty() {
                w.write(", ");
            }
            // Mode
            match param.mode {
                crate::hir::HirParamMode::Ref => w.write("de "),
                crate::hir::HirParamMode::MutRef => w.write("in "),
                crate::hir::HirParamMode::Move => w.write("ex "),
                crate::hir::HirParamMode::Owned => {}
            }
            w.write(&self.type_to_faber(param.ty, types));
            w.write(" ");
            // TODO: Write param name
            w.write("param");
        }
        w.write(")");

        // Return type
        if let Some(ret) = func.ret_ty {
            w.write(" -> ");
            w.write(&self.type_to_faber(ret, types));
        }

        // Body
        if let Some(_body) = &func.body {
            w.writeln(" {");
            w.indented(|w| {
                // TODO: Generate body
                w.writeln("nihil");
            });
            w.writeln("}");
        } else {
            w.newline();
        }

        Ok(())
    }

    fn generate_struct(
        &self,
        s: &HirStruct,
        types: &TypeTable,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("genus ");
        // TODO: Write struct name
        w.write("TodoGenus");

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

        if s.extends.is_some() {
            w.write(" sub ");
            // TODO: Write parent name
            w.write("Parent");
        }

        if !s.implements.is_empty() {
            w.write(" implet ");
            for (i, _impl) in s.implements.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                // TODO: Write interface name
                w.write("Pactum");
            }
        }

        w.writeln(" {");
        w.indented(|w| {
            // Fields
            for field in &s.fields {
                if field.is_static {
                    w.write("generis ");
                }
                w.write(&self.type_to_faber(field.ty, types));
                w.write(" ");
                // TODO: Write field name
                w.writeln("ager");
            }

            // Methods
            for method in &s.methods {
                w.newline();
                let _ = self.generate_function(&method.func, types, w);
            }
        });
        w.writeln("}");

        Ok(())
    }

    fn generate_enum(
        &self,
        e: &HirEnum,
        types: &TypeTable,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("discretio ");
        // TODO: Write enum name
        w.write("TodoDiscretio");

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
                w.write("Varietas");
                if !variant.fields.is_empty() {
                    w.writeln(" {");
                    w.indented(|w| {
                        for field in &variant.fields {
                            w.write(&self.type_to_faber(field.ty, types));
                            w.write(" ");
                            // TODO: Write field name
                            w.writeln("ager");
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

    fn generate_interface(
        &self,
        i: &crate::hir::HirInterface,
        types: &TypeTable,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("pactum ");
        // TODO: Write interface name
        w.write("TodoPactum");

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
                w.write("functio ");
                // TODO: Write method name
                w.write("methodus");
                w.write("(");
                for (idx, param) in method.params.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    w.write(&self.type_to_faber(param.ty, types));
                    w.write(" param");
                }
                w.write(")");
                if let Some(ret) = method.ret_ty {
                    w.write(" -> ");
                    w.write(&self.type_to_faber(ret, types));
                }
                w.newline();
            }
        });
        w.writeln("}");

        Ok(())
    }

    fn type_to_faber(&self, type_id: TypeId, types: &TypeTable) -> String {
        let ty = types.get(type_id);

        match ty {
            Type::Primitive(prim) => match prim {
                Primitive::Textus => "textus",
                Primitive::Numerus => "numerus",
                Primitive::Fractus => "fractus",
                Primitive::Bivalens => "bivalens",
                Primitive::Nihil => "nihil",
                Primitive::Vacuum => "vacuum",
                Primitive::Numquam => "numquam",
                Primitive::Ignotum => "ignotum",
                Primitive::Octeti => "octeti",
            }.to_owned(),

            Type::Array(elem) => {
                format!("lista<{}>", self.type_to_faber(*elem, types))
            }

            Type::Map(key, value) => {
                format!(
                    "tabula<{}, {}>",
                    self.type_to_faber(*key, types),
                    self.type_to_faber(*value, types)
                )
            }

            Type::Set(elem) => {
                format!("copia<{}>", self.type_to_faber(*elem, types))
            }

            Type::Option(inner) => {
                format!("si {}", self.type_to_faber(*inner, types))
            }

            Type::Ref(mutability, inner) => {
                let prefix = match mutability {
                    Mutability::Immutable => "de",
                    Mutability::Mutable => "in",
                };
                format!("{} {}", prefix, self.type_to_faber(*inner, types))
            }

            Type::Struct(_) | Type::Enum(_) | Type::Interface(_) => {
                // TODO: Look up name
                "TodoTypus".to_owned()
            }

            Type::Alias(_, resolved) => {
                self.type_to_faber(*resolved, types)
            }

            Type::Func(sig) => {
                let params: Vec<String> = sig.params.iter()
                    .map(|p| self.type_to_faber(p.ty, types))
                    .collect();
                let ret = self.type_to_faber(sig.ret, types);
                format!("({}) -> {}", params.join(", "), ret)
            }

            Type::Param(_) => "T".to_owned(),

            Type::Applied(base, args) => {
                let base_str = self.type_to_faber(*base, types);
                let args_str: Vec<String> = args.iter()
                    .map(|a| self.type_to_faber(*a, types))
                    .collect();
                format!("{}<{}>", base_str, args_str.join(", "))
            }

            Type::Infer(_) => "ignotum".to_owned(),
            Type::Union(_) => "unio".to_owned(),
            Type::Error => "/* error */".to_owned(),
        }
    }
}

impl Default for FaberCodegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen for FaberCodegen {
    type Output = FaberOutput;

    fn generate(
        &self,
        hir: &HirProgram,
        types: &TypeTable,
    ) -> Result<FaberOutput, CodegenError> {
        let mut w = CodeWriter::new();

        for item in &hir.items {
            self.generate_item(item, types, &mut w)?;
            w.newline();
        }

        Ok(FaberOutput {
            code: w.finish(),
        })
    }
}
