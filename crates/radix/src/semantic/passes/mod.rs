//! Semantic pass registry and execution-order map.
//!
//! This module is the index for the compiler phases that run after parsing and
//! before backend code generation. The first two passes, [`collect`] and
//! [`resolve`], operate directly on the parsed AST: they establish the symbol
//! universe, lexical scope tree, and type-alias entries needed before HIR
//! lowering can attach stable semantic identities. The later passes consume HIR
//! and progressively add type information, target-sensitive analysis, and
//! non-fatal diagnostics.
//!
//! PASS ORDER
//! ==========
//! 1. [`collect`] registers top-level AST declarations and imported module
//!    bindings in the global resolver scope.
//! 2. [`resolve`] walks AST bodies, checks lexical name/type availability, and
//!    lowers type aliases into the shared type table.
//! 3. HIR lowering happens outside this module, after collect/resolve errors
//!    have been reported.
//! 4. [`typecheck`] assigns and verifies expression/statement types on HIR.
//! 5. [`borrow`] runs target-gated ownership checks where required, currently
//!    for Rust output.
//! 6. [`exhaustive`] reports pattern coverage issues.
//! 7. [`lint`] reports quality diagnostics that may be warnings rather than
//!    hard semantic errors.
//!
//! ERROR POLICY
//! ============
//! Collect and resolve return accumulated hard errors; callers stop before HIR
//! lowering when those errors exist because unresolved names would make later
//! phase identities unreliable. HIR-phase passes can contribute warnings as
//! well as errors, and the driver decides success from the final diagnostic
//! severities rather than from pass execution alone.

pub mod borrow;
pub mod collect;
pub mod exhaustive;
pub mod lint;
pub mod resolve;
pub mod typecheck;
