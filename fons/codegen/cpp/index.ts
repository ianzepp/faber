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
 * The generator performs faithful translation — source structure maps
 * directly to output structure.
 *
 * Key principles:
 * - No magic: if source has no incipit, output has no main()
 * - Statements emit in source order
 * - Source is responsible for valid C++ structure
 *
 * Key transformations:
 * - varia -> auto (mutable by default in C++)
 * - fixum -> const auto
 * - functio -> function definition
 * - genus -> struct (public by default, simpler than class)
 * - pactum -> concept (C++20)
 * - incipit -> int main() { }
 * - si/secus -> if/else
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
 * OUTPUT: C++23 source code string (validity depends on source structure)
 * ERRORS: Throws on unsupported AST node types
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code preserves source structure
 * INV-2: All Latin type names are mapped to STL equivalents
 * INV-3: 4-space indentation (common C++ convention)
 * INV-4: No implicit main() generation — source must define entry point
 */

import type { Program } from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { CppGenerator } from './generator';
import { genPreamble } from './preamble';

/**
 * Generate C++23 source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> C++23 source code string
 *
 * WHY: Faithful translation — source structure determines output structure.
 * If source needs a main(), source must define incipit { }.
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration
 * @returns C++23 source code
 */
export function generateCpp(program: Program, options: CodegenOptions = {}): string {
    const g = new CppGenerator(options.indent ?? '    ');

    // Generate all statements in source order
    const statements = program.body.map(stmt => g.genStatement(stmt));

    // Build preamble (includes + helpers)
    const preamble = genPreamble(g.includes, g.needsScopeGuard);

    return preamble + statements.join('\n');
}
