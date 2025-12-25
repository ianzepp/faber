/**
 * Comment Handling for Prettier
 *
 * ARCHITECTURE
 * ============
 * Provides functions for Prettier's comment attachment and printing system.
 * Comments are extracted during parsing and attached to AST nodes based on
 * their position relative to code.
 *
 * @module prettier/comments
 */

import type { AstPath, Doc } from 'prettier';
import { doc } from 'prettier';
import type { AstNode, CommentNode } from './parser.ts';

const { hardline, join } = doc.builders;

// =============================================================================
// COMMENT PREDICATES
// =============================================================================

/**
 * Determine if comments can be attached to a node.
 *
 * WHY: Prettier needs to know which nodes can have comments.
 *      Most nodes can, but some synthetic nodes cannot.
 */
export function canAttachComment(node: AstNode): boolean {
    // All real AST nodes can have comments attached
    return node.type !== 'Comment';
}

/**
 * Determine if a comment is a block comment (multi-line).
 */
export function isBlockComment(comment: CommentNode): boolean {
    return comment.isBlock;
}

// =============================================================================
// COMMENT PRINTING
// =============================================================================

/**
 * Print a comment node.
 *
 * Style rules:
 * - Line comments: // followed by space and content
 * - Block comments: Multi-line with proper indentation
 */
export function printComment(path: AstPath<CommentNode>): Doc {
    const comment = path.getValue();

    if (comment.isBlock) {
        // Block comment: /** ... */ style for doc comments, /* ... */ otherwise
        const lines = comment.value.split('\n');

        if (lines.length === 1) {
            // Single-line block comment
            if (comment.value.startsWith('*')) {
                return ['/** ', comment.value.slice(1).trim(), ' */'];
            }
            return ['/* ', comment.value, ' */'];
        }

        // Multi-line block comment - format as doc comment
        const isDocComment = comment.value.startsWith('*');
        const prefix = isDocComment ? '/**' : '/*';

        const formattedLines = lines.map((line, i) => {
            const trimmed = line.trim();
            // Remove leading * from doc comments for consistent formatting
            const content = trimmed.startsWith('*') ? trimmed.slice(1).trim() : trimmed;
            if (i === 0 && isDocComment) {
                return content; // First line content (after /**)
            }
            return content ? [' * ', content] : ' *';
        });

        return [prefix, hardline, join(hardline, formattedLines.slice(isDocComment ? 0 : 0)), hardline, ' */'];
    }

    // Line comment: // content
    return ['// ', comment.value];
}
