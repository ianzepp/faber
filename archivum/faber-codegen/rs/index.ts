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
 * The generator performs faithful translation — source structure maps
 * directly to output structure.
 *
 * Key principles:
 * - No magic: if source has no incipit, output has no main()
 * - Statements emit in source order
 * - Source is responsible for valid Rust structure
 *
 * Key transformations:
 * - varia -> let mut
 * - fixum -> let (immutable by default)
 * - functio -> fn
 * - genus -> struct + impl
 * - pactum -> trait
 * - incipit -> fn main()
 * - si/secus -> if/else if/else
 * - elige -> match
 * - ex...pro -> for...in
 * - scribe -> println!/print!
 * - ego -> self
 * - novum -> StructName { } or StructName::new()
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Rust source code string (validity depends on source structure)
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
 * INV-1: Generated code preserves source structure
 * INV-2: Indentation is consistently 4 spaces per level (Rust convention)
 * INV-3: Uses idiomatic Rust patterns (ownership, Result, Option)
 * INV-4: No implicit main() generation — source must define entry point
 */

import type { Program } from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { RsGenerator } from './generator';
import { genPreamble } from './preamble';

/**
 * Generate Rust source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> Rust source code string
 *
 * WHY: Faithful translation — source structure determines output structure.
 * If source needs a main(), source must define incipit { }.
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration
 * @returns Rust source code
 */
export function generateRs(program: Program, options: CodegenOptions = {}): string {
    // WHY: 4 spaces is Rust community standard (rustfmt default)
    const g = new RsGenerator(options.indent ?? '    ');

    // Generate all statements in source order
    const statements = program.body.map(stmt => g.genStatement(stmt));

    // Generate preamble AFTER traversal so features are populated
    const lines: string[] = [];
    const preamble = genPreamble(g.features);
    if (preamble) {
        lines.push(preamble);
        lines.push('');
    }
    lines.push(...statements);

    return lines.join('\n');
}
