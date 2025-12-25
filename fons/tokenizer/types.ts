/**
 * Tokenizer Type Definitions
 *
 * COMPILER PHASE
 * ==============
 * lexical
 *
 * ARCHITECTURE
 * ============
 * Defines the core data structures for lexical analysis in the Faber Romanus
 * compiler. The tokenizer transforms raw Latin source text into a stream of
 * typed tokens that preserve source position information for error reporting.
 *
 * This phase separates the character-level concerns (whitespace, comments,
 * string escaping) from the syntactic structure that the parser will handle.
 * Tokens represent the minimal units of meaning in the language.
 *
 * The tokenizer uses a single KEYWORD token type with an optional keyword
 * field to avoid creating separate token types for each Latin keyword. This
 * allows the lexicon to evolve without changing the tokenizer.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Raw source code as string
 * OUTPUT: Stream of Token objects with Position metadata
 * ERRORS: TokenizerError objects for unterminated strings, invalid characters
 *
 * INVARIANTS
 * ==========
 * INV-1: Every Token has a valid Position (line >= 1, column >= 1, offset >= 0)
 * INV-2: Token stream always ends with exactly one EOF token
 * INV-3: KEYWORD tokens always have the keyword field populated
 * INV-4: Position.offset points to the start of the token in the source string
 *
 * @module tokenizer/types
 */

// =============================================================================
// TOKEN TYPES
// =============================================================================

/**
 * Token type discriminator for lexical analysis.
 *
 * WHY: Separate token types rather than a single "TOKEN" type enables
 *      exhaustive matching in the parser and clear error messages.
 *
 * DESIGN: Keywords use a single KEYWORD type with optional keyword field
 *         to decouple tokenizer from the evolving Latin lexicon.
 *
 * DESIGN: ARROW (=>) and THIN_ARROW (->) are distinct to support both
 *         fat arrow functions (modern style) and type annotations (Zig-like).
 */
export type TokenType =
    // ---------------------------------------------------------------------------
    // Literals
    // ---------------------------------------------------------------------------
    | 'NUMBER' // Integer or decimal: 123, 45.67
    | 'BIGINT' // Bigint literal with 'n' suffix: 123n, 456789n
    | 'STRING' // Single or double quoted: "text", 'text'
    | 'TEMPLATE_STRING' // Backtick with interpolation: `template ${expr}`
    | 'IDENTIFIER' // Variable/function names: variableName

    // ---------------------------------------------------------------------------
    // Keywords
    // ---------------------------------------------------------------------------
    | 'KEYWORD' // Latin keywords: si, varia, functio, etc.

    // ---------------------------------------------------------------------------
    // Operators
    // ---------------------------------------------------------------------------
    | 'PLUS' // +
    | 'MINUS' // -
    | 'STAR' // *
    | 'SLASH' // /
    | 'PERCENT' // %
    | 'EQUAL' // =
    | 'EQUAL_EQUAL' // ==
    | 'TRIPLE_EQUAL' // ===
    | 'BANG' // !
    | 'BANG_EQUAL' // !=
    | 'BANG_DOUBLE_EQUAL' // !==
    | 'LESS' // <
    | 'LESS_EQUAL' // <=
    | 'GREATER' // >
    | 'GREATER_EQUAL' // >=
    | 'AND' // &&
    | 'OR' // ||
    | 'ARROW' // => (fat arrow for lambdas)
    | 'THIN_ARROW' // -> (type annotations, Zig-style returns)
    | 'DOT' // .
    | 'DOT_DOT' // .. (range operator)
    | 'QUESTION' // ?
    | 'COLON' // :
    | 'PIPE' // | (single pipe for union types)

    // ---------------------------------------------------------------------------
    // Delimiters
    // ---------------------------------------------------------------------------
    | 'LPAREN' // (
    | 'RPAREN' // )
    | 'LBRACE' // {
    | 'RBRACE' // }
    | 'LBRACKET' // [
    | 'RBRACKET' // ]
    | 'COMMA' // ,
    | 'SEMICOLON' // ;

    // ---------------------------------------------------------------------------
    // Special
    // ---------------------------------------------------------------------------
    | 'NEWLINE' // Line break (currently unused, may be significant later)
    | 'EOF' // End of file sentinel
    | 'COMMENT'; // Comment token (for tooling, not emitted by default)

// =============================================================================
// POSITION TRACKING
// =============================================================================

/**
 * Source position metadata for error reporting.
 *
 * WHY: Track line, column, and offset to support both human-readable error
 *      messages (line:column) and editor integrations (byte offset).
 *
 * INVARIANT: line >= 1, column >= 1, offset >= 0
 */
export interface Position {
    line: number; // 1-based line number
    column: number; // 1-based column number (visual characters, not bytes)
    offset: number; // 0-based byte offset into source string
}

// =============================================================================
// TOKEN REPRESENTATION
// =============================================================================

/**
 * A single lexical token with source position.
 *
 * WHY: value field preserves original source text for:
 *      - Exact error message reproduction
 *      - Preserving numeric literal precision
 *      - Template string interpolation
 *
 * INVARIANT: If type === "KEYWORD", keyword field must be populated
 */
export interface Token {
    type: TokenType;
    value: string; // Original source text of this token
    position: Position; // Source location for error reporting
    keyword?: string; // For KEYWORD tokens: the specific Latin keyword
}

// =============================================================================
// ERROR REPORTING
// =============================================================================

/**
 * Lexical error encountered during tokenization.
 *
 * WHY: Separate from Token to allow continued tokenization after errors.
 *      The tokenizer never throws - it collects errors and continues.
 *
 * WHY: Include code field for testability and tooling support. Tests can
 *      assert on error codes rather than brittle message string matching.
 *
 * EDGE: Position points to where the error was detected, which may not be
 *       where the problem started (e.g., unterminated string detected at EOF).
 */
export interface TokenizerError {
    code: string;
    text: string;
    help: string;
    position: Position;
}

/**
 * Complete result of tokenization.
 *
 * WHY: Return both tokens and errors to enable partial compilation.
 *      Even with lexical errors, we may produce useful tokens for IDE features.
 *
 * INVARIANT: tokens array always contains at least one EOF token
 */
export interface TokenizerResult {
    tokens: Token[];
    errors: TokenizerError[];
}
