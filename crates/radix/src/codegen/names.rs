use crate::hir::visit::HirVisitor;
use crate::hir::{DefId, HirProgram};
use crate::lexer::{Interner, Symbol};
use rustc_hash::FxHashMap;

pub(crate) struct NameCatalog<'a> {
    names: FxHashMap<crate::hir::DefId, Symbol>,
    interner: &'a Interner,
}

impl<'a> NameCatalog<'a> {
    pub(crate) fn new(hir: &HirProgram, interner: &'a Interner) -> Self {
        Self { names: collect_names(hir), interner }
    }

    pub(crate) fn resolve_symbol(&self, sym: Symbol) -> &str {
        self.interner.resolve(sym)
    }

    pub(crate) fn resolve_def(&self, def_id: crate::hir::DefId) -> &str {
        self.names
            .get(&def_id)
            .map(|sym| self.resolve_symbol(*sym))
            .unwrap_or("unresolved_def")
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&crate::hir::DefId, &Symbol)> {
        self.names.iter()
    }
}

pub(crate) fn collect_names(hir: &HirProgram) -> FxHashMap<crate::hir::DefId, Symbol> {
    let mut collector = NameCollector { names: FxHashMap::default() };
    collector.visit_program(hir);
    collector.names
}

struct NameCollector {
    names: FxHashMap<DefId, Symbol>,
}

impl HirVisitor for NameCollector {
    fn visit_def(&mut self, def_id: DefId, name: Symbol) {
        self.names.insert(def_id, name);
    }
}
