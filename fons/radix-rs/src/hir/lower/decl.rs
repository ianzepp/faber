//! Declaration lowering
//!
//! Lowers AST declarations to HIR items.

use super::{HirItem, HirItemKind, Lowerer};
use crate::syntax::{
    ClassDecl, EnumDecl, FuncDecl, ImportDecl, InterfaceDecl, Stmt, TypeAliasDecl, UnionDecl,
    VarDecl,
};

impl<'a> Lowerer<'a> {
    /// Lower varia/ficum declaration
    pub fn lower_varia(&mut self, stmt: &Stmt, decl: &VarDecl) -> Option<HirItem> {
        // TODO: Implement varia lowering
        None
    }

    /// Lower functio declaration
    pub fn lower_functio(&mut self, stmt: &Stmt, decl: &FuncDecl) -> Option<HirItem> {
        // TODO: Implement functio lowering
        None
    }

    /// Lower gens (class) declaration
    pub fn lower_gens(&mut self, stmt: &Stmt, decl: &ClassDecl) -> Option<HirItem> {
        // TODO: Implement gens lowering
        None
    }

    /// Lower ordo (enum) declaration
    pub fn lower_ordo(&mut self, stmt: &Stmt, decl: &EnumDecl) -> Option<HirItem> {
        // TODO: Implement ordo lowering
        None
    }

    /// Lower discretio (union) declaration
    pub fn lower_discretio(&mut self, stmt: &Stmt, decl: &UnionDecl) -> Option<HirItem> {
        // TODO: Implement discretio lowering
        None
    }

    /// Lower pactum (interface) declaration
    pub fn lower_pactum(&mut self, stmt: &Stmt, decl: &InterfaceDecl) -> Option<HirItem> {
        // TODO: Implement pactum lowering
        None
    }

    /// Lower typus (type alias) declaration
    pub fn lower_typus(&mut self, stmt: &Stmt, decl: &TypeAliasDecl) -> Option<HirItem> {
        // TODO: Implement typus lowering
        None
    }

    /// Lower importa (import) declaration
    pub fn lower_importa(&mut self, stmt: &Stmt, decl: &ImportDecl) -> Option<HirItem> {
        // TODO: Implement importa lowering
        None
    }
}
