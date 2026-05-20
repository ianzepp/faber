use super::CodeWriter;
use crate::hir::{DefId, HirLiteral, HirObjectField, HirObjectKey};
use crate::lexer::{Interner, Symbol};
use crate::semantic::TypeTable;
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
    pub(super) fn write_object_field(
        &self,
        field: &HirObjectField,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        match &field.key {
            HirObjectKey::Ident(name) => {
                w.write(&self.symbol_to_string(*name, interner));
                if let Some(value) = &field.value {
                    w.write(": ");
                    self.write_expr(value, types, names, interner, w);
                }
            }
            HirObjectKey::String(name) => {
                self.write_quoted_symbol(*name, interner, w);
                if let Some(value) = &field.value {
                    w.write(": ");
                    self.write_expr(value, types, names, interner, w);
                }
            }
            HirObjectKey::Computed(expr) => {
                w.write("[");
                self.write_expr(expr, types, names, interner, w);
                w.write("]");
                if let Some(value) = &field.value {
                    w.write(": ");
                    self.write_expr(value, types, names, interner, w);
                }
            }
            HirObjectKey::Spread(expr) => {
                w.write("sparge ");
                self.write_expr(expr, types, names, interner, w);
            }
        }
    }
    pub(super) fn write_literal(&self, lit: &HirLiteral, interner: &Interner, w: &mut CodeWriter) {
        match lit {
            HirLiteral::Int(value) => w.write(&value.to_string()),
            HirLiteral::Float(value) => w.write(&value.to_string()),
            HirLiteral::String(sym) => {
                self.write_quoted_symbol(*sym, interner, w);
            }
            HirLiteral::Regex(pattern, flags) => {
                w.write("sed ");
                self.write_quoted_symbol(*pattern, interner, w);
                if let Some(flags) = flags {
                    w.write(" ");
                    w.write(interner.resolve(*flags));
                }
            }
            HirLiteral::Bool(value) => w.write(if *value { "verum" } else { "falsum" }),
            HirLiteral::Nil => w.write("nihil"),
        }
    }

    pub(super) fn write_quoted_symbol(&self, sym: Symbol, interner: &Interner, w: &mut CodeWriter) {
        w.write("\"");
        w.write(interner.resolve(sym));
        w.write("\"");
    }

    pub(super) fn write_quoted_text(&self, text: &str, w: &mut CodeWriter) {
        w.write("\"");
        w.write(text);
        w.write("\"");
    }
    pub(super) fn write_symbol_literal(&self, symbol: Symbol, interner: &Interner, w: &mut CodeWriter) {
        w.write("\"");
        w.write(&self.symbol_to_string(symbol, interner));
        w.write("\"");
    }
}
