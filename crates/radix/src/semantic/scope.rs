//! Lexical scope tree and definition identity registry.
//!
//! Name binding in Radix has two related responsibilities: find the closest
//! visible declaration for a source name, and preserve stable [`DefId`] metadata
//! once that declaration has been found. This module keeps those concerns
//! separate. Scopes store name-to-definition bindings for lexical lookup, while
//! the resolver's symbol table stores the metadata later passes need after the
//! original scope may no longer be current.
//!
//! PASS ROLE
//! =========
//! Collection defines top-level symbols into the global scope. Resolution then
//! enters and exits lexical scopes while binding local names, parameters, and
//! references. HIR lowering and later HIR analyses consume the resulting
//! definition identities instead of re-resolving source names.
//!
//! CONTROL-FLOW CONTEXT
//! ====================
//! Scope kinds are semantic context markers as well as lookup containers.
//! `in_loop` deliberately stops at a function boundary so `break` and
//! `continue` inside nested functions do not inherit an enclosing loop.
//!
//! INVARIANTS
//! ==========
//! - `ScopeId` indexes `Resolver::scopes`; the global scope is always `0`.
//! - Each scope parent points to an earlier scope, forming a tree rooted at
//!   global.
//! - Every `DefId` stored in a scope's symbol map has corresponding metadata in
//!   `Resolver::symbols`.
//! - `current_scope` always points at a valid scope and never exits past global.

use super::types::TypeId;
use crate::hir::DefId;
use crate::lexer::{Span, Symbol as LexSymbol};
use rustc_hash::FxHashMap;

/// Stable index into the resolver's scope arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub u32);

/// Semantic role for a lexical scope.
///
/// The role is used both for name-binding shape and for context-sensitive
/// validation such as loop-only control flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// Root scope for declarations visible at compilation-unit level.
    Global,

    /// Reserved module boundary for package/import-aware resolution.
    Module,

    /// Function body boundary; loop checks do not search past it.
    Function,

    /// Ordinary lexical block for local bindings.
    Block,

    /// Loop body where `break` and `continue` are valid.
    Loop,

    /// Pattern-match arm scope for bindings introduced by a case.
    Match,
}

/// One lexical lookup frame.
///
/// `symbols` maps source-level interned names to definition identities. The
/// heavier symbol metadata lives in [`Resolver`] so callers can retain a
/// `DefId` after leaving the scope where it was declared.
#[derive(Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub kind: ScopeKind,
    pub symbols: FxHashMap<LexSymbol, DefId>,
}

impl Scope {
    pub fn new(id: ScopeId, parent: Option<ScopeId>, kind: ScopeKind) -> Self {
        Self { id, parent, kind, symbols: FxHashMap::default() }
    }
}

/// Metadata attached to one definition identity.
///
/// Symbols are the resolver's long-lived contract with HIR lowering and later
/// semantic passes: once a source name has been resolved to a `DefId`, this
/// record carries the declaration kind, optional type, mutability, and source
/// span needed for diagnostics.
#[derive(Debug)]
pub struct Symbol {
    pub def_id: DefId,
    pub name: LexSymbol,
    pub kind: SymbolKind,
    pub ty: Option<TypeId>,
    pub mutable: bool,
    pub span: Span,
}

/// Declaration category for a resolved definition.
///
/// These categories are semantic rather than syntactic; for example, methods
/// and free functions share callable behavior later, but keeping distinct
/// symbol kinds preserves enough provenance for member lookup and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Local,
    Param,
    Function,
    Method,
    Struct,
    Enum,
    Variant,
    Field,
    Interface,
    TypeAlias,
    TypeParam,
    Module,
}

/// Mutable name-resolution state shared by collection, resolution, and HIR lowering.
///
/// The resolver behaves like a stack while walking syntax, but stores scopes in
/// an arena so `ScopeId` and `DefId` values remain stable after traversal moves
/// elsewhere.
pub struct Resolver {
    scopes: Vec<Scope>,
    symbols: FxHashMap<DefId, Symbol>,
    current_scope: ScopeId,
    next_def_id: u32,
}

impl Resolver {
    /// Build a resolver with an empty global scope.
    pub fn new() -> Self {
        let global = Scope::new(ScopeId(0), None, ScopeKind::Global);
        Self { scopes: vec![global], symbols: FxHashMap::default(), current_scope: ScopeId(0), next_def_id: 0 }
    }

    /// Allocate a fresh definition identity.
    ///
    /// Allocation is separate from [`Self::define`] so passes can prepare symbol
    /// records before inserting them into the current lexical scope.
    pub fn fresh_def_id(&mut self) -> DefId {
        let id = DefId(self.next_def_id);
        self.next_def_id += 1;
        id
    }

    /// Enter a child scope and make it current until [`Self::exit_scope`].
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        let scope = Scope::new(id, Some(self.current_scope), kind);
        self.scopes.push(scope);
        self.current_scope = id;
        id
    }

    /// Exit the current scope, staying at global if there is no parent.
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope.0 as usize].parent {
            self.current_scope = parent;
        }
    }

    /// Define a symbol in the current scope.
    ///
    /// Duplicate names are rejected only within the current lexical frame;
    /// shadowing outer scopes remains representable and is handled by pass
    /// policy when it wants to warn.
    pub fn define(&mut self, symbol: Symbol) -> Result<DefId, LexSymbol> {
        let scope = &mut self.scopes[self.current_scope.0 as usize];

        if scope.symbols.contains_key(&symbol.name) {
            return Err(symbol.name);
        }

        let def_id = symbol.def_id;
        scope.symbols.insert(symbol.name, def_id);
        self.symbols.insert(def_id, symbol);
        Ok(def_id)
    }

    /// Look up the nearest visible definition for a source name.
    pub fn lookup(&self, name: LexSymbol) -> Option<DefId> {
        let mut scope_id = Some(self.current_scope);

        while let Some(id) = scope_id {
            let scope = &self.scopes[id.0 as usize];
            if let Some(&def_id) = scope.symbols.get(&name) {
                return Some(def_id);
            }
            scope_id = scope.parent;
        }

        None
    }

    /// Get symbol metadata by definition identity.
    pub fn get_symbol(&self, def_id: DefId) -> Option<&Symbol> {
        self.symbols.get(&def_id)
    }

    /// Get mutable symbol metadata by definition identity.
    pub fn get_symbol_mut(&mut self, def_id: DefId) -> Option<&mut Symbol> {
        self.symbols.get_mut(&def_id)
    }

    /// Return the current lexical scope.
    pub fn current(&self) -> &Scope {
        &self.scopes[self.current_scope.0 as usize]
    }

    /// Return whether the current context is inside a loop in this function.
    pub fn in_loop(&self) -> bool {
        let mut scope_id = Some(self.current_scope);

        while let Some(id) = scope_id {
            let scope = &self.scopes[id.0 as usize];
            if scope.kind == ScopeKind::Loop {
                return true;
            }
            if scope.kind == ScopeKind::Function {
                return false;
            }
            scope_id = scope.parent;
        }

        false
    }

    /// Return whether the current context is inside a function scope.
    pub fn in_function(&self) -> bool {
        let mut scope_id = Some(self.current_scope);

        while let Some(id) = scope_id {
            let scope = &self.scopes[id.0 as usize];
            if scope.kind == ScopeKind::Function {
                return true;
            }
            scope_id = scope.parent;
        }

        false
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
