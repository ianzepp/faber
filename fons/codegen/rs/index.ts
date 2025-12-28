/**
 * Rust Code Generator - Emit Rust source code
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into Rust source code.
 * Rust is a systems programming language focused on safety, concurrency,
 * and performance through ownership and borrowing.
 *
 * Key transformations:
 * - varia -> let mut
 * - fixum -> let (immutable by default)
 * - functio -> fn
 * - genus -> struct + impl
 * - pactum -> trait
 * - si/aliter -> if/else if/else
 * - elige -> match
 * - ex...pro -> for...in
 * - scribe -> println!/print!
 * - ego -> self
 * - novum -> StructName { } or StructName::new()
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid Rust source code string
 * ERRORS: Throws on unsupported AST node types
 *
 * TARGET DIFFERENCES
 * ==================
 * Rust characteristics:
 * - Ownership and borrowing (& for references, &mut for mutable refs)
 * - No null - uses Option<T> and Result<T, E>
 * - Pattern matching is exhaustive
 * - Traits for polymorphism (no inheritance)
 * - Macros for metaprogramming (println!, vec!, etc.)
 * - Lifetimes for reference validity
 * - No exceptions - Result<T, E> and ? operator
 * - Iterators are lazy and composable
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid Rust
 * INV-2: Indentation is consistently 4 spaces per level (Rust convention)
 * INV-3: Uses idiomatic Rust patterns (ownership, Result, Option)
 */

import type { Program } from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { RsGenerator } from './generator';

/**
 * Generate Rust source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> Rust source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration
 * @returns Rust source code
 */
export function generateRs(program: Program, options: CodegenOptions = {}): string {
    // WHY: 4 spaces is Rust community standard (rustfmt default)
    const g = new RsGenerator(options.indent ?? '    ');

    // Generate body
    const body = program.body.map(stmt => g.genStatement(stmt)).join('\n');

    return body;
}
