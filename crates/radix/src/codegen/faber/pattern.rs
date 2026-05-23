//! Pattern and match-arm emission for canonical Faber output.
//!
//! `discerne` expressions delegate arm and pattern spelling here so match
//! output stays aligned across variants, bindings, aliases, guards, and literal
//! patterns. The writer preserves the semantic pattern tree in grammar-valid
//! Faber rather than attempting to recover the user's original punctuation or
//! whitespace.
//!
//! INVARIANTS
//! ==========
//! - Match arms print as `casu ... { ... }` with optional `si` guards.
//! - Binding names prefer resolved definitions so renamed or shadowed symbols
//!   follow the same name policy as expressions.
//! - Aliases use the canonical `pattern ut name` spelling.
//! - Variant patterns print the variant name followed by `fixum` bindings when
//!   the HIR pattern carries payload patterns.
//! - Literal patterns share literal spelling with expression literals.

use super::CodeWriter;
use crate::hir::{DefId, HirCasuArm, HirExprKind, HirPattern};
use crate::lexer::{Interner, Symbol};
use crate::semantic::TypeTable;
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
    /// Write `discerne` arms from lowered pattern data.
    ///
    /// Arm bodies may already be lowered blocks or single expressions. This
    /// function preserves that distinction only as far as the grammar requires:
    /// every arm is braced, and non-block bodies are emitted as one expression
    /// statement inside the arm.
    pub(super) fn write_match_arms(
        &self,
        arms: &[HirCasuArm],
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        for arm in arms {
            w.write("casu ");
            for (idx, pattern) in arm.patterns.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                self.write_pattern(pattern, names, interner, w);
            }
            if let Some(guard) = &arm.guard {
                w.write(" si ");
                self.write_expr(guard, types, names, interner, w);
            }
            w.writeln(" {");
            w.indented(|w| match &arm.body.kind {
                HirExprKind::Block(block) => self.write_block(block, types, names, interner, w),
                _ => {
                    self.write_expr(&arm.body, types, names, interner, w);
                    w.newline();
                }
            });
            w.writeln("}");
        }
    }

    /// Write one pattern in canonical Faber syntax.
    ///
    /// Pattern output is intentionally name-policy aware but not token
    /// preserving. It uses resolved names for bindings and variants where
    /// available, then delegates literal payloads to the shared literal writer.
    pub(super) fn write_pattern(
        &self,
        pattern: &HirPattern,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        match pattern {
            HirPattern::Wildcard => w.write("_"),
            HirPattern::Binding(def_id, name) => {
                let name = names.get(def_id).copied().unwrap_or(*name);
                w.write(&self.symbol_to_string(name, interner));
            }
            HirPattern::Alias(def_id, name, pattern) => {
                self.write_pattern(pattern, names, interner, w);
                let name = names.get(def_id).copied().unwrap_or(*name);
                w.write(" ut ");
                w.write(&self.symbol_to_string(name, interner));
            }
            HirPattern::Variant(def_id, patterns) => {
                w.write(&self.name_for_def(*def_id, names, interner));
                if !patterns.is_empty() {
                    w.write(" fixum ");
                    for (idx, pat) in patterns.iter().enumerate() {
                        if idx > 0 {
                            w.write(", ");
                        }
                        self.write_pattern(pat, names, interner, w);
                    }
                }
            }
            HirPattern::Literal(lit) => {
                self.write_literal(lit, interner, w);
            }
        }
    }
}
