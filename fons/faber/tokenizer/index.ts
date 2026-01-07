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
 * - Single-line comments (# only)
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
 * '=' -> '===' | '==' | '=>' | '='
 * '!' -> '!==' | '!=' | '!'
 * '<' -> '<=' | '<'
 * '>' -> '>=' | '>'
 * '&' -> '&&' | '&'
 * '|' -> '||' | '|'
 * '-' -> '->' | '-'
 * '#' -> comment (to end of line)
 * '/' -> '/'
 * '^' -> '^'
 * '~' -> '~'
 *
 * @module tokenizer
 */

import type { Token, TokenType, Position, TokenizerResult, TokenizerError, CommentTokenType } from './types';
import { isKeyword, getKeyword } from '../lexicon';
import { TokenizerErrorCode, TOKENIZER_ERRORS } from './errors';

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
        // EDGE: Callers must check isAtEnd() before calling advance()
        const char = source[current]!;

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
     * Check if character is a hexadecimal digit (0-9, a-f, A-F).
     */
    function isHexDigit(char: string): boolean {
        return isDigit(char) || (char >= 'a' && char <= 'f') || (char >= 'A' && char <= 'F');
    }

    /**
     * Check if character is a binary digit (0-1).
     */
    function isBinaryDigit(char: string): boolean {
        return char === '0' || char === '1';
    }

    /**
     * Check if character is an octal digit (0-7).
     */
    function isOctalDigit(char: string): boolean {
        return char >= '0' && char <= '7';
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
     * Scan a number literal (integer, decimal, hex, binary, octal, scientific, or bigint).
     *
     * GRAMMAR: number = hex_literal | binary_literal | octal_literal | decimal_literal
     *          hex_literal = '0' ('x' | 'X') hex_digit+ 'n'?
     *          binary_literal = '0' ('b' | 'B') binary_digit+ 'n'?
     *          octal_literal = '0' ('o' | 'O') octal_digit+ 'n'?
     *          decimal_literal = digit+ ('.' digit+)? exponent? 'n'?
     *          exponent = ('e' | 'E') ('+' | '-')? digit+
     *
     * WHY: Numbers must be tokenized as a unit to distinguish from
     *      member access: 123.toString() vs 123.45
     */
    function scanNumber(): void {
        const pos = position();
        let value = '';

        // Check for hex prefix: 0x or 0X
        if (peek() === '0' && (peek(1) === 'x' || peek(1) === 'X')) {
            value += advance(); // 0
            value += advance(); // x or X

            // Must have at least one hex digit
            if (!isHexDigit(peek())) {
                addError(TokenizerErrorCode.InvalidHexLiteral, pos);
                return;
            }

            while (isHexDigit(peek())) {
                value += advance();
            }

            // Check for bigint suffix 'n'
            if (peek() === 'n') {
                advance();
                addToken('BIGINT', value, pos);
                return;
            }

            addToken('NUMBER', value, pos);
            return;
        }

        // Check for binary prefix: 0b or 0B
        if (peek() === '0' && (peek(1) === 'b' || peek(1) === 'B')) {
            value += advance(); // 0
            value += advance(); // b or B

            // Must have at least one binary digit
            if (!isBinaryDigit(peek())) {
                addError(TokenizerErrorCode.InvalidBinaryLiteral, pos);
                return;
            }

            while (isBinaryDigit(peek())) {
                value += advance();
            }

            // Check for bigint suffix 'n'
            if (peek() === 'n') {
                advance();
                addToken('BIGINT', value, pos);
                return;
            }

            addToken('NUMBER', value, pos);
            return;
        }

        // Check for octal prefix: 0o or 0O
        if (peek() === '0' && (peek(1) === 'o' || peek(1) === 'O')) {
            value += advance(); // 0
            value += advance(); // o or O

            // Must have at least one octal digit
            if (!isOctalDigit(peek())) {
                addError(TokenizerErrorCode.InvalidOctalLiteral, pos);
                return;
            }

            while (isOctalDigit(peek())) {
                value += advance();
            }

            // Check for bigint suffix 'n'
            if (peek() === 'n') {
                advance();
                addToken('BIGINT', value, pos);
                return;
            }

            addToken('NUMBER', value, pos);
            return;
        }

        // Decimal integer part
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

        // Scientific notation exponent (e.g., 1.5e10, 1e-5, 2.5E+3)
        if (peek() === 'e' || peek() === 'E') {
            const next = peek(1);
            // Exponent must be followed by digit or sign then digit
            if (isDigit(next) || ((next === '+' || next === '-') && isDigit(peek(2)))) {
                value += advance(); // e or E

                // Optional sign
                if (peek() === '+' || peek() === '-') {
                    value += advance();
                }

                // Exponent digits
                while (isDigit(peek())) {
                    value += advance();
                }
            }
        }

        // Check for bigint suffix 'n' (must follow a valid number)
        if (peek() === 'n') {
            advance(); // consume 'n'
            addToken('BIGINT', value, pos);
            return;
        }

        addToken('NUMBER', value, pos);
    }

    /**
     * Record a lexical error without stopping tokenization.
     *
     * WHY: Collecting errors rather than throwing enables:
     *      - IDE error squiggles for all errors at once
     *      - Partial compilation when some sections are valid
     *      - Better batch error reporting
     *
     * @param code - Error code from TokenizerErrorCode enum
     * @param pos - Source position where error occurred
     */
    function addError(code: TokenizerErrorCode, pos: Position): void {
        const { text, help } = TOKENIZER_ERRORS[code];

        errors.push({ code, text, help, position: pos });
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
        // True while there's whitespace or comments to consume
        const hasWhitespaceOrComment = () => !isAtEnd();

        while (hasWhitespaceOrComment()) {
            const char = peek();

            if (char === ' ' || char === '\t' || char === '\r') {
                advance();
            } else if (char === '\n') {
                advance();
                line++;
                lineStart = current;
            } else if (char === '#') {
                // Single-line comment extends to end of line
                const pos = position();
                let comment = '';

                advance(); // #

                // True while there's more comment content on this line
                const hasCommentContent = () => !isAtEnd() && peek() !== '\n';

                while (hasCommentContent()) {
                    comment += advance();
                }

                // Emit COMMENT token for comment preservation
                addToken('COMMENT', comment.trim(), pos, undefined, 'line');
            } else {
                break;
            }
        }
    }

    // =============================================================================
    // TOKEN CREATION
    // =============================================================================

    /**
     * Add a token to the output stream.
     *
     * @param type - Token type discriminator
     * @param value - Original source text of token
     * @param pos - Source position where token started
     * @param keyword - For KEYWORD tokens, the specific Latin keyword
     * @param commentType - For COMMENT tokens, the comment style (line/block/doc)
     */
    function addToken(type: TokenType, value: string, pos: Position, keyword?: string, commentType?: CommentTokenType): void {
        tokens.push({ type, value, position: pos, keyword, commentType });
    }

    // =============================================================================
    // LITERAL SCANNING
    // =============================================================================

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

        // True while there's more string content to consume
        const hasStringContent = () => !isAtEnd() && peek() !== quote;

        while (hasStringContent()) {
            if (peek() === '\n') {
                addError(TokenizerErrorCode.UnterminatedString, pos);

                return;
            }

            if (peek() === '\\') {
                // WHY: Preserve escape sequences literally rather than interpreting them.
                // The target language runtime will interpret \n, \t, \uXXXX, etc.
                // This ensures escape sequences pass through to generated code unchanged.
                value += advance(); // backslash

                if (!isAtEnd() && peek() !== '\n') {
                    const escaped = peek();

                    // For \uXXXX, consume all 5 characters (u + 4 hex digits)
                    if (escaped === 'u') {
                        value += advance(); // u

                        for (let i = 0; i < 4 && isHexDigit(peek()); i++) {
                            value += advance();
                        }
                    } else {
                        value += advance(); // the escaped character
                    }
                }
            } else {
                value += advance();
            }
        }

        if (isAtEnd()) {
            addError(TokenizerErrorCode.UnterminatedString, pos);

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

        // True while there's more template content to consume
        const hasTemplateContent = () => !isAtEnd() && peek() !== '`';

        while (hasTemplateContent()) {
            if (peek() === '\n') {
                // Template strings can span multiple lines (unlike STRING)
                line++;
                lineStart = current + 1;
                value += advance();
            } else if (peek() === '\\') {
                // Preserve escapes in template for later processing
                advance();

                value += advance();
            } else if (peek() === '$' && peek(1) === '{') {
                // Capture interpolation syntax including nested braces
                value += advance(); // $
                value += advance(); // {

                let braceDepth = 1;

                // True while interpolation is not terminated
                const hasInterpolationContent = () => !isAtEnd() && braceDepth > 0;

                while (hasInterpolationContent()) {
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
            addError(TokenizerErrorCode.UnterminatedTemplateString, pos);

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

        // Check lexicon for keyword match
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

        // Operators and delimiters - advance first, then lookahead for multi-character operators
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

            // WHY: | can be || (logical OR), |= (compound), or | (bitwise OR)
            case '|':
                if (peek() === '|') {
                    advance();
                    addToken('OR', '||', pos);
                } else if (peek() === '=') {
                    advance();
                    addToken('PIPE_EQUAL', '|=', pos);
                } else {
                    addToken('PIPE', char, pos);
                }

                break;

            // WHY: & can be && (logical AND), &= (compound), or & (bitwise AND)
            case '&':
                if (peek() === '&') {
                    advance();
                    addToken('AND', '&&', pos);
                } else if (peek() === '=') {
                    advance();
                    addToken('AMPERSAND_EQUAL', '&=', pos);
                } else {
                    addToken('AMPERSAND', char, pos);
                }

                break;

            // Bitwise XOR
            case '^':
                addToken('CARET', char, pos);
                break;

            // Bitwise NOT (unary)
            case '~':
                addToken('TILDE', char, pos);
                break;

            // Annotation marker
            case '@':
                addToken('AT', char, pos);
                break;

            // Single-character arithmetic operators
            case '+':
                if (peek() === '=') {
                    advance();
                    addToken('PLUS_EQUAL', '+=', pos);
                } else {
                    addToken('PLUS', char, pos);
                }
                break;

            // WHY: - can be -> (type arrow), -= (compound), or - (minus/subtract)
            case '-':
                if (peek() === '>') {
                    advance();
                    addToken('THIN_ARROW', '->', pos);
                } else if (peek() === '=') {
                    advance();
                    addToken('MINUS_EQUAL', '-=', pos);
                } else {
                    addToken('MINUS', char, pos);
                }

                break;

            case '*':
                if (peek() === '=') {
                    advance();
                    addToken('STAR_EQUAL', '*=', pos);
                } else {
                    addToken('STAR', char, pos);
                }
                break;

            // WHY: / can be /= (compound) or / (divide)
            case '/':
                if (peek() === '=') {
                    advance();
                    addToken('SLASH_EQUAL', '/=', pos);
                } else {
                    addToken('SLASH', char, pos);
                }
                break;

            case '%':
                if (peek() === '=') {
                    advance();
                    addToken('PERCENT_EQUAL', '%=', pos);
                } else {
                    addToken('PERCENT', char, pos);
                }
                break;

            // WHY: = can be === (strict equality), == (equality), => (fat arrow), or = (assignment)
            case '=':
                if (peek() === '=' && peek(1) === '=') {
                    advance();
                    advance();
                    addToken('TRIPLE_EQUAL', '===', pos);
                } else if (peek() === '=') {
                    advance();
                    addToken('EQUAL_EQUAL', '==', pos);
                } else {
                    addToken('EQUAL', char, pos);
                }

                break;

            // WHY: ! can be !== (strict not equal), != (not equal), or ! (logical not)
            case '!':
                if (peek() === '=' && peek(1) === '=') {
                    advance();
                    advance();
                    addToken('BANG_DOUBLE_EQUAL', '!==', pos);
                } else if (peek() === '=') {
                    advance();
                    addToken('BANG_EQUAL', '!=', pos);
                } else {
                    addToken('BANG', char, pos);
                }

                break;

            // WHY: < can be <= (less or equal) or < (less than)
            // NOTE: << removed - use sinistratum keyword for left shift
            case '<':
                if (peek() === '=') {
                    advance();
                    addToken('LESS_EQUAL', '<=', pos);
                }
                else {
                    addToken('LESS', char, pos);
                }

                break;

            // WHY: > can be >= (greater or equal) or > (greater than)
            // NOTE: >> removed - use dextratum keyword for right shift
            case '>':
                if (peek() === '=') {
                    advance();
                    addToken('GREATER_EQUAL', '>=', pos);
                }
                else {
                    addToken('GREATER', char, pos);
                }

                break;

            // EDGE: Unrecognized character - emit error but continue
            default:
                addError(TokenizerErrorCode.UnexpectedCharacter, pos);
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
export * from './errors';
