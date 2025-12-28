/**
 * Tokenizer Error Catalog
 *
 * COMPILER PHASE
 * ==============
 * lexical
 *
 * ARCHITECTURE
 * ============
 * Centralized error definitions for the lexical analysis phase. This module
 * provides structured error messages with both concise text and detailed help
 * for teaching Latin syntax to compiler users.
 *
 * Each error has a unique L-prefixed code for testability and tooling support.
 * Error messages are designed to be actionable - they explain what went wrong
 * and how to fix it.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Error code enum value
 * OUTPUT: Structured error object with text and help fields
 * ERRORS: N/A (this module defines errors, doesn't generate them)
 *
 * INVARIANTS
 * ==========
 * INV-1: Every TokenizerErrorCode has an entry in TOKENIZER_ERRORS
 * INV-2: All error codes start with 'L' (lexical phase prefix)
 * INV-3: Error codes are sequential starting from L001
 *
 * @module tokenizer/errors
 */

// =============================================================================
// ERROR CODES
// =============================================================================

/**
 * Lexical error codes for tokenization failures.
 *
 * WHY: L-prefix distinguishes lexical errors from parser (P) and semantic (S)
 *      errors. Sequential numbering makes adding new errors straightforward.
 *
 * DESIGN: Codes are strings (not numbers) for easier grep-ability and
 *         compatibility with error reporting tools.
 */
export enum TokenizerErrorCode {
    UnterminatedString = 'L001',
    UnterminatedTemplateString = 'L002',
    UnexpectedCharacter = 'L003',
    InvalidHexLiteral = 'L004',
    InvalidBinaryLiteral = 'L005',
    InvalidOctalLiteral = 'L006',
}

// =============================================================================
// ERROR CATALOG
// =============================================================================

/**
 * Structured error messages for all tokenizer errors.
 *
 * WHY: Separating error definitions from usage enables:
 *      - Consistent error formatting across the tokenizer
 *      - Easy review of all error messages in one place
 *      - Testability via error codes instead of brittle string matching
 *      - Future IDE integration (hover text, quick fixes)
 *
 * STRUCTURE:
 *   text: Brief error message (what went wrong)
 *   help: Detailed explanation with context (how to fix it)
 */
export const TOKENIZER_ERRORS = {
    [TokenizerErrorCode.UnterminatedString]: {
        text: 'Unterminated string',
        help: 'String literals must be closed on the same line. Use backticks (`) for multi-line strings.',
    },
    [TokenizerErrorCode.UnterminatedTemplateString]: {
        text: 'Unterminated template string',
        help: 'Template strings must end with a closing backtick (`). Check for missing backtick or unmatched braces.',
    },
    [TokenizerErrorCode.UnexpectedCharacter]: {
        text: 'Unexpected character',
        help: 'This character is not valid in Latin syntax. Check for typos or unsupported operators.',
    },
    [TokenizerErrorCode.InvalidHexLiteral]: {
        text: 'Invalid hex literal',
        help: 'Hex literals must have at least one digit after 0x prefix. Use 0x followed by hex digits (0-9, a-f, A-F).',
    },
    [TokenizerErrorCode.InvalidBinaryLiteral]: {
        text: 'Invalid binary literal',
        help: 'Binary literals must have at least one digit after 0b prefix. Use 0b followed by binary digits (0-1).',
    },
    [TokenizerErrorCode.InvalidOctalLiteral]: {
        text: 'Invalid octal literal',
        help: 'Octal literals must have at least one digit after 0o prefix. Use 0o followed by octal digits (0-7).',
    },
} as const;
