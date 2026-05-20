use super::CodeWriter;
use crate::hir::{DefId, HirCasuArm, HirExprKind, HirPattern};
use crate::lexer::{Interner, Symbol};
use crate::semantic::TypeTable;
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
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
