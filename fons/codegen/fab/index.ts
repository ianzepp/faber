/**
 * Faber Code Generator - Emit canonical Faber source code from AST
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST back into Faber source code.
 * Primary use cases:
 * - TS-to-Faber migration tools
 * - Refactoring tools that transform AST and re-emit
 * - Programmatic code generation
 *
 * The generator produces canonical Faber style:
 * - si/sin/secus for conditionals; casu/ceterum for elige/discerne
 * - &&/|| (not et/aut), non (not !)
 * - -> for return type (not fit/fiet/fiunt/fient)
 * - pro x: expr for lambdas (not pro x redde expr)
 * - 4 space indentation
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node from parser
 * OUTPUT: Valid Faber source code string that can round-trip through the parser
 * ERRORS: Throws on unknown AST node types
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid Faber
 * INV-2: Generated code can be parsed back to an equivalent AST
 * INV-3: Output follows canonical style conventions
 */

import type { Program } from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { FabGenerator } from './generator';

/**
 * Generate Faber source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> Faber source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration (indent)
 * @returns Faber source code
 */
export function generateFab(program: Program, options: CodegenOptions = {}): string {
    // WHY: 4 spaces is Faber convention
    const indent = options.indent ?? '    ';

    const g = new FabGenerator(indent);

    // Generate body - no preamble needed for Faber output
    const body = program.body.map(stmt => g.genStatement(stmt)).join('\n');

    return body;
}
