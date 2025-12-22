/**
 * Lexical Analyzer (Tokenizer)
 *
 * COMPILER PHASE
 * ==============
 * lexical
 *
 * ARCHITECTURE
 * ============
 * Implements a single-pass lexical scanner for Faber Romanus source code.
 * Transforms raw character input into a typed token stream suitable for
 * syntactic analysis.
 *
 * The tokenizer operates as a finite state machine, advancing through the
 * source character by character. It handles:
 * - Multi-character operators (==, !=, <=, >=, &&, ||, =>, ->)
 * - String literals with escape sequences (", ', `)
 * - Template strings with interpolation tracking
 * - Single and multi-line comments (C++ and C style)
 * - Latin keywords identified via the lexicon module
 *
 * The scanner never throws on invalid input. Instead, it collects errors
 * and continues tokenizing to provide the best possible error recovery for
 * IDE tooling and batch compilation.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Raw Latin source code as UTF-8 string
 * OUTPUT: TokenizerResult containing:
 *         - tokens: Array of Token objects (always ends with EOF)
 *         - errors: Array of TokenizerError for lexical issues
 * ERRORS: Unterminated strings, unterminated template strings, unexpected characters
 *
 * INVARIANTS
 * ==========
 * INV-1: Token stream always ends with exactly one EOF token
 * INV-2: Position tracking never goes backwards (line/column/offset only increase)
 * INV-3: Tokenizer never throws - all errors collected in result.errors
 * INV-4: Keyword detection is case-insensitive (Latin had no case distinction)
 *
 * CHARACTER CLASSES
 * =================
 * Digit:         0-9
 * Alpha:         a-z, A-Z, _ (underscore allowed in identifiers)
 * AlphaNumeric:  Alpha | Digit
 * Whitespace:    space, tab, \r, \n
 *
 * TOKEN PATTERNS
 * ==============
 * NUMBER:          digit+ ('.' digit+)?
 * STRING:          '"' (escape | [^"\n])* '"' | "'" (escape | [^'\n])* "'"
 * TEMPLATE_STRING: '`' (escape | [^`] | '${' expr '}')* '`'
 * IDENTIFIER:      alpha alphanumeric*
 * KEYWORD:         IDENTIFIER matching lexicon entry
 *
 * OPERATOR LOOKAHEAD:
 * '=' -> '==' | '=>' | '='
 * '!' -> '!=' | '!'
 * '<' -> '<=' | '<'
 * '>' -> '>=' | '>'
 * '&' -> '&&' (single '&' is invalid)
 * '|' -> '||' | '|'
 * '-' -> '->' | '-'
 * '/' -> '//' comment | '/*' comment | '/'
 *
 * @module tokenizer
 */

import type { Token, TokenType, Position, TokenizerResult, TokenizerError } from './types';
import { isKeyword, getKeyword } from '../lexicon';

// =============================================================================
// MAIN TOKENIZER FUNCTION
// =============================================================================

/**
 * Tokenize Latin source code into a stream of tokens.
 *
 * WHY: Single entry point for lexical analysis, returns both success and
 *      error information to enable partial compilation and IDE features.
 *
 * @param source - Raw Latin source code as UTF-8 string
 * @returns TokenizerResult with tokens and any errors encountered
 */
