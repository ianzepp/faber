//! Type lowering
//!
//! Lowers AST type expressions to TypeIds.

use super::Lowerer;
use crate::semantic::TypeId;
use crate::syntax::TypeExpr;

impl<'a> Lowerer<'a> {
    /// Lower a type expression to TypeId
    pub fn lower_type(&mut self, ty: &TypeExpr) -> TypeId {
        // TODO: Look up type in resolver/types table
        // For now, return a placeholder
        TypeId(0)
    }
}
