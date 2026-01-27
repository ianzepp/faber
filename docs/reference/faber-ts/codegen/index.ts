/**
 * Code Generation Entry Point - TypeScript source code emission
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * Faber is the TypeScript-only reference compiler. Multi-target codegen
 * (Python, Rust, Zig, C++) is handled by Rivus.
 *
 * See consilia/compiler-roles.md for the compiler separation rationale.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node from parser (assumed valid after semantic analysis)
 *         CodegenOptions specifying formatting preferences
 * OUTPUT: String containing valid TypeScript source code
 *
 * INVARIANTS
 * ==========
 * INV-1: Program AST must be valid (semantic analysis must have passed)
 * INV-2: Generated code must be syntactically valid TypeScript
 * INV-3: Generated code preserves Latin source semantics
 */

import type { Program } from '../parser/ast';
import type { CodegenOptions } from './types';
import { generateTs } from './ts/index';

// =============================================================================
// PUBLIC API
// =============================================================================

export type { CodegenOptions } from './types';
export { generateTs } from './ts/index';

// =============================================================================
// GENERATOR
// =============================================================================

/**
 * Generate TypeScript source code from AST.
 *
 * WHY: Faber targets TypeScript only. For other targets, use Rivus.
 *
 * @param program - Validated AST from parser/semantic analyzer
 * @param options - Formatting configuration
 * @returns Generated TypeScript source code string
 */
export function generate(program: Program, options: CodegenOptions = {}): string {
    return generateTs(program, options);
}
