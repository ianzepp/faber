/**
 * Code Generation Types - Configuration and shared utilities
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * Faber is the TypeScript-only reference compiler. This module defines
 * configuration and utilities for TypeScript code generation.
 *
 * For multi-target codegen (Python, Rust, Zig, C++), see Rivus.
 * See consilia/compiler-roles.md for compiler separation rationale.
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
 * Faber targets TypeScript only. For other targets, use Rivus.
 */
export type CodegenTarget = 'ts';

/**
 * Features used in the source code that require preamble setup.
 *
 * WHY: TypeScript codegen needs to know which features are used to emit
 *      appropriate imports and helper definitions in the preamble.
 *
 * DESIGN: Codegen traverses AST and sets flags. After traversal, preamble
 *         generator emits only what's needed for that specific program.
 */
export interface RequiredFeatures {
    // Error handling
    panic: boolean; // mori used - needs Panic class

    // Collections
    lista: boolean; // lista<T> or array methods
    tabula: boolean; // tabula<K,V>
    copia: boolean; // copia<T>

    // Async
    async: boolean; // futura, cede, promissum
    asyncIterator: boolean; // fiet, async for

    // Generators
    generator: boolean; // cursor, fiunt

    // Numeric types
    decimal: boolean; // decimus - needs decimal.js import

    // Flumina (streams-first)
    flumina: boolean; // fit functions using Responsum protocol

    // Regex
    usesRegex: boolean; // sed literals

    // Node.js modules - needs import statements
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
        flumina: false,
        usesRegex: false,
        fs: false,
        nodePath: false,
    };
}

/**
 * Configuration options for code generation.
 */
export interface CodegenOptions {
    /**
     * Indentation string for generated code.
     * Default: 2 spaces (TypeScript convention)
     */
    indent?: string;

    /**
     * Whether to emit semicolons at end of statements.
     * Default: true
     */
    semicolons?: boolean;

    /**
     * Absolute path to the source file.
     * Required for CLI module resolution (@ imperia ex module).
     */
    filePath?: string;
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
 * Comment syntax configuration for TypeScript.
 */
export interface CommentSyntax {
    line: string; // Line comment prefix: '//'
    blockStart: string; // Block comment start: '/*'
    blockEnd: string; // Block comment end: '*/'
}

/**
 * TypeScript comment syntax.
 */
export const COMMENT_SYNTAX: CommentSyntax = {
    line: '//',
    blockStart: '/*',
    blockEnd: '*/',
};

/**
 * Format a single comment for TypeScript output.
 *
 * @param comment - The comment to format
 * @param indent - Current indentation string
 * @returns Formatted comment string(s)
 */
export function formatComment(comment: Comment, indent: string): string {
    if (comment.type === 'line') {
        return `${indent}${COMMENT_SYNTAX.line} ${comment.value}`;
    }

    // Block or doc comment
    const lines = comment.value.split('\n');
    if (lines.length === 1) {
        return `${indent}${COMMENT_SYNTAX.blockStart} ${comment.value.trim()} ${COMMENT_SYNTAX.blockEnd}`;
    }

    // Multi-line: preserve formatting
    const result = [`${indent}${COMMENT_SYNTAX.blockStart}`];
    for (const line of lines) {
        result.push(`${indent} ${line.trim()}`);
    }
    result.push(`${indent} ${COMMENT_SYNTAX.blockEnd}`);
    return result.join('\n');
}

/**
 * Format leading comments for a node.
 *
 * @param node - The AST node
 * @param indent - Current indentation string
 * @returns Formatted leading comments with trailing newline, or empty string
 */
export function formatLeadingComments(node: BaseNode, indent: string): string {
    if (!node.leadingComments || node.leadingComments.length === 0) {
        return '';
    }
    return node.leadingComments.map(c => formatComment(c, indent)).join('\n') + '\n';
}

/**
 * Format trailing comments for a node.
 *
 * @param node - The AST node
 * @returns Formatted trailing comments with leading space, or empty string
 */
export function formatTrailingComments(node: BaseNode): string {
    if (!node.trailingComments || node.trailingComments.length === 0) {
        return '';
    }
    // Trailing comments go on the same line, so no indent
    return node.trailingComments.map(c => ` ${COMMENT_SYNTAX.line} ${c.value}`).join('');
}
