/**
 * ParserContext - Core State and Navigation Primitives
 *
 * The ParserContext class maintains all state for recursive descent parsing:
 * - Token stream and current position
 * - Comment accumulator for AST attachment
 * - Error accumulator for diagnostics
 * - Unique ID generator for synthetic bindings
 *
 * INVARIANTS:
 * - Token stream always ends with EOF token
 * - Position never advances past EOF
 * - Never throws - all errors collected in errors array
 * - Comments collected and attached to next AST node
 *
 * @module parser/context
 */

import type { Token, TokenType, Position } from '../tokenizer/types';
import type { Comment, Identifier } from './ast';
import { ParserErrorCode, PARSER_ERRORS } from './errors';
import { builtinTypes } from '../lexicon/types-builtin';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Parser error with source location.
 *
 * INVARIANT: position always references valid source location.
 * INVARIANT: code is from ParserErrorCode enum.
 * INVARIANT: message combines error text with context (e.g., "got 'x'").
 */
export interface ParserError {
    code: ParserErrorCode;
    message: string;
    position: Position;
}

// =============================================================================
// TYPE NAME LOOKUP
// =============================================================================

/**
 * Compute nominative singular form from stem, declension, and gender.
 *
 * WHY: Users write nominative forms in source (textus, numerus), but lexicon
 *      stores stems (text, numer). This computes the nominative for lookup.
 */
function computeNominative(stem: string, declension: number, gender: string): string {
    switch (declension) {
        case 1:
            return stem + 'a'; // lista, tabula, copia
        case 2:
            return gender === 'neuter' ? stem + 'um' : stem + 'us'; // numerus, datum
        case 3:
            return stem; // 3rd decl nominative varies - handled via nominative field
        case 4:
            return stem + 'us'; // textus, fluxus
        case 5:
            return stem + 'es';
        default:
            return stem;
    }
}

/**
 * Set of all builtin type names for quick lookup.
 *
 * PERF: Pre-computed Set enables O(1) type name checking.
 *
 * WHY: Used by isTypeName() to distinguish type names from regular identifiers
 *      in type-first syntax parsing (e.g., "fixum textus nomen" vs "fixum nomen").
 *
 * DESIGN: Computed from builtinTypes to avoid duplication. Uses nominative forms
 *         since that's what users write in source code.
 */
const BUILTIN_TYPE_NAMES = new Set(
    builtinTypes.map(t => t.nominative ?? computeNominative(t.stem, t.declension, t.gender))
);

// =============================================================================
// PARSER CONTEXT
// =============================================================================

/**
 * Parser state machine.
 *
 * Maintains token stream position, accumulated errors, and pending comments.
 * All parsing methods operate on this shared state via the Resolver interface.
 */
export class ParserContext {
    readonly tokens: Token[];
    current: number = 0;
    readonly errors: ParserError[] = [];
    private uniqueIdCounter: number = 0;
    private pendingComments: Comment[] = [];

    constructor(tokens: Token[]) {
        this.tokens = tokens;
    }

    // =========================================================================
    // Unique ID Generation
    // =========================================================================

    /**
     * Generate unique identifier for synthetic bindings.
     * WHY: Used for auto-generated names (e.g., cura arena {} without explicit binding).
     */
    genUniqueId(prefix: string): string {
        return `_${prefix}_${this.uniqueIdCounter++}`;
    }

    // =========================================================================
    // Comment Collection
    // =========================================================================

    /**
     * Convert a COMMENT token to a Comment AST node.
     */
    private tokenToComment(token: Token): Comment {
        return {
            type: token.commentType ?? 'line',
            value: token.value,
            position: token.position,
        };
    }

    /**
     * Consume all pending COMMENT tokens and add to pendingComments buffer.
     *
     * WHY: Called before parsing any statement or significant expression
     *      to collect comments that should be attached as leadingComments.
     */
    collectComments(): void {
        while (this.tokens[this.current]?.type === 'COMMENT') {
            this.pendingComments.push(this.tokenToComment(this.tokens[this.current]!));
            this.current++;
        }
    }

    /**
     * Consume pending comments and return them, clearing the buffer.
     *
     * WHY: Called when creating an AST node to attach collected comments.
     */
    consumePendingComments(): Comment[] | undefined {
        if (this.pendingComments.length === 0) {
            return undefined;
        }
        const comments = this.pendingComments;
        this.pendingComments = [];
        return comments;
    }

    /**
     * Check for trailing comment on the same line after current position.
     *
     * WHY: Trailing comments are on the same line as the node, after its content.
     *      Example: `fixum x = 5  // this is a trailing comment`
     */
    collectTrailingComment(nodeLine: number): Comment[] | undefined {
        if (
            this.tokens[this.current]?.type === 'COMMENT' &&
            this.tokens[this.current]!.position.line === nodeLine
        ) {
            const comment = this.tokenToComment(this.tokens[this.current]!);
            this.current++;
            return [comment];
        }
        return undefined;
    }

