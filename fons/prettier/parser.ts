/**
 * Prettier Parser Wrapper
 *
 * ARCHITECTURE
 * ============
 * Thin wrapper around the Faber Romanus tokenizer and parser to provide
 * the interface Prettier expects. Handles comment extraction and attachment.
 *
 * @module prettier/parser
 */

import type { Program, Statement, Expression } from '../parser/ast.ts';
import type { Token } from '../tokenizer/types.ts';
import { tokenize } from '../tokenizer/index.ts';
import { parse } from '../parser/index.ts';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Comment node for Prettier's comment attachment system.
 */
export interface CommentNode {
    type: 'Comment';
    value: string;
    position: {
        line: number;
        column: number;
        offset: number;
    };
    endPosition: {
        line: number;
        column: number;
        offset: number;
    };
    leading: boolean;
    trailing: boolean;
    printed: boolean;
    isBlock: boolean;
}

/**
 * Extended Program node with comments array for Prettier.
 */
export interface PrettierProgram extends Program {
    comments: CommentNode[];
}

// AST node type for Prettier
export type AstNode = Program | Statement | Expression | CommentNode;

// =============================================================================
// COMMENT EXTRACTION
// =============================================================================

/**
 * Extract comments from source code.
 *
 * WHY: The tokenizer currently skips comments. We need to extract them
 *      separately for Prettier's comment attachment system.
 */
function extractComments(source: string): CommentNode[] {
    const comments: CommentNode[] = [];
    let i = 0;
    let line = 1;
    let column = 1;
    let lineStart = 0;

    while (i < source.length) {
        const char = source[i];
        const next = source[i + 1];

        // Track line and column
        if (char === '\n') {
            line++;
            column = 1;
            lineStart = i + 1;
            i++;
            continue;
        }

        // Single-line comment
        if (char === '/' && next === '/') {
            const startOffset = i;
            const startLine = line;
            const startColumn = column;

            i += 2; // Skip //
            column += 2;

            let value = '';
            while (i < source.length && source[i] !== '\n') {
                value += source[i];
                i++;
                column++;
            }

            comments.push({
                type: 'Comment',
                value: value.trim(),
                position: { line: startLine, column: startColumn, offset: startOffset },
                endPosition: { line, column, offset: i },
                leading: false,
                trailing: false,
                printed: false,
                isBlock: false,
            });
            continue;
        }

        // Multi-line comment
        if (char === '/' && next === '*') {
            const startOffset = i;
            const startLine = line;
            const startColumn = column;

            i += 2; // Skip /*
            column += 2;

            let value = '';
            while (i < source.length - 1 && !(source[i] === '*' && source[i + 1] === '/')) {
                if (source[i] === '\n') {
                    line++;
                    column = 1;
                    lineStart = i + 1;
                    value += '\n';
                } else {
                    value += source[i];
                    column++;
                }
                i++;
            }

            i += 2; // Skip */
            column += 2;

            comments.push({
                type: 'Comment',
                value: value.trim(),
                position: { line: startLine, column: startColumn, offset: startOffset },
                endPosition: { line, column, offset: i },
                leading: false,
                trailing: false,
                printed: false,
                isBlock: true,
            });
            continue;
        }

        // Skip strings to avoid false positives
        if (char === '"' || char === "'") {
            const quote = char;
            i++;
            column++;
            while (i < source.length && source[i] !== quote) {
                if (source[i] === '\\' && i + 1 < source.length) {
                    i += 2;
                    column += 2;
                } else if (source[i] === '\n') {
                    line++;
                    column = 1;
                    lineStart = i + 1;
                    i++;
                } else {
                    i++;
                    column++;
                }
            }
            i++; // Skip closing quote
            column++;
            continue;
        }

        // Skip template strings
        if (char === '`') {
            i++;
            column++;
            while (i < source.length && source[i] !== '`') {
                if (source[i] === '\\' && i + 1 < source.length) {
                    i += 2;
                    column += 2;
                } else if (source[i] === '\n') {
                    line++;
                    column = 1;
                    lineStart = i + 1;
                    i++;
                } else {
                    i++;
                    column++;
                }
            }
            i++; // Skip closing backtick
            column++;
            continue;
        }

        i++;
        column++;
    }

    return comments;
}

// =============================================================================
// PARSER INTERFACE
// =============================================================================

/**
 * Parse source code into AST for Prettier.
 *
 * This wraps the Faber Romanus parser and adds comment extraction.
 */
export function faberParse(text: string): PrettierProgram {
    // Tokenize
    const { tokens, errors: lexErrors } = tokenize(text);

    if (lexErrors.length > 0) {
        const err = lexErrors[0];
        throw new SyntaxError(`Lexical error at ${err.position.line}:${err.position.column}: ${err.message}`);
    }

    // Parse
    const { program, errors: parseErrors } = parse(tokens);

    if (!program) {
        const err = parseErrors[0];
        throw new SyntaxError(`Parse error at ${err?.position.line}:${err?.position.column}: ${err?.message ?? 'Unknown error'}`);
    }

    // Extract comments for Prettier
    // WHY: Only attach comments when there are no parse errors.
    // Parse errors can cause statement positions to be inconsistent,
    // leading to "Comment location overlaps with node location" errors.
    const comments = parseErrors.length === 0 ? extractComments(text) : [];

    return {
        ...program,
        comments,
    };
}

/**
 * Get the start offset of an AST node.
 *
 * WHY: Prettier calls this on all values it finds while walking the AST,
 *      including strings and primitives. We must return undefined for non-nodes.
 */
export function locStart(node: unknown): number | undefined {
    if (!node || typeof node !== 'object') {
        return undefined;
    }

    const n = node as any;

    if (n.position && typeof n.position.offset === 'number') {
        return n.position.offset;
    }

    return undefined;
}

/**
 * Get the end offset of an AST node.
 *
 * WHY: AST nodes don't track end position. We estimate based on node type
 *      and content to give Prettier approximate boundaries for comment attachment.
 */
export function locEnd(node: unknown): number | undefined {
    if (!node || typeof node !== 'object') {
        return undefined;
    }

    const n = node as any;

    if (n.type === 'Comment' && n.endPosition) {
        return n.endPosition.offset;
    }

    if (!n.position || typeof n.position.offset !== 'number') {
        return undefined;
    }

    const start = n.position.offset;

    // Estimate end based on node type
    switch (n.type) {
        case 'Identifier':
            return start + (n.name?.length ?? 0);

        case 'Literal':
            return start + (n.raw?.length ?? 0);

        case 'TemplateLiteral':
            return start + (n.raw?.length ?? 0);

        default:
            // For complex nodes, we don't have a good estimate
            // Return start + 1 to at least give some range
            return start + 1;
    }
}
