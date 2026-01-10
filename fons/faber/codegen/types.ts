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
export type CodegenTarget = 'ts' | 'zig' | 'py' | 'rs' | 'cpp' | 'fab';

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

    // Callable (Python) - for function type annotations
    callable: boolean; // (T) -> U type annotations - needs Callable import

    // Compile-time evaluation
    praefixum: boolean; // praefixum blocks - needs __praefixum__ helper (Python)

    // Dataclasses (Python)
    dataclass: boolean; // discretio - needs dataclass import

    // Flumina (streams-first)
    flumina: boolean; // fit functions using Responsum protocol

    // Regex
    usesRegex: boolean; // sed literals - needs re import (Python) or regex crate (Rust)

    // Math (Python) - needs import math
    math: boolean;

    // Random (Python) - needs import random
    random: boolean;

    // UUID (Python) - needs import uuid
    uuid: boolean;

    // Secrets (Python) - needs import secrets (for crypto-safe random bytes)
    secrets: boolean;

    // I/O (Python) - needs import sys for stderr output
    sys: boolean;

    // Warnings (Python) - needs import warnings for _mone
    warnings: boolean;

    // Time (Python) - needs import time for norma/tempus
    time: boolean;

    // I/O streams (Zig) - needs stdout/stderr/stdin setup
    stdout: boolean;
    stderr: boolean;
    stdin: boolean;

    // Node.js modules (TS) - needs import statements
    fs: boolean; // norma/solum - needs import * as fs from 'fs'
    nodePath: boolean; // norma/solum path utils - needs import * as path from 'path'
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
        callable: false,
        praefixum: false,
        dataclass: false,
        flumina: false,
        usesRegex: false,
        math: false,
        random: false,
        uuid: false,
        secrets: false,
        sys: false,
        warnings: false,
        time: false,
        stdout: false,
        stderr: false,
        stdin: false,
        fs: false,
        nodePath: false,
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

// =============================================================================
// COMMENT FORMATTING
// =============================================================================

import type { Comment, BaseNode, Annotation, Visibility } from '../parser/ast';

// Re-export for use by statement generators
export type { Visibility };

// =============================================================================
// ANNOTATION UTILITIES
// =============================================================================

/**
 * Extract visibility from annotations.
 *
 * WHY: Visibility modifiers in Latin have gender agreement, but all forms
 *      map to the same semantic meaning. This normalizes to English visibility.
 *
 * @param annotations - Array of annotations from an AST node
 * @returns Visibility level, or 'private' if not specified
 */
export function getVisibilityFromAnnotations(annotations?: Annotation[]): Visibility {
    if (!annotations) return 'private';

    for (const ann of annotations) {
        if (ann.name === 'publicum' || ann.name === 'publica' || ann.name === 'publicus') return 'public';
        if (ann.name === 'privatum' || ann.name === 'privata' || ann.name === 'privatus') return 'private';
        if (ann.name === 'protectum' || ann.name === 'protecta' || ann.name === 'protectus') return 'protected';
    }

    return 'private';
}

/**
 * Check if annotations include abstract modifier.
 *
 * @param annotations - Array of annotations from an AST node
 * @returns true if abstractum/abstracta/abstractus is present
 */
export function isAbstractFromAnnotations(annotations?: Annotation[]): boolean {
    if (!annotations) return false;

    for (const ann of annotations) {
        if (ann.name === 'abstractum' || ann.name === 'abstracta' || ann.name === 'abstractus') return true;
    }

    return false;
}

/**
 * Check if annotations include async modifier.
 *
 * @param annotations - Array of annotations from an AST node
 * @returns true if futura is present
 */
export function isAsyncFromAnnotations(annotations?: Annotation[]): boolean {
    if (!annotations) return false;

    for (const ann of annotations) {
        if (ann.name === 'futura') return true;
    }

    return false;
}

/**
 * Check if annotations include generator modifier.
 *
 * @param annotations - Array of annotations from an AST node
 * @returns true if cursor is present
 */
export function isGeneratorFromAnnotations(annotations?: Annotation[]): boolean {
    if (!annotations) return false;

    for (const ann of annotations) {
        if (ann.name === 'cursor') return true;
    }

    return false;
}