    // =========================================================================
    // Token Navigation
    // =========================================================================

    /**
     * Look ahead at token without consuming, skipping COMMENT tokens.
     *
     * INVARIANT: Returns EOF token if offset goes beyond end.
     */
    peek(offset = 0): Token {
        let pos = this.current;
        let skipped = 0;

        // Skip initial comment tokens
        while (pos < this.tokens.length && this.tokens[pos]!.type === 'COMMENT') {
            pos++;
        }

        // Skip 'offset' non-comment tokens
        while (skipped < offset && pos < this.tokens.length) {
            pos++;
            while (pos < this.tokens.length && this.tokens[pos]!.type === 'COMMENT') {
                pos++;
            }
            skipped++;
        }

        return this.tokens[pos] ?? this.tokens[this.tokens.length - 1]!;
    }

    /**
     * Check if at end of token stream.
     */
    isAtEnd(): boolean {
        return this.peek().type === 'EOF';
    }

    /**
     * Consume and return current token, skipping COMMENT tokens.
     *
     * INVARIANT: Never advances past EOF.
     */
    advance(): Token {
        this.collectComments();
        if (!this.isAtEnd()) {
            this.current++;
        }
        return this.tokens[this.current - 1]!;
    }

    /**
     * Check if current token matches type without consuming.
     */
    check(type: TokenType): boolean {
        return this.peek().type === type;
    }

    /**
     * Check if current token is specific keyword without consuming.
     *
     * WHY: Latin keywords are stored in token.keyword field, not token.type.
     */
    checkKeyword(keyword: string): boolean {
        return this.peek().type === 'KEYWORD' && this.peek().keyword === keyword;
    }

    /**
     * Match and consume token if type matches.
     *
     * @returns true if matched and consumed, false otherwise
     */
    match(...types: TokenType[]): boolean {
        for (const type of types) {
            if (this.check(type)) {
                this.advance();
                return true;
            }
        }
        return false;
    }

    /**
     * Match and consume token if keyword matches.
     *
     * @returns true if matched and consumed, false otherwise
     */
    matchKeyword(keyword: string): boolean {
        if (this.checkKeyword(keyword)) {
            this.advance();
            return true;
        }
        return false;
    }

    // =========================================================================
    // Error Handling
    // =========================================================================

    /**
     * Report error using error catalog.
     *
     * WHY: Centralizes error creation with consistent structure.
     *
     * @param code - Error code from ParserErrorCode enum
     * @param context - Optional context to append (e.g., "got 'x'")
     */
    reportError(code: ParserErrorCode, context?: string): void {
        const token = this.peek();
        const { text } = PARSER_ERRORS[code];
        const message = context ? `${text}, ${context}` : text;
        this.errors.push({ code, message, position: token.position });
    }

    /**
     * Expect specific token type or record error.
     *
     * ERROR RECOVERY: Records error and advances past the unexpected token to
     * prevent infinite loops. Returns a synthetic token of the expected type.
     *
     * WHY: If we don't advance, callers in loops (like parseObjectPattern) will
     * spin forever on the same unexpected token.
     *
     * @returns Matched token if found, synthetic token after advancing if not
     */
    expect(type: TokenType, code: ParserErrorCode): Token {
        if (this.check(type)) {
            return this.advance();
        }

        const token = this.peek();
        this.reportError(code, `got '${token.value}'`);

        // Advance past the unexpected token to prevent infinite loops
        if (!this.isAtEnd()) {
            this.advance();
        }

        // Return synthetic token with expected type but actual position
        return { type, value: '', position: token.position };
    }

    /**
     * Expect specific keyword or record error.
     *
     * ERROR RECOVERY: Records error and advances past the unexpected token.
     */
    expectKeyword(keyword: string, code: ParserErrorCode): Token {
        if (this.checkKeyword(keyword)) {
            return this.advance();
        }

        const token = this.peek();
        this.reportError(code, `got '${token.value}'`);

        // Advance past the unexpected token to prevent infinite loops
        if (!this.isAtEnd()) {
            this.advance();
        }

        // Return synthetic token with expected keyword but actual position
        return { type: 'KEYWORD', value: keyword, keyword, position: token.position };
    }

    /**
     * Record error and throw for error recovery.
     *
     * WHY: Used in expression parsing where we can't easily recover locally.
     *      Caught by statement parser which calls synchronize().
     */
    error(code: ParserErrorCode, context?: string): never {
        this.reportError(code, context);
        throw new Error(PARSER_ERRORS[code].text);
    }

    // =========================================================================
    // Error Recovery
    // =========================================================================

