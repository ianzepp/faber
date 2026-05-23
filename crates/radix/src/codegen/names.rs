//! Shared HIR name recovery for target backends.
//!
//! HIR nodes refer to declarations through [`DefId`]s, while generated source
//! needs stable textual names. This module builds a small catalog by walking the
//! HIR once and recording every definition symbol the visitor exposes. Backends
//! use it to recover source spellings without each target reimplementing HIR
//! traversal or reaching back into resolver internals.
//!
//! INVARIANTS
//! ==========
//! - The catalog is best-effort recovery from HIR, not a name-resolution pass.
//! - Missing entries produce a deterministic placeholder so codegen failures are
//!   reproducible and obvious instead of panicking in a backend.
//! - Symbols remain interned; text is resolved lazily through the shared
//!   [`Interner`] owned by the compile session.

use crate::hir::visit::HirVisitor;
use crate::hir::{DefId, HirProgram};
use crate::lexer::{Interner, Symbol};
use rustc_hash::FxHashMap;

/// DefId-to-symbol lookup shared by target emitters.
///
/// This type exists because HIR is intentionally identifier-light after
/// semantic resolution: references carry compiler IDs, and only declarations
/// retain the source symbol needed for target names. The catalog keeps that
/// recovery local to codegen while preserving the invariant that the interner
/// owns all string storage.
pub(crate) struct NameCatalog<'a> {
    names: FxHashMap<crate::hir::DefId, Symbol>,
    interner: &'a Interner,
}

impl<'a> NameCatalog<'a> {
    /// Build a catalog from the definition-bearing nodes in one HIR program.
    pub(crate) fn new(hir: &HirProgram, interner: &'a Interner) -> Self {
        Self { names: collect_names(hir), interner }
    }

    /// Resolve an already-known symbol through the compile-session interner.
    pub(crate) fn resolve_symbol(&self, sym: Symbol) -> &str {
        self.interner.resolve(sym)
    }

    /// Resolve a definition ID to its source spelling.
    ///
    /// Missing names usually mean a backend is trying to emit an incomplete or
    /// synthetic HIR surface. Returning a fixed sentinel keeps output stable
    /// enough for diagnostics and tests to identify the upstream gap.
    pub(crate) fn resolve_def(&self, def_id: crate::hir::DefId) -> &str {
        self.names
            .get(&def_id)
            .map(|sym| self.resolve_symbol(*sym))
            .unwrap_or("unresolved_def")
    }

    /// Iterate over collected definitions for backend-specific precomputation.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&crate::hir::DefId, &Symbol)> {
        self.names.iter()
    }
}

/// Collect every definition symbol exposed by the HIR visitor.
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
