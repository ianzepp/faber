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
 * WHY: Multiple targets are supported for different use cases:
 *      - ts: TypeScript (default) - web-first development
 *      - zig: Systems programming, educational
 *      - py: Python - popular, good for teaching
 *      - rs: Rust - memory safety without garbage collection
 *      - cpp: C++ - systems programming alternative
 */
export type CodegenTarget = 'ts' | 'zig' | 'py' | 'rs' | 'cpp';

/**
 * Features used in the source code that require preamble setup.
 *
 * WHY: Different targets need different setup code (imports, includes, class
 *      definitions) based on which language features are actually used.
 *      Tracking usage allows minimal, tree-shakeable preambles.
 *
 * DESIGN: Codegen traverses AST and sets flags. After traversal, preamble
 *         generator emits only what's needed for that specific program.
 */
export interface RequiredFeatures {
    // Error handling
    panic: boolean; // mori used - needs Panic class (TS) or includes (C++)

    // Collections (for targets that need imports)
    lista: boolean; // lista<T> or array methods
    tabula: boolean; // tabula<K,V>
    copia: boolean; // copia<T>

    // Async
    async: boolean; // futura, cede, promissum, figendum, variandum
    asyncIterator: boolean; // fiet, async for

    // Generators
    generator: boolean; // cursor, fiunt

    // Numeric types
    decimal: boolean; // decimus - needs decimal.js import

    // Enums
    enum: boolean; // ordo - needs Enum import (Python)

    // Compile-time evaluation
    praefixum: boolean; // praefixum blocks - needs __praefixum__ helper (Python)
}

/**
 * Create a RequiredFeatures object with all flags set to false.
 */
export function createRequiredFeatures(): RequiredFeatures {
    return {
        panic: false,
        lista: false,
        tabula: false,
        copia: false,
        async: false,
        asyncIterator: false,
        generator: false,
        decimal: false,
        enum: false,
        praefixum: false,
    };
}

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
    target?: CodegenTarget;

    /**
     * Indentation string for generated code.
     * WHY: TypeScript convention is 2 spaces, Zig convention is 4 spaces.
     *      Each target sets its own default.
     */
    indent?: string;

    /**
     * Whether to emit semicolons at end of statements.
     * TARGET: TypeScript only - Zig always requires semicolons.
     *         This option is ignored for Zig target.
     */
    semicolons?: boolean;
}
