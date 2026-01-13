/**
 * Python Code Generator - Emit Python 3.10+ source code
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into Python source code.
 * Python uses significant whitespace (indentation) rather than braces,
 * which requires careful depth tracking during code generation.
 *
 * Key transformations:
 * - varia/fixum -> assignment (Python has no const)
 * - functio -> def
 * - futura functio -> async def
 * - genus -> class with dataclass-like pattern
 * - pactum -> typing.Protocol
 * - si/secus -> if/elif/else
 * - elige -> match/case (Python 3.10+)
 * - ex...pro -> for...in
 * - scribe -> print()
 * - ego -> self
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid Python 3.10+ source code string
 * ERRORS: Throws on unsupported AST node types
 *
 * TARGET DIFFERENCES
 * ==================
 * Python characteristics:
 * - Indentation-based blocks (no braces)
 * - Dynamic typing (type hints are optional annotations)
 * - No `new` keyword (classes called directly)
 * - `self` explicit in method signatures
 * - Pattern matching via match/case (3.10+)
 * - Union types via X | Y syntax (3.10+)
 * - Async via asyncio module
 * - No true constants (convention: UPPER_CASE)
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid Python 3.10+
 * INV-2: All Latin type names are mapped to Python type hints
 * INV-3: Indentation is consistently 4 spaces per level
 * INV-4: Generated code follows PEP 8 style guidelines where practical
 */

import type { Program } from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { PyGenerator } from './generator';
import { genPreamble } from './preamble';

/**
 * Generate Python source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> Python 3.10+ source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration
 * @returns Python source code
 */
export function generatePy(program: Program, options: CodegenOptions = {}): string {
    const g = new PyGenerator(options.indent ?? '    ');

    // First pass: generate body (this populates features)
    const body = program.body.map(stmt => g.genStatement(stmt)).join('\n');

    // Second: prepend preamble based on detected features
    const preamble = genPreamble(g.features);

    return preamble + body;
}
