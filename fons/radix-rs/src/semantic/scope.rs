//! Scope and symbol table

use super::types::TypeId;
use crate::hir::DefId;
use crate::lexer::{Span, Symbol as LexSymbol};
use rustc_hash::FxHashMap;

/// Scope ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub u32);

/// Scope kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    Module,
    Function,
    Block,
    Loop,
    Match,
}

/// A scope containing symbols
#[derive(Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub kind: ScopeKind,
    pub symbols: FxHashMap<LexSymbol, DefId>,
}

impl Scope {
    pub fn new(id: ScopeId, parent: Option<ScopeId>, kind: ScopeKind) -> Self {
        Self {
            id,
            parent,
            kind,
            symbols: FxHashMap::default(),
        }
    }
}

/// Symbol information
#[derive(Debug)]
pub struct Symbol {
    pub def_id: DefId,
    pub name: LexSymbol,
    pub kind: SymbolKind,
    pub ty: Option<TypeId>,
    pub mutable: bool,
    pub span: Span,
}

/// Kind of symbol
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

/// Name resolver - manages scopes and symbol definitions
pub struct Resolver {
    scopes: Vec<Scope>,
    symbols: FxHashMap<DefId, Symbol>,
    current_scope: ScopeId,
    next_def_id: u32,
}

impl Resolver {
    pub fn new() -> Self {
        let global = Scope::new(ScopeId(0), None, ScopeKind::Global);
        Self {
            scopes: vec![global],
            symbols: FxHashMap::default(),
            current_scope: ScopeId(0),
            next_def_id: 0,
        }
    }

    /// Create a new DefId
    pub fn fresh_def_id(&mut self) -> DefId {
        let id = DefId(self.next_def_id);
        self.next_def_id += 1;
        id
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        let scope = Scope::new(id, Some(self.current_scope), kind);
        self.scopes.push(scope);
        self.current_scope = id;
        id
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope.0 as usize].parent {
            self.current_scope = parent;
        }
    }

    /// Define a symbol in the current scope
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

    /// Look up a symbol by name
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

    /// Get symbol information by DefId
    pub fn get_symbol(&self, def_id: DefId) -> Option<&Symbol> {
        self.symbols.get(&def_id)
    }

    /// Get mutable symbol information
    pub fn get_symbol_mut(&mut self, def_id: DefId) -> Option<&mut Symbol> {
        self.symbols.get_mut(&def_id)
    }

    /// Get current scope
    pub fn current(&self) -> &Scope {
        &self.scopes[self.current_scope.0 as usize]
    }

    /// Check if we're in a loop
    pub fn in_loop(&self) -> bool {
        let mut scope_id = Some(self.current_scope);

        while let Some(id) = scope_id {
            let scope = &self.scopes[id.0 as usize];
            if scope.kind == ScopeKind::Loop {
                return true;
            }
            if scope.kind == ScopeKind::Function {
                return false; // Don't look past function boundary
            }
            scope_id = scope.parent;
        }

        false
    }

    /// Check if we're in a function
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
