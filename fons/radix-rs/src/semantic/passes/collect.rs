//! Pass 1: Collect declarations
//!
//! Walks the AST and registers all top-level declarations,
//! creating DefIds for each named item. Does not look inside
//! function bodies.

use crate::syntax::Program;
use crate::semantic::{Resolver, TypeTable, SemanticError};

/// Collect all declarations from the program
pub fn collect(
    _program: &Program,
    _resolver: &mut Resolver,
    _types: &mut TypeTable,
) -> Result<(), Vec<SemanticError>> {
    // TODO: Implement declaration collection
    //
    // 1. Walk all top-level statements
    // 2. For each declaration (function, class, enum, etc.):
    //    - Create a DefId
    //    - Register in the global scope
    //    - Store basic info (name, kind)
    // 3. Do NOT descend into function bodies
    // 4. Handle imports (resolve module paths)

    Ok(())
}
