/**
 * Code Generation Types - Configuration and target specification
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module defines the configuration interface for the code generation phase.
 * It specifies which target language to emit (TypeScript or Zig) and formatting
 * preferences for the generated output.
 *
 * The codegen phase is target-agnostic at the AST level but produces radically
 * different output based on the target. TypeScript output preserves JavaScript
 * semantics with type annotations. Zig output transforms to systems programming
 * patterns with compile-time evaluation and explicit memory management.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Type parameters from codegen functions
 * OUTPUT: Type constraints for valid codegen options
 * ERRORS: TypeScript compile-time errors for invalid option combinations
 */

// =============================================================================
// TYPES
// =============================================================================

/**
 * WHY: Three targets are supported - TypeScript (default), Zig, and WASM
 *      Future targets (C, Rust) would extend this union
 */
export type CodegenTarget = 'ts' | 'zig' | 'wasm';

/**
 * Configuration options for code generation.
 *
 * DESIGN: Optional fields allow sensible defaults in each target generator.
 *         Target-specific options are documented with comments.
 */
export interface CodegenOptions {
    /**
   * Target language to generate.
   * WHY: Defaults to 'ts' in generate() function for web-first development.
   */
    target?: CodegenTarget

    /**
   * Indentation string for generated code.
   * WHY: TypeScript convention is 2 spaces, Zig convention is 4 spaces.
   *      Each target sets its own default.
   */
    indent?: string

    /**
   * Whether to emit semicolons at end of statements.
   * TARGET: TypeScript only - Zig always requires semicolons.
   *         This option is ignored for Zig target.
   */
    semicolons?: boolean
}
