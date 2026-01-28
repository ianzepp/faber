//! Pass 2: Name resolution
//!
//! Resolves all identifiers to their definitions.

use crate::semantic::{Resolver, SemanticError, TypeTable};
use crate::syntax::Program;

/// Resolve all names in the program
pub fn resolve(
    _program: &Program,
    _resolver: &mut Resolver,
    _types: &mut TypeTable,
) -> Result<(), Vec<SemanticError>> {
    // TODO: Implement name resolution
    //
    // 1. Walk the entire AST
    // 2. For each identifier:
    //    - Look up in scope chain
    //    - Error if not found
    //    - Record the DefId binding
    // 3. Enter/exit scopes for blocks, functions, etc.
    // 4. Define local variables when encountered
    // 5. Lower AST to HIR as we go

    Ok(())
}