export function tokenize(source: string): TokenizerResult {
    const tokens: Token[] = [];
    const errors: TokenizerError[] = [];

    // ---------------------------------------------------------------------------
    // Scanner State
    // ---------------------------------------------------------------------------

    let current = 0; // Current position in source string
    let line = 1; // Current line (1-based)

    let column = 1; // Current column (1-based) - tracked for potential future use
    let lineStart = 0; // Offset of current line start (for column calculation)

    // ---------------------------------------------------------------------------
    // Position Tracking
    // ---------------------------------------------------------------------------

    /**
     * Capture current source position.
     *
     * WHY: Position snapshots taken at token start, not end, to point error
     *      messages at the beginning of problematic tokens.
     */
    function position(): Position {
        return { line, column: current - lineStart + 1, offset: current };
    }

    // ---------------------------------------------------------------------------
    // Character Navigation
    // ---------------------------------------------------------------------------

    /**
     * Look ahead at upcoming characters without advancing.
     *
     * WHY: Enables multi-character operator detection (==, !=, etc.) and
     *      comment recognition (//, /*) without consuming input.
     *
     * @param offset - How many characters ahead to look (default 0 = current)
     * @returns Character at position, or empty string if past end
     */
    function peek(offset = 0): string {
        return source[current + offset] ?? '';
    }

    /**
     * Consume current character and advance position.
     *
     * WHY: Centralizes position tracking to maintain invariant that
     *      line/column/offset always advance together.
     */
    function advance(): string {
        const char = source[current];

        current++;
        column++;

        return char;
    }

    /**
     * Check if we've consumed all input.
     */
    function isAtEnd(): boolean {
        return current >= source.length;
    }

    // ---------------------------------------------------------------------------
    // Character Class Predicates
    // ---------------------------------------------------------------------------

    /**
     * Check if character is a decimal digit.
     *
     * WHY: Using ASCII range comparison is faster than regex and avoids
     *      allocating regex objects in the hot tokenization loop.
     */
    function isDigit(char: string): boolean {
        return char >= '0' && char <= '9';
    }

    /**
     * Check if character can start an identifier.
     *
     * WHY: Allows underscore prefix (common in modern languages) while
     *      excluding digits (which cannot start identifiers).
     */
    function isAlpha(char: string): boolean {
        return (char >= 'a' && char <= 'z') || (char >= 'A' && char <= 'Z') || char === '_';
    }

    /**
     * Check if character can continue an identifier.
     */
    function isAlphaNumeric(char: string): boolean {
        return isAlpha(char) || isDigit(char);
    }

    // ---------------------------------------------------------------------------
    // Token and Error Collection
    // ---------------------------------------------------------------------------

    /**
     * Add a token to the output stream.
     *
     * @param type - Token type discriminator
     * @param value - Original source text of token
     * @param pos - Source position where token started
     * @param keyword - For KEYWORD tokens, the specific Latin keyword
     */
    function addToken(type: TokenType, value: string, pos: Position, keyword?: string): void {
        tokens.push({ type, value, position: pos, keyword });
    }

    /**
     * Record a lexical error without stopping tokenization.
     *
     * WHY: Collecting errors rather than throwing enables:
     *      - IDE error squiggles for all errors at once
     *      - Partial compilation when some sections are valid
     *      - Better batch error reporting
     */
    function addError(message: string, pos: Position): void {
        errors.push({ message, position: pos });
    }

    // =============================================================================
    // WHITESPACE AND COMMENT HANDLING
    // =============================================================================

    /**
     * Skip over whitespace and comments.
     *
     * WHY: Latin syntax (like C/JavaScript) treats whitespace as insignificant.
     *      Handling comments here keeps the main scanner logic clean.
     *
     * DESIGN: Comments are consumed but not emitted as tokens by default.
     *         The commented-out COMMENT token emission can be enabled for
     *         documentation generators or IDE tooling.
     */
    function skipWhitespace(): void {
        while (!isAtEnd()) {
            const char = peek();

            if (char === ' ' || char === '\t' || char === '\r') {
                advance();
            } else if (char === '\n') {
                advance();
                line++;
                lineStart = current;
            } else if (char === '/' && peek(1) === '/') {
                // WHY: Single-line comments extend to end of line (C++ style)

                const pos = position();
                let comment = '';

                advance(); // /
                advance(); // /
                while (!isAtEnd() && peek() !== '\n') {
                    comment += advance();
                }
                // FUTURE: Emit comment tokens for documentation tooling
                // addToken("COMMENT", comment.trim(), pos)
            } else if (char === '/' && peek(1) === '*') {
                // WHY: Multi-line comments can span multiple lines (C style)
                advance(); // /
                advance(); // *
                while (!isAtEnd() && !(peek() === '*' && peek(1) === '/')) {
                    if (peek() === '\n') {
                        line++;
                        lineStart = current + 1;
                    }

                    advance();
                }

                if (!isAtEnd()) {
                    advance(); // *
                    advance(); // /
                }
            } else {
                break;
            }
        }
    }

    // =============================================================================
    // LITERAL SCANNING
    // =============================================================================

    /**
     * Scan a numeric literal.
     *
     * PATTERN: digit+ ('.' digit+)?
     *
     * WHY: Supports both integers (123) and decimals (45.67).
     *      Does not handle scientific notation (1e10) or hex (0xFF) yet.
     *
     * WHY: Lookahead required for decimal point to distinguish from
     *      member access: 123.toString() vs 123.45
     */
    function scanNumber(): void {
        const pos = position();
        let value = '';

        // Integer part
        while (isDigit(peek())) {
            value += advance();
        }

        // Decimal part (only if followed by digit to avoid member access)
        if (peek() === '.' && isDigit(peek(1))) {
            value += advance(); // .
            while (isDigit(peek())) {
                value += advance();
            }
        }

        addToken('NUMBER', value, pos);
    }

    /**
     * Scan a string literal (single or double quoted).
     *
     * PATTERN: '"' (escape | [^"\n])* '"' | "'" (escape | [^'\n])* "'"
     *
     * WHY: Supports standard escape sequences (\n, \t, \r, \\, \", \')
     *      for compatibility with JavaScript/TypeScript output.
     *
     * ERROR RECOVERY: On newline or EOF before closing quote, emit error
     *                 but don't emit a malformed STRING token.
     *
     * @param quote - The quote character that opened this string (' or ")
     */
    function scanString(quote: string): void {
        const pos = position();
        let value = '';

        advance(); // opening quote

        while (!isAtEnd() && peek() !== quote) {
            if (peek() === '\n') {
                addError('Unterminated string', pos);

                return;
            }

            if (peek() === '\\') {
                advance();
                const escaped = advance();

                // WHY: Map escape sequences to their runtime equivalents
                switch (escaped) {
                    case 'n':
                        value += '\n';
                        break;
                    case 't':
                        value += '\t';
                        break;
                    case 'r':
                        value += '\r';
                        break;
                    case '\\':
                        value += '\\';
                        break;
                    case quote:
                        value += quote;
                        break;
                    default:
                        value += escaped; // Unknown escapes pass through
                }
            } else {
                value += advance();
            }
        }

        if (isAtEnd()) {
            addError('Unterminated string', pos);

            return;
        }

        advance(); // closing quote
        addToken('STRING', value, pos);
    }

    /**
     * Scan a template string literal.
     *
     * PATTERN: '`' (escape | [^`] | '${' expr '}')* '`'
     *
     * WHY: Template strings allow multi-line content (unlike regular strings)
     *      and interpolation via ${expression} syntax.
     *
     * DESIGN: Currently captures the entire template as a single token,
     *         including interpolation syntax. A more sophisticated approach
     *         would tokenize interpolated expressions separately, but that
     *         requires tracking brace nesting depth.
     *
     * WHY: Track brace depth to handle nested braces in interpolations:
     *      `result: ${obj.map(x => { return x * 2 })}`
     */
    function scanTemplateString(): void {
        const pos = position();
        let value = '';

        advance(); // opening backtick

        while (!isAtEnd() && peek() !== '`') {
            if (peek() === '\n') {
                // WHY: Template strings can span multiple lines (unlike STRING)
                line++;
                lineStart = current + 1;
                value += advance();
            } else if (peek() === '\\') {
                // WHY: Preserve escapes in template for later processing
                advance();
                value += advance();
            } else if (peek() === '$' && peek(1) === '{') {
                // WHY: Capture interpolation syntax including nested braces
                value += advance(); // $
                value += advance(); // {
                let braceDepth = 1;

                while (!isAtEnd() && braceDepth > 0) {
                    if (peek() === '{') {
                        braceDepth++;
                    }

                    if (peek() === '}') {
                        braceDepth--;
                    }

                    value += advance();
                }
            } else {
                value += advance();
            }
        }

        if (isAtEnd()) {
            addError('Unterminated template string', pos);

            return;
        }

        advance(); // closing backtick
        addToken('TEMPLATE_STRING', value, pos);
    }

    // =============================================================================
    // IDENTIFIER AND KEYWORD SCANNING
    // =============================================================================

    /**
     * Scan an identifier or keyword.
     *
     * PATTERN: alpha alphanumeric*
     *
     * WHY: Delegates keyword detection to the lexicon module, which maintains
     *      the canonical list of Latin keywords. This keeps the tokenizer
     *      independent of language evolution.
     *
     * WHY: Keywords are case-insensitive in Latin (classical Latin had no
     *      case distinction). 'si', 'Si', and 'SI' all match the keyword.
     */
    function scanIdentifier(): void {
        const pos = position();
        let value = '';

        while (isAlphaNumeric(peek())) {
            value += advance();
        }

        // WHY: Check lexicon for keyword match
        if (isKeyword(value)) {
            const kw = getKeyword(value)!;

            addToken('KEYWORD', value, pos, kw.latin);
        } else {
            addToken('IDENTIFIER', value, pos);
        }
    }

    // =============================================================================
    // MAIN SCANNER DISPATCH
    // =============================================================================

    /**
     * Scan a single token from current position.
     *
     * DESIGN: Uses longest-match strategy for operators. For example, '=='
     *         takes precedence over '=' when both match.
     *
     * ERROR RECOVERY: On unexpected character, emits error and advances to
     *                 next character to avoid infinite loop.
     */
    function scanToken(): void {
        skipWhitespace();
        if (isAtEnd()) {
            return;
        }

        const pos = position();
        const char = peek();

        // WHY: Numbers can't start with letter, so check digits first
        if (isDigit(char)) {
            scanNumber();

            return;
        }

        // WHY: Identifiers and keywords both start with alpha
        if (isAlpha(char)) {
            scanIdentifier();

            return;
        }

        // WHY: String literals can use ' or "
        if (char === '"' || char === "'") {
            scanString(char);

            return;
        }

        // WHY: Template strings use backticks
        if (char === '`') {
            scanTemplateString();

            return;
        }

        // ---------------------------------------------------------------------------
        // Operators and Delimiters
        // ---------------------------------------------------------------------------

        // WHY: Advance first, then lookahead for multi-character operators
        advance();
        switch (char) {
            // Single-character delimiters
            case '(':
                addToken('LPAREN', char, pos);
                break;
            case ')':
                addToken('RPAREN', char, pos);
                break;
            case '{':
                addToken('LBRACE', char, pos);
                break;
            case '}':
                addToken('RBRACE', char, pos);
                break;
            case '[':
                addToken('LBRACKET', char, pos);
                break;
            case ']':
                addToken('RBRACKET', char, pos);
                break;
            case ',':
                addToken('COMMA', char, pos);
                break;
            case ';':
                addToken('SEMICOLON', char, pos);
                break;
            // WHY: . can be .. (range) or . (member access)
            case '.':
                if (peek() === '.') {
                    advance();
                    addToken('DOT_DOT', '..', pos);
                } else {
                    addToken('DOT', char, pos);
                }

                break;
            case ':':
                addToken('COLON', char, pos);
                break;
            case '?':
                addToken('QUESTION', char, pos);
                break;

            // WHY: | can be || (logical OR) or | (union type operator)
            case '|':
                if (peek() === '|') {
                    advance();
                    addToken('OR', '||', pos);
                } else {
                    addToken('PIPE', char, pos);
                }

                break;

            // WHY: & must be && (logical AND), single & is reserved/invalid
            case '&':
                if (peek() === '&') {
                    advance();
                    addToken('AND', '&&', pos);
                } else {
                    addError(`Unexpected character '${char}'`, pos);
                }

                break;

            // Single-character arithmetic operators
            case '+':
                addToken('PLUS', char, pos);
                break;

            // WHY: - can be -> (type arrow) or - (minus/subtract)
            case '-':
                if (peek() === '>') {
                    advance();
                    addToken('THIN_ARROW', '->', pos);
                } else {
                    addToken('MINUS', char, pos);
                }

                break;

            case '*':
                addToken('STAR', char, pos);
                break;

            // WHY: / handled separately - already consumed by comment scanner
            case '/':
                addToken('SLASH', char, pos);
                break;

            case '%':
                addToken('PERCENT', char, pos);
                break;

            // WHY: = can be == (equality), => (fat arrow), or = (assignment)
            case '=':
                if (peek() === '=') {
                    advance();
                    addToken('EQUAL_EQUAL', '==', pos);
                } else if (peek() === '>') {
                    advance();
                    addToken('ARROW', '=>', pos);
                } else {
                    addToken('EQUAL', char, pos);
                }

                break;

            // WHY: ! can be != (not equal) or ! (logical not)
            case '!':
                if (peek() === '=') {
                    advance();
                    addToken('BANG_EQUAL', '!=', pos);
                } else {
                    addToken('BANG', char, pos);
                }

                break;

            // WHY: < can be <= (less or equal) or < (less than)
            case '<':
                if (peek() === '=') {
                    advance();
                    addToken('LESS_EQUAL', '<=', pos);
                } else {
                    addToken('LESS', char, pos);
                }

                break;

            // WHY: > can be >= (greater or equal) or > (greater than)
            case '>':
                if (peek() === '=') {
                    advance();
                    addToken('GREATER_EQUAL', '>=', pos);
                } else {
                    addToken('GREATER', char, pos);
                }

                break;

            // EDGE: Unrecognized character - emit error but continue
            default:
                addError(`Unexpected character '${char}'`, pos);
        }
    }

    // =============================================================================
    // MAIN TOKENIZATION LOOP
    // =============================================================================

    // WHY: Continue scanning until end of input, collecting all errors
    while (!isAtEnd()) {
        scanToken();
    }

    // WHY: EOF sentinel simplifies parser lookahead (no null checks needed)
    addToken('EOF', '', position());

    return { tokens, errors };
}

export * from './types';
