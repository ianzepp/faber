/**
 * Zig Code Generator - Emit systems programming code
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into Zig source code. The
 * generator performs faithful translation — source structure maps directly
 * to output structure.
 *
 * Key principles:
 * - No magic: if source has no main(), output has no main()
 * - Statements emit in source order
 * - Source is responsible for valid Zig structure
 *
 * Key transformations:
 * - Latin types map to Zig types (numerus -> i64, etc.)
 * - Async becomes error union types (!T)
 * - Exceptions map to error returns
 * - scribe() maps to std.debug.print()
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Zig source code string (validity depends on source structure)
 * ERRORS: Throws on unknown AST node types (should never happen with valid AST)
 *
 * TARGET DIFFERENCES
 * ==================
 * Zig uses systems programming semantics:
 * - Compile-time vs runtime execution (comptime)
 * - Explicit types required for var declarations
 * - Error unions instead of exceptions (!T)
 * - No garbage collection (manual memory)
 * - Nullable via optional types (?T)
 * - No arrow functions (use struct { fn ... }.call pattern)
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code preserves source structure
 * INV-2: All Latin type names are mapped to Zig equivalents
 * INV-3: No implicit main() generation — source must define entry point
 */

import type { Program } from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { ZigGenerator } from './generator';
import { genPreamble } from './preamble';

/**
 * Generate Zig source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> Zig source code string
 *
 * WHY: Faithful translation — source structure determines output structure.
 * If source needs a main(), source must define functio main().
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration (indent only, semicolons always used)
 * @returns Zig source code
 */
export function generateZig(program: Program, options: CodegenOptions = {}): string {
    const g = new ZigGenerator(options.indent ?? '    ');

    // Generate all statements in source order
    const statements = program.body.map(stmt => g.genStatement(stmt));

    // Prepend preamble (imports based on features used)
    const preamble = genPreamble(g.features);

    return [preamble, ...statements].join('\n');
}
