//! Semantic analysis passes
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Organizes the seven semantic analysis passes into a module hierarchy.
//! Each pass builds on the results of previous passes, with early exits on
//! errors to prevent cascading failures.
//!
//! PASS ORDER
//! ==========
//! 1. collect - Register top-level declarations
//! 2. resolve - Resolve all name references
//! 3. (lowering happens in hir module)
//! 4. typecheck - Bidirectional type inference
//! 5. borrow - Ownership and borrowing validation (Rust target)
//! 6. exhaustive - Pattern match coverage checking
//! 7. lint - Code quality warnings
//!
//! WHY: Modules are organized by pass rather than by language construct,
//! reflecting the multi-pass architecture of the compiler.

pub mod borrow;
pub mod collect;
pub mod exhaustive;
pub mod lint;
pub mod resolve;
pub mod typecheck;
