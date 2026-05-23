use crate::hir::visit::HirVisitor;
use crate::hir::{DefId, HirProgram};
use crate::lexer::{Interner, Symbol};
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
    pub(super) fn name_for_def(&self, def_id: DefId, names: &FxHashMap<DefId, Symbol>, interner: &Interner) -> String {
        names
            .get(&def_id)
            .map(|sym| self.symbol_to_string(*sym, interner))
            .unwrap_or_else(|| format!("def_{}", def_id.0))
    }

    pub(super) fn symbol_to_string(&self, sym: Symbol, interner: &Interner) -> String {
        interner.resolve(sym).to_owned()
    }

    /// Collect DefId -> Symbol mappings for all definitions in the program.
    ///
    /// WHY: HIR uses DefIds for references; we need to map them back to their
    /// original names for source generation. This is a single upfront traversal
    /// rather than repeated lookups during generation.
    pub(super) fn collect_names(&self, hir: &HirProgram) -> FxHashMap<DefId, Symbol> {
        let mut collector = FaberNameCollector { names: FxHashMap::default() };
        collector.visit_program(hir);
        collector.names
    }
}

struct FaberNameCollector {
    names: FxHashMap<DefId, Symbol>,
}

impl HirVisitor for FaberNameCollector {
    fn visit_def(&mut self, def_id: DefId, name: Symbol) {
        self.names.insert(def_id, name);
    }
}
