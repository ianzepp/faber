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
 * - Top-level code is wrapped in fn main()
 * - Functions, structs, traits remain at module scope
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
 * INV-4: Main function is only emitted if there are runtime statements
 */

import type { Program, Statement, Expression } from '../../parser/ast';
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

    // WHY: Rust distinguishes module-level (fn, struct, trait) from runtime code
    const topLevel: Statement[] = [];
    const runtime: Statement[] = [];

    for (const stmt of program.body) {
        if (isTopLevelDeclaration(stmt)) {
            topLevel.push(stmt);
        } else {
            runtime.push(stmt);
        }
    }

    const lines: string[] = [];

    // Emit top-level declarations (functions, structs, traits, etc.)
    lines.push(...topLevel.map(stmt => g.genStatement(stmt)));

    // WHY: Only emit main() if there's runtime code to execute
    if (runtime.length > 0) {
        if (topLevel.length > 0) {
            lines.push('');
        }

        lines.push('fn main() {');
        g.depth++;
        lines.push(...runtime.map(stmt => g.genStatement(stmt)));
        g.depth--;
        lines.push('}');
    }

    return lines.join('\n');
}

/**
 * Determine if a statement belongs at module scope.
 *
 * WHY: Rust requires functions, structs, traits, enums, and type aliases
 *      to be at module scope. Only executable statements go in main().
 */
function isTopLevelDeclaration(node: Statement): boolean {
    switch (node.type) {
        // Functions are always module-level
        case 'FunctioDeclaration':
            return true;

        // Imports are module-level
        case 'ImportaDeclaration':
            return true;

        // Structs and traits are module-level
        case 'GenusDeclaration':
        case 'PactumDeclaration':
            return true;

        // Type aliases and enums are module-level
        case 'TypeAliasDeclaration':
        case 'OrdoDeclaration':
        case 'DiscretioDeclaration':
            return true;

        // Test blocks are module-level (Rust #[test] functions)
        case 'ProbandumStatement':
            return true;

        // WHY: Variable declarations go in main() - Rust const requires
        //      explicit types and SCREAMING_CASE which we don't have
        case 'VariaDeclaration':
            return false;

        default:
            return false;
    }
}

/**
 * Determine if an expression can be a const value at module scope.
 *
 * WHY: Rust allows const declarations at module scope for literals
 *      and const expressions.
 */
function isConstValue(node: Expression): boolean {
    if (node.type === 'Literal') {
        return true;
    }

    if (node.type === 'TemplateLiteral') {
        return true;
    }

    // WHY: verum, falsum, nihil are Latin keywords for literal values
    if (node.type === 'Identifier') {
        return ['verum', 'falsum', 'nihil'].includes(node.name);
    }

    // WHY: Binary/unary expressions with const operands are also const
    if (node.type === 'BinaryExpression') {
        return isConstValue(node.left) && isConstValue(node.right);
    }

    if (node.type === 'UnaryExpression') {
        return isConstValue(node.argument);
    }

    return false;
}
