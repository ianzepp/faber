/**
 * Code Generation Entry Point - Multi-target source code emission
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module serves as the router for the code generation phase. It receives
 * a validated AST from the semantic analysis phase and dispatches to the
 * appropriate target-specific code generator.
 *
 * The design uses a simple switch statement rather than a plugin architecture
 * because we only support two targets and want to keep the codebase simple.
 * Each target generator is responsible for its own language semantics and
 * formatting conventions.
 *
 * This module also acts as the public API surface for the codegen package,
 * re-exporting types and target-specific generators for direct use.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node from parser (assumed valid after semantic analysis)
 *         CodegenOptions specifying target and formatting preferences
 * OUTPUT: String containing valid source code in the target language
 * ERRORS: Throws on unknown target (should be prevented by TypeScript types)
 *
 * INVARIANTS
 * ==========
 * INV-1: Program AST must be valid (semantic analysis must have passed)
 * INV-2: Generated code must be syntactically valid in target language
 * INV-3: Generated code preserves Latin source semantics in target language
 */

import type { Program } from "../parser/ast"
import type { CodegenOptions } from "./types"
import { generateTs } from "./ts"
import { generateZig } from "./zig"
import { generateWasm } from "./wasm"

// =============================================================================
// PUBLIC API
// =============================================================================

export type { CodegenOptions, CodegenTarget } from "./types"
export { generateTs } from "./ts"
export { generateZig } from "./zig"
export { generateWasm } from "./wasm"

// =============================================================================
// TARGET DISPATCHER
// =============================================================================

/**
 * Generate source code from AST for the specified target language.
 *
 * TRANSFORMS:
 *   Latin AST -> TypeScript (default)
 *   Latin AST -> Zig (when target: "zig")
 *
 * TARGET DIFFERENCES:
 *   TypeScript: Preserves JavaScript runtime semantics with type annotations
 *   Zig:        Transforms to compile-time evaluation and explicit types
 *
 * WHY: Defaults to TypeScript for web-first development workflow.
 *
 * @param program - Validated AST from parser/semantic analyzer
 * @param options - Target and formatting configuration
 * @returns Generated source code string
 * @throws Error if target is not 'ts' or 'zig' (should never happen with types)
 */
export function generate(program: Program, options: CodegenOptions = {}): string {
  // WHY: Default to TypeScript for web-first development
  const target = options.target ?? "ts"

  switch (target) {
    case "ts":
      return generateTs(program, options)
    case "zig":
      return generateZig(program, options)
    case "wasm":
      return generateWasm(program, options)
    default:
      // EDGE: TypeScript types prevent this, but defensive check for runtime
      throw new Error(`Unknown codegen target: ${target}`)
  }
}