    /**
     * Skip tokens until reaching a statement boundary keyword.
     *
     * WHY: Called after catching parse error to resume at known-good state.
     */
    synchronize(): void {
        this.advance();
        while (!this.isAtEnd()) {
            if (
                // Annotations may precede declarations
                this.check('AT') ||
                // Blocks can always start a statement
                this.check('LBRACE') ||
                // Statement starters
                this.checkKeyword('ex') ||
                this.checkKeyword('de') ||
                this.checkKeyword('in') ||
                this.checkKeyword('varia') ||
                this.checkKeyword('fixum') ||
                this.checkKeyword('figendum') ||
                this.checkKeyword('variandum') ||
                this.checkKeyword('functio') ||
                this.checkKeyword('typus') ||
                this.checkKeyword('ordo') ||
                this.checkKeyword('genus') ||
                this.checkKeyword('pactum') ||
                this.checkKeyword('discretio') ||
                this.checkKeyword('si') ||
                this.checkKeyword('dum') ||
                this.checkKeyword('elige') ||
                this.checkKeyword('discerne') ||
                this.checkKeyword('custodi') ||
                this.checkKeyword('adfirma') ||
                this.checkKeyword('redde') ||
                this.checkKeyword('rumpe') ||
                this.checkKeyword('perge') ||
                this.checkKeyword('iace') ||
                this.checkKeyword('mori') ||
                this.checkKeyword('scribe') ||
                this.checkKeyword('vide') ||
                this.checkKeyword('mone') ||
                this.checkKeyword('tempta') ||
                this.checkKeyword('fac') ||
                this.checkKeyword('probandum') ||
                this.checkKeyword('proba') ||
                this.checkKeyword('ad') ||
                this.checkKeyword('incipit') ||
                this.checkKeyword('incipiet') ||
                this.checkKeyword('cura')
            ) {
                return;
            }
            this.advance();
        }
    }

    /**
     * Synchronize within genus body (for field/method recovery).
     *
     * WHY: Looks for field types, method declarations, or closing brace.
     */
    synchronizeGenusMember(): void {
        this.advance();
        let braceDepth = 0;

        while (!this.isAtEnd()) {
            if (this.check('LBRACE')) {
                braceDepth++;
                this.advance();
                continue;
            }

            if (this.check('RBRACE')) {
                if (braceDepth === 0) {
                    return;
                }
                braceDepth--;
                this.advance();
                continue;
            }

            // Stop at tokens that could start a new member (only at genus-body depth)
            if (
                braceDepth === 0 &&
                (this.check('AT') ||
                    this.checkKeyword('functio') ||
                    this.checkKeyword('publicus') ||
                    this.checkKeyword('privatus') ||
                    this.checkKeyword('protectus') ||
                    this.checkKeyword('abstractus') ||
                    this.checkKeyword('generis') ||
                    // Type annotations may begin with borrow prepositions or a type
                    this.checkKeyword('de') ||
                    this.checkKeyword('in') ||
                    this.check('LPAREN') ||
                    this.check('IDENTIFIER') ||
                    this.check('KEYWORD'))
            ) {
                return;
            }

            this.advance();
        }
    }

    // =========================================================================
    // Type Helpers
    // =========================================================================

    /**
     * Check if token is a builtin type name.
     *
     * WHY: Type-first syntax requires distinguishing type names from identifiers.
     *      "fixum textus nomen" (type-first) vs "fixum nomen" (type inference).
     *
     * @returns true if token is an identifier and a known builtin type
     */
    isTypeName(token: Token): boolean {
        return token.type === 'IDENTIFIER' && BUILTIN_TYPE_NAMES.has(token.value);
    }

    /**
     * Check if token is a preposition used in parameters.
     *
     * WHY: Prepositions indicate semantic roles:
     *      de = borrowed/read-only, in = mutable borrow, ex = source
     *      Note: 'ad' is reserved for statement-level dispatch, not parameters.
     *
     * @returns true if token is a preposition keyword
     */
    isPreposition(token: Token): boolean {
        return token.type === 'KEYWORD' && ['de', 'in', 'ex'].includes(token.keyword ?? '');
    }

    // =========================================================================
    // Identifier Parsing
    // =========================================================================

    /**
     * Parse an identifier token.
     *
     * @returns Identifier AST node
     */
    parseIdentifier(): Identifier {
        const token = this.expect('IDENTIFIER', ParserErrorCode.ExpectedIdentifier);
        return { type: 'Identifier', name: token.value, position: token.position };
    }

    /**
     * Parse identifier or keyword as a name.
     *
     * WHY: Import specifiers can be keywords (ex norma importa scribe).
     *      In this context, 'scribe' is a valid name, not a statement keyword.
     */
    parseIdentifierOrKeyword(): Identifier {
        const token = this.peek();

        if (token.type === 'IDENTIFIER' || token.type === 'KEYWORD') {
            this.advance();
            return { type: 'Identifier', name: token.value, position: token.position };
        }

        // Fall back to normal identifier parsing (will report error and advance)
        return this.parseIdentifier();
    }
}
