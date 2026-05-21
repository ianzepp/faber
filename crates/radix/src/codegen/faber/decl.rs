use super::{CodeWriter, CodegenError};
use crate::hir::{DefId, HirEnum, HirFunction, HirInterface, HirItem, HirItemKind, HirStruct, HirTestModifier};
use crate::lexer::{Interner, Symbol};
use crate::semantic::TypeTable;
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
    pub(super) fn is_synthetic_proba_function(
        &self,
        func: &HirFunction,
        _types: &TypeTable,
        _interner: &Interner,
    ) -> bool {
        func.test.is_some()
    }
    pub(super) fn generate_item(
        &self,
        item: &HirItem,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        match &item.kind {
            HirItemKind::Function(func) => {
                self.generate_function(func, types, names, interner, w)?;
            }
            HirItemKind::Struct(s) => {
                self.generate_struct(s, types, names, interner, w)?;
            }
            HirItemKind::Enum(e) => {
                self.generate_enum(e, types, names, interner, w)?;
            }
            HirItemKind::Interface(i) => {
                self.generate_interface(i, types, names, interner, w)?;
            }
            HirItemKind::TypeAlias(a) => {
                w.write("typus ");
                w.write(&self.symbol_to_string(a.name, interner));
                w.write(" = ");
                w.write(&self.type_to_faber(a.ty, types, names, interner));
                w.newline();
            }
            HirItemKind::Const(c) => {
                w.write("fixum ");
                if let Some(ty) = c.ty {
                    w.write(&self.type_to_faber(ty, types, names, interner));
                    w.write(" ");
                }
                w.write(&self.symbol_to_string(c.name, interner));
                w.write(" ← ");
                self.write_expr(&c.value, types, names, interner, w);
                w.newline();
            }
            HirItemKind::Import(import) => {
                w.write("importa ex ");
                self.write_symbol_literal(import.path, interner, w);
                w.write(" ");
                w.write(match import.visibility {
                    crate::syntax::Visibility::Private => "privata",
                    crate::syntax::Visibility::Public => "publica",
                });
                if let Some(item) = import.items.first() {
                    w.write(" ");
                    let name = self.symbol_to_string(item.name, interner);
                    if item.alias == Some(item.name) {
                        w.write("* ut ");
                        w.write(&name);
                    } else if let Some(alias) = item.alias {
                        let alias = self.symbol_to_string(alias, interner);
                        w.write(&name);
                        w.write(" ut ");
                        w.write(&alias);
                    } else {
                        w.write(&name);
                    }
                }
                w.newline();
            }
        }
        Ok(())
    }

    pub(super) fn generate_function(
        &self,
        func: &HirFunction,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        if self.is_synthetic_proba_function(func, types, interner) {
            self.generate_proba_function(func, types, names, interner, w);
            return Ok(());
        }

        if func.is_async {
            w.writeln("@ futura");
        }
        if func.is_generator {
            w.writeln("@ cursor");
        }

        w.write("functio ");
        w.write(&self.symbol_to_string(func.name, interner));

        if !func.type_params.is_empty() {
            w.write("(");
            for (i, param) in func.type_params.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                w.write("prae typus ");
                w.write(&self.symbol_to_string(param.name, interner));
            }
            w.write(")");
        } else {
            w.write("(");
        }

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 || !func.type_params.is_empty() {
                w.write(", ");
            }
            if param.optional {
                w.write("si ");
            }
            match param.mode {
                crate::hir::HirParamMode::Ref => w.write("de "),
                crate::hir::HirParamMode::MutRef => w.write("in "),
                crate::hir::HirParamMode::Move => w.write("ex "),
                crate::hir::HirParamMode::Owned => {}
            }
            w.write(&self.type_to_faber(param.ty, types, names, interner));
            w.write(" ");
            w.write(&self.symbol_to_string(param.name, interner));
        }
        w.write(")");

        if let Some(ret) = func.ret_ty {
            w.write(" → ");
            w.write(&self.type_to_faber(ret, types, names, interner));
        }

        if let Some(body) = &func.body {
            w.writeln(" {");
            w.indented(|w| self.write_block(body, types, names, interner, w));
            w.writeln("}");
        } else {
            w.newline();
        }

        Ok(())
    }

    pub(super) fn generate_proba_function(
        &self,
        func: &HirFunction,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        w.write("proba ");
        if let Some(test) = &func.test {
            for modifier in &test.modifiers {
                self.write_test_modifier(modifier, interner, w);
                w.write(" ");
            }
            self.write_quoted_text(interner.resolve(test.name), w);
        } else {
            self.write_quoted_text(interner.resolve(func.name), w);
        }

        if let Some(body) = &func.body {
            w.writeln(" {");
            w.indented(|w| self.write_block(body, types, names, interner, w));
            w.writeln("}");
        } else {
            w.newline();
        }
    }

    pub(super) fn generate_struct(
        &self,
        s: &HirStruct,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("genus ");
        w.write(&self.symbol_to_string(s.name, interner));

        if !s.type_params.is_empty() {
            w.write("<");
            for (i, param) in s.type_params.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                w.write(&self.symbol_to_string(param.name, interner));
            }
            w.write(">");
        }

        if let Some(parent) = s.extends {
            w.write(" sub ");
            w.write(&self.name_for_def(parent, names, interner));
        }

        if !s.implements.is_empty() {
            w.write(" implet ");
            for (i, interface) in s.implements.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                w.write(&self.name_for_def(*interface, names, interner));
            }
        }

        w.writeln(" {");
        let mut struct_result = Ok(());
        w.indented(|w| {
            for field in &s.fields {
                if field.is_static {
                    w.write("generis ");
                }
                w.write(&self.type_to_faber(field.ty, types, names, interner));
                w.write(" ");
                w.write(&self.symbol_to_string(field.name, interner));
                if let Some(init) = &field.init {
                    w.write(": ");
                    self.write_expr(init, types, names, interner, w);
                }
                w.newline();
            }

            for method in &s.methods {
                if struct_result.is_err() {
                    return;
                }
                w.newline();
                struct_result = self.generate_function(&method.func, types, names, interner, w);
            }
        });
        struct_result?;
        w.writeln("}");

        Ok(())
    }

    fn write_test_modifier(&self, modifier: &HirTestModifier, interner: &Interner, w: &mut CodeWriter) {
        match modifier {
            HirTestModifier::Omitte(reason) => {
                w.write("omitte ");
                self.write_quoted_text(interner.resolve(*reason), w);
            }
            HirTestModifier::Futurum(reason) => {
                w.write("futurum ");
                self.write_quoted_text(interner.resolve(*reason), w);
            }
            HirTestModifier::Solum => {
                w.write("solum");
            }
            HirTestModifier::Tag(tag) => {
                w.write("tag ");
                self.write_quoted_text(interner.resolve(*tag), w);
            }
            HirTestModifier::Temporis(n) => {
                w.write("temporis ");
                w.write(&n.to_string());
            }
            HirTestModifier::Metior => {
                w.write("metior");
            }
            HirTestModifier::Repete(n) => {
                w.write("repete ");
                w.write(&n.to_string());
            }
            HirTestModifier::Fragilis(n) => {
                w.write("fragilis ");
                w.write(&n.to_string());
            }
            HirTestModifier::Requirit(req) => {
                w.write("requirit ");
                self.write_quoted_text(interner.resolve(*req), w);
            }
            HirTestModifier::SolumIn(env) => {
                w.write("solum_in ");
                self.write_quoted_text(interner.resolve(*env), w);
            }
        }
    }

    pub(super) fn generate_enum(
        &self,
        e: &HirEnum,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("discretio ");
        w.write(&self.symbol_to_string(e.name, interner));

        if !e.type_params.is_empty() {
            w.write("<");
            for (i, param) in e.type_params.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                w.write(&self.symbol_to_string(param.name, interner));
            }
            w.write(">");
        }

        w.writeln(" {");
        w.indented(|w| {
            for variant in &e.variants {
                w.write(&self.symbol_to_string(variant.name, interner));
                if !variant.fields.is_empty() {
                    w.writeln(" {");
                    w.indented(|w| {
                        for field in &variant.fields {
                            w.write(&self.type_to_faber(field.ty, types, names, interner));
                            w.write(" ");
                            w.write(&self.symbol_to_string(field.name, interner));
                            w.newline();
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

    pub(super) fn generate_interface(
        &self,
        i: &HirInterface,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("pactum ");
        w.write(&self.symbol_to_string(i.name, interner));

        if !i.type_params.is_empty() {
            w.write("<");
            for (idx, param) in i.type_params.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(&self.symbol_to_string(param.name, interner));
            }
            w.write(">");
        }

        w.writeln(" {");
        w.indented(|w| {
            for method in &i.methods {
                w.write("functio ");
                w.write(&self.symbol_to_string(method.name, interner));
                w.write("(");
                for (idx, param) in method.params.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    w.write(&self.type_to_faber(param.ty, types, names, interner));
                    w.write(" ");
                    w.write(&self.symbol_to_string(param.name, interner));
                }
                w.write(")");
                if let Some(ret) = method.ret_ty {
                    w.write(" → ");
                    w.write(&self.type_to_faber(ret, types, names, interner));
                }
                w.newline();
            }
        });
        w.writeln("}");

        Ok(())
    }
}
