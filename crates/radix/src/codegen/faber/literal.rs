//! Literal and object-field spelling for the canonical Faber backend.
//!
//! Literal emission is shared by expressions, patterns, object literals, and
//! conversion entries. Keeping these rules in one module prevents subtle drift
//! between `{"key" = value}`, regex literals, pattern literals, and generated
//! object fields.
//!
//! INVARIANTS
//! ==========
//! - Ident, string, computed, and spread object fields use the parser's current
//!   Faber spellings.
//! - Regex values print as `sed "pattern"` with optional flags when HIR carries
//!   them.
//! - Boolean and nil literals print as `verum`, `falsum`, and `nihil`.
//!
//! LIMITS
//! ======
//! Quoted text is written directly from interned strings. This module does not
//! currently perform escape reconstruction or guarantee that every arbitrary
//! string payload can be re-emitted as a valid quoted literal.

use super::CodeWriter;
use crate::hir::{DefId, HirLiteral, HirObjectField, HirObjectKey};
use crate::lexer::{Interner, Symbol};
use crate::semantic::TypeTable;
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
    /// Write one object-style field in canonical Faber syntax.
    ///
    /// The same field form is used for object literals and conversion entries.
    /// Identifier shorthand is preserved when HIR has no explicit value;
    /// string, computed, and spread fields always print their explicit source
    /// shape.
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
                    w.write(" = ");
                    self.write_expr(value, types, names, interner, w);
                }
            }
            HirObjectKey::String(name) => {
                self.write_quoted_symbol(*name, interner, w);
                if let Some(value) = &field.value {
                    w.write(" = ");
                    self.write_expr(value, types, names, interner, w);
                }
            }
            HirObjectKey::Computed(expr) => {
                w.write("[");
                self.write_expr(expr, types, names, interner, w);
                w.write("]");
                if let Some(value) = &field.value {
                    w.write(" = ");
                    self.write_expr(value, types, names, interner, w);
                }
            }
            HirObjectKey::Spread(expr) => {
                w.write("sparge ");
                self.write_expr(expr, types, names, interner, w);
            }
        }
    }

    /// Write a lowered literal using source-level Faber keywords and sigils.
    ///
    /// Numeric formatting follows Rust's display output for the stored numeric
    /// value. String and regex payloads share the quoted-symbol helper, so this
    /// function does not claim original delimiter or escape preservation.
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

    /// Quote interned source text without adding or normalizing escapes.
    pub(super) fn write_quoted_symbol(&self, sym: Symbol, interner: &Interner, w: &mut CodeWriter) {
        w.write("\"");
        w.write(interner.resolve(sym));
        w.write("\"");
    }

    /// Quote caller-provided text without adding or normalizing escapes.
    pub(super) fn write_quoted_text(&self, text: &str, w: &mut CodeWriter) {
        w.write("\"");
        w.write(text);
        w.write("\"");
    }

    /// Quote a symbol after resolving it through the backend name policy.
    pub(super) fn write_symbol_literal(&self, symbol: Symbol, interner: &Interner, w: &mut CodeWriter) {
        w.write("\"");
        w.write(&self.symbol_to_string(symbol, interner));
        w.write("\"");
    }
}