/**
 * Check if annotations include static modifier.
 *
 * @param annotations - Array of annotations from an AST node
 * @returns true if generis is present
 */
export function isStaticFromAnnotations(annotations?: Annotation[]): boolean {
    if (!annotations) return false;

    for (const ann of annotations) {
        if (ann.name === 'generis') return true;
    }

    return false;
}

/**
 * Check if annotations include external declaration modifier.
 *
 * WHY: External declarations tell the compiler the symbol exists but is
 *      provided elsewhere (runtime, FFI, linker). No initializer or body required.
 *
 * @param annotations - Array of annotations from an AST node
 * @returns true if externa is present
 */
export function isExternaFromAnnotations(annotations?: Annotation[]): boolean {
    if (!annotations) return false;

    for (const ann of annotations) {
        if (ann.name === 'externa') return true;
    }

    return false;
}

/**
 * Comment syntax configuration per target.
 *
 * WHY: Different targets have different comment syntax:
 *      - TS/Rust/C++: // line and slash-star block
 *      - Python: # line and """ block
 *      - Zig: // line only (no block comments)
 */
export interface CommentSyntax {
    line: string; // Line comment prefix (e.g., '//', '#')
    blockStart: string | null; // Block comment start (e.g., '/*', '"""') or null if not supported
    blockEnd: string | null; // Block comment end (e.g., '*/', '"""') or null if not supported
}

/**
 * Comment syntax for each target.
 */
export const COMMENT_SYNTAX: Record<CodegenTarget, CommentSyntax> = {
    ts: { line: '//', blockStart: '/*', blockEnd: '*/' },
    py: { line: '#', blockStart: '"""', blockEnd: '"""' },
    rs: { line: '//', blockStart: '/*', blockEnd: '*/' },
    cpp: { line: '//', blockStart: '/*', blockEnd: '*/' },
    zig: { line: '//', blockStart: null, blockEnd: null }, // Zig has no block comments
    fab: { line: '#', blockStart: null, blockEnd: null },
};

/**
 * Format a single comment for a target.
 *
 * @param comment - The comment to format
 * @param syntax - The comment syntax for the target
 * @param indent - Current indentation string
 * @returns Formatted comment string(s)
 */
export function formatComment(comment: Comment, syntax: CommentSyntax, indent: string): string {
    if (comment.type === 'line') {
        return `${indent}${syntax.line} ${comment.value}`;
    }

    // Block or doc comment
    if (syntax.blockStart && syntax.blockEnd) {
        // Multi-line block comment
        const lines = comment.value.split('\n');
        if (lines.length === 1) {
            return `${indent}${syntax.blockStart} ${comment.value.trim()} ${syntax.blockEnd}`;
        }
        // Multi-line: preserve formatting
        const result = [`${indent}${syntax.blockStart}`];
        for (const line of lines) {
            result.push(`${indent} ${line.trim()}`);
        }
        result.push(`${indent} ${syntax.blockEnd}`);
        return result.join('\n');
    }

    // Target doesn't support block comments (e.g., Zig) - convert to line comments
    const lines = comment.value.split('\n');
    return lines.map(line => `${indent}${syntax.line} ${line.trim()}`).join('\n');
}

/**
 * Format leading comments for a node.
 *
 * @param node - The AST node
 * @param syntax - The comment syntax for the target
 * @param indent - Current indentation string
 * @returns Formatted leading comments with trailing newline, or empty string
 */
export function formatLeadingComments(node: BaseNode, syntax: CommentSyntax, indent: string): string {
    if (!node.leadingComments || node.leadingComments.length === 0) {
        return '';
    }
    return node.leadingComments.map(c => formatComment(c, syntax, indent)).join('\n') + '\n';
}

/**
 * Format trailing comments for a node.
 *
 * @param node - The AST node
 * @param syntax - The comment syntax for the target
 * @returns Formatted trailing comments with leading space, or empty string
 */
export function formatTrailingComments(node: BaseNode, syntax: CommentSyntax): string {
    if (!node.trailingComments || node.trailingComments.length === 0) {
        return '';
    }
    // Trailing comments go on the same line, so no indent
    return node.trailingComments.map(c => ` ${syntax.line} ${c.value}`).join('');
}
