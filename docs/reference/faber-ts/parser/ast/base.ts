/**
 * AST Base Types - Foundation types for all AST nodes
 *
 * @module parser/ast/base
 */

import type { Position } from '../../tokenizer/types';
import type { SemanticType } from '../../semantic/types';

// =============================================================================
// COMMENT TYPES
// =============================================================================

/**
 * Comment type discriminator.
 *
 * Faber uses # for all comments. Only 'line' type exists.
 */
export type CommentType = 'line';

/**
 * Comment node attached to AST nodes.
 *
 * INVARIANT: value contains comment text without delimiters.
 * INVARIANT: position points to the start of the comment in source.
 *
 * WHY: Comments are preserved through the pipeline for:
 *      - Source-to-source transformation (formatting)
 *      - Code generation with documentation
 *      - IDE tooling and documentation extraction
 */
export interface Comment {
    type: CommentType;
    value: string;
    position: Position;
}

// =============================================================================
// BASE NODE
// =============================================================================

/**
 * Base node with position for error reporting.
 *
 * INVARIANT: Every AST node extends this to ensure source tracking.
 *
 * The resolvedType field is populated by the semantic analyzer and used by
 * code generators to make type-aware decisions.
 *
 * Comments are attached during parsing:
 * - leadingComments: comments appearing before this node (on previous lines)
 * - trailingComments: comments on the same line after this node
 */
export interface BaseNode {
    position: Position;
    resolvedType?: SemanticType;
    leadingComments?: Comment[];
    trailingComments?: Comment[];
}

// =============================================================================
// VISIBILITY
// =============================================================================

/**
 * Visibility level for genus members.
 *
 * WHY: Three-level visibility matching TypeScript/C++/Python:
 *   - public: accessible from anywhere (default, struct semantics)
 *   - protected: accessible from class and subclasses
 *   - private: accessible only within the class
 */
export type Visibility = 'public' | 'protected' | 'private';
