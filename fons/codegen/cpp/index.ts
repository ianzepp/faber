/**
 * C++23 Code Generator - Emit modern C++ source code
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into C++23 source code.
 * C++23 provides modern features that map well to Latin constructs:
 * - std::print for output (replaces iostream verbosity)
 * - std::expected<T,E> for error handling (aligns with Rust's Result)
 * - Concepts for interfaces (cleaner than abstract classes)
 * - Range-based for with views
 *
 * Key transformations:
 * - varia -> auto (mutable by default in C++)
 * - fixum -> const auto
 * - functio -> function definition
 * - genus -> struct (public by default, simpler than class)
 * - pactum -> concept (C++20)
 * - si/aliter -> if/else
 * - elige -> switch
 * - ex...pro -> range-based for
 * - scribe -> std::print
 * - ego -> this (pointer, use this-> for access)
 * - novum -> std::make_unique<T>(...) for heap allocation
 * - iace -> throw (using exceptions, not std::expected for simplicity)
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid C++23 source code string
 * ERRORS: Throws on unsupported AST node types
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid C++23
 * INV-2: All Latin type names are mapped to STL equivalents
 * INV-3: 4-space indentation (common C++ convention)
 */

import type { Program, Statement } from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { CppGenerator } from './generator';

/**
 * Generate C++23 source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> C++23 source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration
 * @returns C++23 source code
 */
export function generateCpp(program: Program, options: CodegenOptions = {}): string {
    const g = new CppGenerator(options.indent ?? '    ');

    // Separate declarations from runtime statements
    const declarations: Statement[] = [];
    const runtime: Statement[] = [];

    for (const stmt of program.body) {
        if (isDeclaration(stmt)) {
            declarations.push(stmt);
        } else {
            runtime.push(stmt);
        }
    }

    const lines: string[] = [];

    // Generate declarations first (functions, classes, etc.)
    for (const stmt of declarations) {
        lines.push(g.genStatement(stmt));
    }

    // Wrap runtime code in main()
    if (runtime.length > 0) {
        if (declarations.length > 0) {
            lines.push('');
        }

        lines.push('int main() {');
        g.depth++;

        for (const stmt of runtime) {
            lines.push(g.genStatement(stmt));
        }

        lines.push(`${g.ind()}return 0;`);
        g.depth--;
        lines.push('}');
    }

    // Build preamble (includes + helpers)
    const preamble = genPreamble(g);

    return preamble + lines.join('\n');
}

/**
 * Check if a statement is a declaration (vs runtime code).
 */
function isDeclaration(node: Statement): boolean {
    switch (node.type) {
        case 'FunctioDeclaration':
        case 'GenusDeclaration':
        case 'PactumDeclaration':
        case 'TypeAliasDeclaration':
        case 'OrdoDeclaration':
        case 'DiscretioDeclaration':
            return true;
        // Top-level const could be declaration, but we'll put in main for simplicity
        default:
            return false;
    }
}

/**
 * Generate preamble based on features used.
 *
 * WHY: C++ requires #include directives at the top of the file.
 *      Only emit includes for features actually used in the program.
 */
function genPreamble(g: CppGenerator): string {
    const parts: string[] = [];

    // Generate includes
    const includeLines = genIncludes(g);
    if (includeLines) {
        parts.push(includeLines);
    }

    // Add scope guard helper if demum was used
    if (g.needsScopeGuard) {
        parts.push('');
        parts.push(genScopeGuardHelper());
    }

    return parts.length > 0 ? parts.join('\n') + '\n' : '';
}

/**
 * Generate #include directives based on features used.
 */
function genIncludes(g: CppGenerator): string {
    // Always include these for basic functionality
    g.includes.add('<print>');
    g.includes.add('<string>');
    g.includes.add('<cstdint>');

    const sorted = Array.from(g.includes).sort();

    return sorted.map(h => `#include ${h}`).join('\n');
}

/**
 * Generate scope guard helper for demum (finally) blocks.
 *
 * WHY: C++ lacks finally. This RAII helper runs cleanup on scope exit.
 * The lambda captures by reference, so it can access local variables.
 */
function genScopeGuardHelper(): string {
    return `
template<typename F>
struct _ScopeGuard {
    F fn;
    ~_ScopeGuard() { fn(); }
};
`.trim();
}
