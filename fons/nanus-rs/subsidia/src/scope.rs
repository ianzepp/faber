use crate::{Locus, SemanticTypus};
use std::collections::HashMap;

/// Symbol species indicates what kind of symbol this is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolSpecies {
    Variabilis,
    Functio,
    Parametrum,
    Typus,
    Genus,
    Ordo,
    Discretio,
    Pactum,
    Varians,
}

impl std::fmt::Display for SymbolSpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolSpecies::Variabilis => write!(f, "variabilis"),
            SymbolSpecies::Functio => write!(f, "functio"),
            SymbolSpecies::Parametrum => write!(f, "parametrum"),
            SymbolSpecies::Typus => write!(f, "typus"),
            SymbolSpecies::Genus => write!(f, "genus"),
            SymbolSpecies::Ordo => write!(f, "ordo"),
            SymbolSpecies::Discretio => write!(f, "discretio"),
            SymbolSpecies::Pactum => write!(f, "pactum"),
            SymbolSpecies::Varians => write!(f, "varians"),
        }
    }
}

/// Scope species indicates what kind of scope this is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScopusSpecies {
    Global,
    Functio,
    Massa,
    Genus,
}

/// Symbol represents a named entity in the symbol table.
#[derive(Debug, Clone)]
pub struct Symbolum {
    pub nomen: String,
    pub typus: SemanticTypus,
    pub species: SymbolSpecies,
    pub mutabilis: bool,
    pub locus: Locus,
}

/// Scope represents a lexical scope with a symbol table.
#[derive(Debug)]
pub struct Scopus {
    pub symbola: HashMap<String, Symbolum>,
    pub species: ScopusSpecies,
    pub nomen: String,
}

impl Scopus {
    pub fn new(species: ScopusSpecies, nomen: impl Into<String>) -> Self {
        Self {
            symbola: HashMap::new(),
            species,
            nomen: nomen.into(),
        }
    }

    pub fn define(&mut self, sym: Symbolum) {
        self.symbola.insert(sym.nomen.clone(), sym);
    }

    pub fn lookup_local(&self, nomen: &str) -> Option<&Symbolum> {
        self.symbola.get(nomen)
    }
}

/// Semantic error during analysis.
#[derive(Debug, Clone)]
pub struct SemanticError {
    pub nuntius: String,
    pub locus: Locus,
}

/// SemanticContext holds the state during semantic analysis.
/// Uses a Vec stack instead of parent pointers for scope management.
#[derive(Debug)]
pub struct SemanticContext {
    scope_stack: Vec<Scopus>,
    pub typi: HashMap<String, SemanticTypus>,
    pub ordo_registry: HashMap<String, SemanticTypus>,
    pub disc_registry: HashMap<String, SemanticTypus>,
    pub genus_registry: HashMap<String, SemanticTypus>,
    pub errores: Vec<SemanticError>,
    expr_types: HashMap<usize, SemanticTypus>,
    next_expr_id: usize,
}

impl SemanticContext {
    pub fn new() -> Self {
        let global = Scopus::new(ScopusSpecies::Global, "");
        Self {
            scope_stack: vec![global],
            typi: HashMap::new(),
            ordo_registry: HashMap::new(),
            disc_registry: HashMap::new(),
            genus_registry: HashMap::new(),
            errores: Vec::new(),
            expr_types: HashMap::new(),
            next_expr_id: 0,
        }
    }

    pub fn enter_scope(&mut self, species: ScopusSpecies, nomen: impl Into<String>) {
        self.scope_stack.push(Scopus::new(species, nomen));
    }

    pub fn exit_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    pub fn current_scope(&self) -> &Scopus {
        self.scope_stack.last().unwrap()
    }

    pub fn current_scope_mut(&mut self) -> &mut Scopus {
        self.scope_stack.last_mut().unwrap()
    }

    pub fn define(&mut self, sym: Symbolum) {
        self.current_scope_mut().define(sym);
    }

    /// Look up a symbol in the scope chain (walks from current to global).
    pub fn lookup(&self, nomen: &str) -> Option<&Symbolum> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(sym) = scope.lookup_local(nomen) {
                return Some(sym);
            }
        }
        None
    }

    pub fn error(&mut self, nuntius: impl Into<String>, locus: Locus) {
        self.errores.push(SemanticError {
            nuntius: nuntius.into(),
            locus,
        });
    }

    pub fn register_typus(&mut self, nomen: impl Into<String>, typus: SemanticTypus) {
        self.typi.insert(nomen.into(), typus);
    }

    /// Resolve a type name to its semantic type.
    pub fn resolve_typus_nomen(&self, nomen: &str) -> SemanticTypus {
        use crate::types::*;

        match nomen {
            "textus" => textus(),
            "numerus" => numerus(),
            "fractus" => fractus(),
            "bivalens" => bivalens(),
            "nihil" => nihil(),
            "vacuum" | "vacuus" => vacuum(),
            "ignotum" | "quodlibet" | "quidlibet" => ignotum(),
            _ => {
                if let Some(t) = self.typi.get(nomen) {
                    return t.clone();
                }
                if let Some(t) = self.ordo_registry.get(nomen) {
                    return t.clone();
                }
                if let Some(t) = self.disc_registry.get(nomen) {
                    return t.clone();
                }
                if let Some(t) = self.genus_registry.get(nomen) {
                    return t.clone();
                }
                SemanticTypus::Usitatum {
                    nomen: nomen.to_string(),
                    nullabilis: false,
                }
            }
        }
    }

    pub fn next_id(&mut self) -> usize {
        let id = self.next_expr_id;
        self.next_expr_id += 1;
        id
    }

    pub fn set_expr_type(&mut self, id: usize, typus: SemanticTypus) {
        self.expr_types.insert(id, typus);
    }

    pub fn get_expr_type(&self, id: usize) -> SemanticTypus {
        self.expr_types
            .get(&id)
            .cloned()
            .unwrap_or_else(crate::types::ignotum)
    }
}

impl Default for SemanticContext {
    fn default() -> Self {
        Self::new()
    }
}
