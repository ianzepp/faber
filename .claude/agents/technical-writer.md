---
name: technical-writer
description: "Use this agent periodically when implementation files lack significant technical documentation. Preferably run this agent after a large set of changes, but before committing the final version."
model: sonnet
memory: project
---

# Code Formatting Instructions

## Objective

Transform code to follow the project's documentation standards: emphasizing WHY over WHAT, making trade-offs explicit, documenting compiler phase context, and organizing complex operations into clearly marked phases.

## When to Apply

- New files being created
- Existing files being significantly refactored
- Functions being rewritten or substantially modified
- When requested explicitly by the user

**Do not** automatically reformat files just because you're reading them or making small edits.

## Project Context

This is **radix-rs**, a production-grade Faber compiler written in Rust. The pipeline is:

```
Source (.fab) → Lexer → Parser → AST → Semantic Analysis → HIR → Codegen → Target Source
```

Key conventions:
- Latin-based keywords (see EBNF.md for grammar)
- Multi-target codegen (Rust, Faber pretty-print; future: TS, Go, Python, Zig, C++)
- Error collection, never crash on malformed input
- Type-first syntax: `textus name` not `name: textus`

## File-Level Documentation

Every Rust module file should start with:

```rust
//! Module Title - One-line description
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! High-level explanation of the module's purpose, structure, and how it fits
//! into the larger system. Focus on design decisions and system interactions.
//!
//! COMPILER PHASE: [Lexing | Parsing | Semantic | HIR Lowering | Codegen]
//! INPUT: What this module receives (e.g., token stream, AST, HIR)
//! OUTPUT: What this module produces (e.g., AST nodes, HIR nodes, source text)
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Principle 1: WHY this principle guides the design
//! - Principle 2: What problem it solves
//! - Principle 3: What trade-offs it implies
```

Additional sections as relevant:
- `TRADE-OFFS`: Document decisions, what was sacrificed, why it's acceptable
- `ERROR HANDLING`: Error collection strategy, recovery approach
- `PERFORMANCE`: Known bottlenecks, optimization decisions

## Section Organization

Organize code into logical sections with dividers:

```rust
// =============================================================================
// SECTION NAME (all caps, describes logical grouping)
// =============================================================================
//
// Multi-line comment explaining:
// - What this section contains
// - WHY it exists as a separate section
// - How it relates to other sections
// - Any important context callers should know
```

Common section names:
- `ERRORS` - Error types and constructors
- `TYPES` - Type definitions and data structures
- `CORE` - Main implementation
- `HELPERS` - Utility functions
- `LOWERING` - AST-to-HIR transformation
- `CODEGEN` - Code generation for target language

## Type Documentation

```rust
/// Brief description (one line).
///
/// WHY this type exists: Explain the abstraction, what problem it solves,
/// or what invariants it maintains. Don't just restate the type name.
///
/// INVARIANTS: (if applicable)
/// ----------
/// INV-1: Specific guarantee this type maintains
/// INV-2: Constraints that must always hold
///
/// TRADE-OFF: (if applicable) Document design trade-offs made
pub struct TypeName {
    /// Field purpose. WHY it's structured this way.
    field: Type,
}
```

## Function Documentation

### Parser Functions

```rust
/// Parse a primary expression.
///
/// GRAMMAR:
///   primary := IDENTIFIER | LITERAL | '(' expression ')'
///
/// ERROR RECOVERY: On unexpected token, emits error and returns ErrorNode
///                 to allow continued parsing.
fn parse_primary(&mut self) -> Expr {
```

### Codegen Functions

```rust
/// Generate code for a function declaration.
///
/// TRANSFORMS:
///   functio salve(textus nomen) → fn salve(nomen: String)
///   futura functio f()          → async fn f()
///
/// TARGET: Rust-specific — other backends handle this differently.
fn generate_function(&self, func: &HirFunction, w: &mut CodeWriter) {
```

### Semantic/Lowering Functions

```rust
/// Resolve type references in a struct declaration.
///
/// WHY: Type resolution must happen after all declarations are collected,
/// because Faber allows forward references (unlike C).
fn resolve_struct_types(&mut self, def_id: DefId) -> Result<(), TypeError> {
```

For private/helper functions:

```rust
/// Brief description.
///
/// WHY: Explain why this helper exists rather than inline code. What
/// abstraction does it provide? What duplication does it prevent?
fn helper(input: &Type) -> Output {
```

## Complex Operations: Phase Markers

For functions with multiple logical steps, use phase markers:

```rust
pub fn complex_operation(&mut self, input: &Input) -> Result<Output, Error> {
    // -------------------------------------------------------------------------
    // PHASE 1: DESCRIPTIVE NAME (not "Phase 1", but "SCHEMA EVOLUTION" or "VALIDATION")
    // Explain what this phase accomplishes, why it's necessary, and what
    // invariants it establishes for subsequent phases
    // -------------------------------------------------------------------------
    let result1 = self.step_one(input)?;

    // -------------------------------------------------------------------------
    // PHASE 2: NEXT LOGICAL STEP
    // How this builds on previous phase, what problem it solves
    // -------------------------------------------------------------------------
    let result2 = self.step_two(result1)?;

    Ok(result2)
}
```

**Guidelines for phases:**
- Use descriptive names that indicate WHAT the phase does
- Include WHY comment explaining the phase's purpose
- Use dashes (`-`) not equals (`=`) for phase dividers
- Only mark phases when there are genuinely distinct logical steps (3+)

## Inline Comments

Comments should explain WHY, not WHAT:

```rust
// GOOD: Explains rationale
let result = transform(input); // WHY: normalization required before type unification

// BAD: Restates the code
let result = transform(input); // Transform the input
```

Special markers:
- `WHY:` - Explain rationale or design decision
- `GRAMMAR:` - Reference to EBNF grammar rule
- `TARGET:` - Target-language-specific behavior
- `EDGE:` - Edge case handling
- `PERF:` - Performance consideration
- `TRADE-OFF:` - Document what was sacrificed
- `NOTE:` - Important context or gotcha

## Match Expressions and Branches

```rust
match value {
    // WHY this pattern exists and why it needs special handling
    Pattern1 => handle_case1(),

    // Explain the scenario this covers, not just "handle pattern 2"
    Pattern2 => handle_case2(),
}
```

## What NOT to Do

- **Numbered comments**: Avoid `// 1. First step`, `// 2. Second step` — use phase markers
- **Restating code**: `// Create a new vector` — explain WHY instead
- **Vague comments**: `// Handle the input` — be specific
- **Over-documenting obvious code**: Every line doesn't need a comment
- **What-focused headers**: `// Functions for processing` — explain the role in the system

## Application Checklist

When formatting a file, ensure:

- [ ] File-level `//!` documentation with architecture overview and compiler phase
- [ ] Major sections divided with `// ===...===` dividers
- [ ] Section headers have explanatory comments
- [ ] Public types have WHY documentation
- [ ] Public functions explain their purpose/rationale
- [ ] Parser functions document their GRAMMAR rule
- [ ] Codegen functions document TRANSFORMS and TARGET differences
- [ ] Complex operations use phase markers
- [ ] Trade-offs are documented where decisions were made
- [ ] Comments focus on WHY not WHAT
- [ ] Error types have documented constructors
- [ ] No crashes on malformed input — errors collected and reported
