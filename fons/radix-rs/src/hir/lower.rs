//! AST to HIR lowering

use crate::syntax::Program;
use crate::semantic::Resolver;
use super::HirProgram;

/// Lower AST to HIR
///
/// This is performed during name resolution, so the resolver
/// is passed in to provide DefId mappings.
pub fn lower(_program: &Program, _resolver: &Resolver) -> HirProgram {
    // TODO: Implement lowering
    HirProgram {
        items: Vec::new(),
        entry: None,
    }
}
