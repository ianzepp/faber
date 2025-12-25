/**
 * Parser - Recursive Descent Parser for Latin Source Code
 *
 * COMPILER PHASE
 * ==============
 * syntactic
 *
 * ARCHITECTURE
 * ============
 * This module implements a recursive descent parser that transforms a stream of tokens
 * from the lexical analyzer into an Abstract Syntax Tree (AST). The parser uses
 * predictive parsing with one token of lookahead to determine which production to use.
 *
 * The parser is organized around the grammar's structure:
 * - Statement parsing functions handle declarations and control flow
 * - Expression parsing uses precedence climbing to handle operator precedence
 * - Error recovery via synchronization prevents cascading errors
 *
 * Key design decisions:
 * 1. Never throws exceptions - collects errors and continues parsing
 * 2. Synchronizes at statement boundaries after errors
 * 3. Uses helper functions (peek, match, expect) for token manipulation
 * 4. Preserves Latin keywords in AST for semantic analysis phase
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Token[] from tokenizer (includes position info for error reporting)
 * OUTPUT: ParserResult containing Program AST and array of ParserErrors
 * ERRORS: Syntax errors (unexpected tokens, missing punctuation, malformed constructs)
 *
 * INVARIANTS
 * ==========
 * INV-1: Parser never crashes on malformed input (errors collected, not thrown)
 * INV-2: After error, synchronize() finds next statement boundary
 * INV-3: All AST nodes include position for error reporting
 * INV-4: Empty input produces valid Program with empty body
 * INV-5: Parser maintains single token lookahead (peek() without consuming)
 *
 * GRAMMAR
 * =======
 * High-level grammar in EBNF (detailed rules in function comments):
 *
 *   program        := statement*
 *   statement      := importDecl | varDecl | funcDecl | ifStmt | whileStmt | forStmt
 *                   | returnStmt | throwStmt | tryStmt | blockStmt | exprStmt
 *   expression     := assignment
 *   assignment     := or ('=' assignment)?
 *   or             := and ('||' and)*
 *   and            := equality ('&&' equality)*
 *   equality       := comparison (('==' | '!=') comparison)*
 *   comparison     := additive (('<' | '>' | '<=' | '>=') additive)*
 *   additive       := multiplicative (('+' | '-') multiplicative)*
 *   multiplicative := unary (('*' | '/' | '%') unary)*
 *   unary          := ('!' | '-' | 'non' | 'cede' | 'novum') unary | call
 *   call           := primary ('(' args ')' | '.' IDENTIFIER | '[' expr ']')*
 *   primary        := IDENTIFIER | NUMBER | STRING | TEMPLATE_STRING
 *                   | 'verum' | 'falsum' | 'nihil' | '(' expression ')'
 *
 * ERROR RECOVERY STRATEGY
 * =======================
 * When a parse error occurs:
 * 1. Record error with message and position
 * 2. Call synchronize() to skip tokens until statement boundary
 * 3. Resume parsing at next statement
 * 4. Return partial AST with collected errors
 *
 * Synchronization points (keywords that start statements):
 * - functio, varia, fixum (declarations)
 * - si, dum, pro (control flow)
 * - redde, tempta (other statements)
 *
 * WHY: Allows parser to report multiple errors in one pass and produce
 *      partial AST for IDE/tooling use even with syntax errors.
 *
 * @module parser
 */

import type { Token, TokenType, Position } from '../tokenizer/types';
import type {
    Program,
    Statement,
    Expression,
    ImportDeclaration,
    VariableDeclaration,
    FunctionDeclaration,
    GenusDeclaration,
    FieldDeclaration,
    PactumDeclaration,
    PactumMethod,
    IfStatement,
    WhileStatement,
    ForStatement,
    WithStatement,
    SwitchStatement,
    SwitchCase,
    GuardStatement,
    GuardClause,
    AssertStatement,
    ReturnStatement,
    BreakStatement,
    ContinueStatement,
    BlockStatement,
    ThrowStatement,
    ScribeStatement,
    OutputLevel,
    ExpressionStatement,
    Identifier,
    ThisExpression,
    ArrowFunctionExpression,
    Parameter,
    TypeAnnotation,
    TypeParameter,
    CatchClause,
    NewExpression,
    TypeAliasDeclaration,
    EnumDeclaration,
    EnumMember,
    Literal,
    RangeExpression,
    ConditionalExpression,
    ObjectPattern,
    ObjectPatternProperty,
    ObjectExpression,
    ObjectProperty,
    SpreadElement,
    FacBlockStatement,
    LambdaExpression,
    TypeCastExpression,
} from './ast';
import { builtinTypes } from '../lexicon/types-builtin';
import { ParserErrorCode, PARSER_ERRORS } from './errors';

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

/**
 * Parser output containing AST and errors.
 *
 * INVARIANT: If parse succeeds, program is non-null and errors is empty.
 * INVARIANT: If parse fails catastrophically, program is null.
 * INVARIANT: Partial errors (recovered) have non-null program with non-empty errors.
 */
export interface ParserResult {
    program: Program | null;
    errors: ParserError[];
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
const BUILTIN_TYPE_NAMES = new Set(builtinTypes.map(t => t.nominative ?? computeNominative(t.stem, t.declension, t.gender)));

// =============================================================================
// MAIN PARSER FUNCTION
// =============================================================================

/**
 * Parse a token stream into an Abstract Syntax Tree.
 *
 * GRAMMAR:
 *   program := statement*
 *
 * ERROR RECOVERY: Catches errors at statement level, synchronizes, and continues.
 *
 * @param tokens - Token array from tokenizer (must end with EOF token)
 * @returns ParserResult with program AST and error list
 */
export function parse(tokens: Token[]): ParserResult {
    const errors: ParserError[] = [];
    let current = 0;

    // ---------------------------------------------------------------------------
    // Token Navigation Helpers
    // ---------------------------------------------------------------------------

    /**
     * Look ahead at token without consuming.
     *
     * INVARIANT: Returns EOF token if offset goes beyond end.
     */
    function peek(offset = 0): Token {
        return tokens[current + offset] ?? tokens[tokens.length - 1];
    }

    /**
     * Check if at end of token stream.
     */
    function isAtEnd(): boolean {
        return peek().type === 'EOF';
    }

    /**
     * Consume and return current token.
     *
     * INVARIANT: Never advances past EOF.
     */
    function advance(): Token {
        if (!isAtEnd()) {
            current++;
        }

        return tokens[current - 1];
    }

    /**
     * Check if current token matches type without consuming.
     */
    function check(type: TokenType): boolean {
        return peek().type === type;
    }

    /**
     * Check if current token is specific keyword without consuming.
     *
     * WHY: Latin keywords are stored in token.keyword field, not token.type.
     */
    function checkKeyword(keyword: string): boolean {
        return peek().type === 'KEYWORD' && peek().keyword === keyword;
    }

    /**
     * Match and consume token if type matches.
     *
     * @returns true if matched and consumed, false otherwise
     */
    function match(...types: TokenType[]): boolean {
        for (const type of types) {
            if (check(type)) {
                advance();

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
    function matchKeyword(keyword: string): boolean {
        if (checkKeyword(keyword)) {
            advance();

            return true;
        }

        return false;
    }

    /**
     * Report error using error catalog.
     *
     * WHY: Centralizes error creation with consistent structure.
     *
     * @param code - Error code from ParserErrorCode enum
     * @param context - Optional context to append (e.g., "got 'x'")
     */
    function reportError(code: ParserErrorCode, context?: string): void {
        const token = peek();
        const { text } = PARSER_ERRORS[code];
        const message = context ? `${text}, ${context}` : text;

        errors.push({ code, message, position: token.position });
    }

    /**
     * Expect specific token type or record error.
     *
     * ERROR RECOVERY: Records error but returns current token (possibly wrong type).
     *
     * @returns Matched token if found, current token if not
     */
    function expect(type: TokenType, code: ParserErrorCode): Token {
        if (check(type)) {
            return advance();
        }

        const token = peek();

        reportError(code, `got '${token.value}'`);

        return token;
    }

    /**
     * Expect specific keyword or record error.
     *
     * ERROR RECOVERY: Records error but returns current token.
     */
    function expectKeyword(keyword: string, code: ParserErrorCode): Token {
        if (checkKeyword(keyword)) {
            return advance();
        }

        const token = peek();

        reportError(code, `got '${token.value}'`);

        return token;
    }

    /**
     * Record error and throw for error recovery.
     *
     * WHY: Used in expression parsing where we can't easily recover locally.
     *      Caught by statement parser which calls synchronize().
     */
    function error(code: ParserErrorCode, context?: string): never {
        reportError(code, context);
        throw new Error(PARSER_ERRORS[code].text);
    }

    // ---------------------------------------------------------------------------
    // Type Name Helpers
    // ---------------------------------------------------------------------------

    /**
     * Check if token is a builtin type name.
     *
     * WHY: Type-first syntax requires distinguishing type names from identifiers.
     *      "fixum textus nomen" (type-first) vs "fixum nomen" (type inference).
     *
     * @returns true if token is an identifier and a known builtin type
     */
    function isTypeName(token: Token): boolean {
        // Case-insensitive lookup (textus, textus, TEXTUS all match)
        return token.type === 'IDENTIFIER' && BUILTIN_TYPE_NAMES.has(token.value.toLowerCase());
    }

    /**
     * Check if token is a preposition used in parameters.
     *
     * WHY: Prepositions indicate semantic roles (ad, de, in, ex).
     *
     * @returns true if token is a preposition keyword
     */
    function isPreposition(token: Token): boolean {
        return token.type === 'KEYWORD' && ['ad', 'de', 'in', 'ex'].includes(token.keyword ?? '');
    }

    // =============================================================================
    // STATEMENT PARSING
    // =============================================================================

    /**
     * Parse top-level program.
     *
     * GRAMMAR:
     *   program := statement*
     *
     * ERROR RECOVERY: Catches statement errors, synchronizes, continues.
     *
     * EDGE: Empty source file produces valid Program with empty body.
     */
    function parseProgram(): Program {
        const body: Statement[] = [];
        const position = peek().position;

        while (!isAtEnd()) {
            // Consume optional semicolons between statements
            while (match('SEMICOLON')) {
                // do nothing - avoids linter no-empty
            }

            if (isAtEnd()) {
                break;
            }

            try {
                body.push(parseStatement());
            } catch {
                synchronize();
            }
        }

        return { type: 'Program', body, position };
    }

    /**
     * Synchronize parser after error by skipping to next statement boundary.
     *
     * ERROR RECOVERY: Advances until finding a keyword that starts a statement.
     *
     * WHY: Prevents cascading errors by resuming at known-good parse state.
     */
    function synchronize(): void {
        advance();
        while (!isAtEnd()) {
            if (
                checkKeyword('functio') ||
                checkKeyword('varia') ||
                checkKeyword('fixum') ||
                checkKeyword('figendum') ||
                checkKeyword('variandum') ||
                checkKeyword('typus') ||
                checkKeyword('ordo') ||
                checkKeyword('si') ||
                checkKeyword('dum') ||
                checkKeyword('pro') ||
                checkKeyword('redde') ||
                checkKeyword('tempta')
            ) {
                return;
            }

            advance();
        }
    }

    /**
     * Synchronize parser after error in genus member by skipping to next member boundary.
     *
     * ERROR RECOVERY: Advances until finding a token that could start a genus member.
     *
     * WHY: Prevents infinite loops when malformed syntax (e.g., TS-style `fixum name: textus`)
     *      causes parseGenusMember to return without advancing.
     */
    function synchronizeGenusMember(): void {
        advance();
        while (!isAtEnd() && !check('RBRACE')) {
            // Stop at tokens that could start a new member
            if (
                checkKeyword('functio') ||
                checkKeyword('publicus') ||
                checkKeyword('privatus') ||
                checkKeyword('generis') ||
                checkKeyword('nexum') ||
                checkKeyword('futura') ||
                checkKeyword('cursor') ||
                // Type keywords that could start a field
                checkKeyword('textus') ||
                checkKeyword('numerus') ||
                checkKeyword('fractus') ||
                checkKeyword('verus') ||
                checkKeyword('nihil') ||
                checkKeyword('lista') ||
                checkKeyword('mappa') ||
                checkKeyword('vacuum') ||
                // Generic identifier could be a type name
                check('IDENTIFIER')
            ) {
                return;
            }

            advance();
        }
    }

    /**
     * Parse any statement by dispatching to specific parser.
     *
     * GRAMMAR:
     *   statement := importDecl | varDecl | funcDecl | typeAliasDecl | ifStmt | whileStmt | forStmt
     *              | returnStmt | throwStmt | tryStmt | blockStmt | exprStmt
     *
     * WHY: Uses lookahead to determine statement type via keyword inspection.
     */
    function parseStatement(): Statement {
        // Distinguish 'ex norma importa' (import), 'ex items pro n' (for-loop),
        // and 'ex response fixum { }' (destructuring)
        if (checkKeyword('ex')) {
            // Look ahead: ex (IDENTIFIER|STRING) importa -> import
            const nextType = peek(1).type;

            if ((nextType === 'IDENTIFIER' || nextType === 'STRING') && peek(2).keyword === 'importa') {
                return parseImportDeclaration();
            }

            // Could be for-loop or destructuring - parse and dispatch
            return parseExStatement();
        }

        // 'de' for for-in loops (iterate over keys)
        // de tabula pro k { } → for-in loop
        if (checkKeyword('de')) {
            return parseDeStatement();
        }

        // 'in' for mutation blocks
        // in user { } → with-block (mutation)
        if (checkKeyword('in')) {
            return parseInStatement();
        }

        if (checkKeyword('varia') || checkKeyword('fixum') || checkKeyword('figendum') || checkKeyword('variandum')) {
            return parseVariableDeclaration();
        }

        if (checkKeyword('functio') || checkKeyword('futura') || checkKeyword('cursor')) {
            return parseFunctionDeclaration();
        }

        if (checkKeyword('typus')) {
            return parseTypeAliasDeclaration();
        }

        if (checkKeyword('ordo')) {
            return parseEnumDeclaration();
        }

        if (checkKeyword('genus')) {
            return parseGenusDeclaration();
        }

        if (checkKeyword('pactum')) {
            return parsePactumDeclaration();
        }

        if (checkKeyword('si')) {
            return parseIfStatement();
        }

        if (checkKeyword('dum')) {
            return parseWhileStatement();
        }

        if (checkKeyword('elige')) {
            return parseSwitchStatement();
        }

        if (checkKeyword('custodi')) {
            return parseGuardStatement();
        }

        if (checkKeyword('adfirma')) {
            return parseAssertStatement();
        }

        if (checkKeyword('redde')) {
            return parseReturnStatement();
        }

        if (checkKeyword('rumpe')) {
            return parseBreakStatement();
        }

        if (checkKeyword('perge')) {
            return parseContinueStatement();
        }

        if (checkKeyword('iace')) {
            return parseThrowStatement(false);
        }

        if (checkKeyword('mori')) {
            return parseThrowStatement(true);
        }

        if (checkKeyword('scribe')) {
            return parseScribeStatement('log');
        }

        if (checkKeyword('vide')) {
            return parseScribeStatement('debug');
        }

        if (checkKeyword('mone')) {
            return parseScribeStatement('warn');
        }

        if (checkKeyword('tempta')) {
            return parseTryStatement();
        }

        // fac { } cape { } is block with optional catch (see parseFacBlockStatement)
        if (checkKeyword('fac') && peek(1).type === 'LBRACE') {
            return parseFacBlockStatement();
        }

        if (check('LBRACE')) {
            return parseBlockStatement();
        }

        return parseExpressionStatement();
    }

    // ---------------------------------------------------------------------------
    // Declaration Statements
    // ---------------------------------------------------------------------------

    /**
     * Parse import declaration.
     *
     * GRAMMAR:
     *   importDecl := 'ex' (STRING | IDENTIFIER) 'importa' (identifierList | '*')
     *   identifierList := IDENTIFIER (',' IDENTIFIER)*
     *
     * Examples:
     *   ex norma importa scribe, lege
     *   ex "norma/tempus" importa nunc, dormi
     *   ex norma importa *
     */
    function parseImportDeclaration(): ImportDeclaration {
        const position = peek().position;

        expectKeyword('ex', ParserErrorCode.ExpectedKeywordEx);

        // WHY: Accept both bare identifiers (ex norma) and strings (ex "norma/tempus")
        // String paths enable hierarchical module organization for stdlib
        let source: string;

        if (check('STRING')) {
            const sourceToken = advance();
            source = sourceToken.value;
        } else {
            const sourceToken = expect('IDENTIFIER', ParserErrorCode.ExpectedModuleName);
            source = sourceToken.value;
        }

        expectKeyword('importa', ParserErrorCode.ExpectedKeywordImporta);

        if (match('STAR')) {
            return { type: 'ImportDeclaration', source, specifiers: [], wildcard: true, position };
        }

        const specifiers: Identifier[] = [];

        do {
            specifiers.push(parseIdentifier());
        } while (match('COMMA'));

        return { type: 'ImportDeclaration', source, specifiers, wildcard: false, position };
    }

    /**
     * Parse variable declaration.
     *
     * GRAMMAR:
     *   varDecl := ('varia' | 'fixum') (typeAnnotation IDENTIFIER | IDENTIFIER) ('=' expression)?
     *
     * WHY: Type-first syntax: "fixum textus nomen = value" or "fixum nomen = value"
     *      Latin 'varia' (let it be) for mutable, 'fixum' (fixed) for immutable.
     *
     * EDGE: If next token after varia/fixum is a type name, parse type first.
     *       Otherwise, parse identifier (type inference case).
     *
     * Async bindings (figendum/variandum) work the same way syntactically,
     * but imply await on the initializer during codegen.
     */
    function parseVariableDeclaration(): VariableDeclaration {
        const position = peek().position;
        const kind = peek().keyword as 'varia' | 'fixum' | 'figendum' | 'variandum';

        advance(); // varia, fixum, figendum, or variandum

        let typeAnnotation: TypeAnnotation | undefined;
        let name: Identifier | ObjectPattern;

        // Check for object destructuring pattern: fixum { a, b } = obj
        if (check('LBRACE')) {
            name = parseObjectPattern();
        } else if (isTypeName(peek())) {
            typeAnnotation = parseTypeAnnotation();
            name = parseIdentifier();
        } else {
            name = parseIdentifier();
        }

        let init: Expression | undefined;

        if (match('EQUAL')) {
            init = parseExpression();
        }

        return { type: 'VariableDeclaration', kind, name, typeAnnotation, init, position };
    }

    /**
     * Parse object destructuring pattern.
     *
     * GRAMMAR:
     *   objectPattern := '{' patternProperty (',' patternProperty)* '}'
     *   patternProperty := IDENTIFIER (':' IDENTIFIER)?
     *
     * Examples:
     *   { nomen, aetas }
     *   { nomen: localName, aetas }
     */
    function parseObjectPattern(): ObjectPattern {
        const position = peek().position;

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const properties: ObjectPatternProperty[] = [];

        // True while there are unparsed properties (not at '}' or EOF)
        const hasMoreProperties = () => !check('RBRACE') && !isAtEnd();

        while (hasMoreProperties()) {
            const propPosition = peek().position;

            // Check for rest pattern: ceteri restName
            let rest = false;
            if (checkKeyword('ceteri')) {
                advance(); // consume 'ceteri'
                rest = true;
            }

            const key = parseIdentifier();

            let value = key;

            // Check for rename: { nomen: localName } (not valid with ceteri)
            if (match('COLON') && !rest) {
                value = parseIdentifier();
            }

            properties.push({
                type: 'ObjectPatternProperty',
                key,
                value,
                rest,
                position: propPosition,
            });

            if (!check('RBRACE')) {
                expect('COMMA', ParserErrorCode.ExpectedComma);
            }
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return { type: 'ObjectPattern', properties, position };
    }

    /**
     * Parse function declaration.
     *
     * GRAMMAR:
     *   funcDecl := ('futura' | 'cursor')* 'functio' IDENTIFIER '(' paramList ')' returnClause? blockStmt
     *   returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
     *
     * WHY: Arrow syntax for return types: "functio greet(textus name) -> textus"
     *      'futura' prefix marks async functions (future/promise-based).
     *      'cursor' prefix marks generator functions (yield-based).
     *
     * RETURN TYPE VERBS: Latin verb forms encode async/generator semantics directly:
     *      '->'    neutral arrow (semantics from prefix only)
     *      'fit'   "it becomes" - sync, returns single value
     *      'fiet'  "it will become" - async, returns Promise<T>
     *      'fiunt' "they become" - sync generator, yields multiple values
     *      'fient' "they will become" - async generator, yields Promise values
     *
     * When using verb forms, the futura/cursor prefix is NOT required - the verb
     * itself carries the semantic information. The prefix becomes redundant:
     *      functio compute() -> numerus { ... }    // arrow: sync by default
     *      functio compute() fit numerus { ... }   // verb: explicitly sync
     *      functio fetch() fiet textus { ... }     // verb implies async (no futura needed)
     *      functio items() fiunt numerus { ... }   // verb implies generator (no cursor needed)
     *      functio stream() fient datum { ... }    // verb implies async generator
     *
     * Prefix is still allowed for emphasis, but verb/prefix conflicts are errors.
     */
    function parseFunctionDeclaration(): FunctionDeclaration {
        const position = peek().position;

        // Parse optional prefixes (futura = async, cursor = generator)
        let prefixAsync = false;
        let prefixGenerator = false;

        while (checkKeyword('futura') || checkKeyword('cursor')) {
            if (matchKeyword('futura')) {
                prefixAsync = true;
            } else if (matchKeyword('cursor')) {
                prefixGenerator = true;
            }
        }

        expectKeyword('functio', ParserErrorCode.ExpectedKeywordFunctio);

        const name = parseIdentifier();

        expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);

        const params = parseParameterList();

        expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

        let returnType: TypeAnnotation | undefined;
        let verbAsync: boolean | undefined;
        let verbGenerator: boolean | undefined;

        // Parse return type with arrow or verb form
        // Verb forms: fit (sync, single), fiet (async, single), fiunt (sync, generator), fient (async, generator)
        if (match('THIN_ARROW')) {
            returnType = parseTypeAnnotation();
            // Arrow is neutral - semantics from prefix only
        } else if (matchKeyword('fit')) {
            returnType = parseTypeAnnotation();
            verbAsync = false;
            verbGenerator = false;
        } else if (matchKeyword('fiet')) {
            returnType = parseTypeAnnotation();
            verbAsync = true;
            verbGenerator = false;
        } else if (matchKeyword('fiunt')) {
            returnType = parseTypeAnnotation();
            verbAsync = false;
            verbGenerator = true;
        } else if (matchKeyword('fient')) {
            returnType = parseTypeAnnotation();
            verbAsync = true;
            verbGenerator = true;
        }

        // Validate prefix/verb agreement
        // If verb specifies sync (fit/fiunt) but prefix says async (futura) = error
        // If verb specifies single (fit/fiet) but prefix says generator (cursor) = error
        if (verbAsync === false && prefixAsync) {
            errors.push({
                code: ParserErrorCode.GenericError,
                message: 'fit/fiunt (sync) contradicts futura (async)',
                position,
            });
        }
        if (verbGenerator === false && prefixGenerator) {
            errors.push({
                code: ParserErrorCode.GenericError,
                message: 'fit/fiet (single) contradicts cursor (generator)',
                position,
            });
        }

        // Merge semantics: verb takes precedence if specified, otherwise use prefix
        const async = verbAsync ?? prefixAsync;
        const generator = verbGenerator ?? prefixGenerator;

        const body = parseBlockStatement();

        return {
            type: 'FunctionDeclaration',
            name,
            params,
            returnType,
            body,
            async,
            generator,
            position,
        };
    }

    /**
     * Parse function parameter list.
     *
     * GRAMMAR:
     *   paramList := (parameter (',' parameter)*)?
     */
    function parseParameterList(): Parameter[] {
        const params: Parameter[] = [];

        if (check('RPAREN')) {
            return params;
        }

        do {
            params.push(parseParameter());
        } while (match('COMMA'));

        return params;
    }

    /**
     * Parse single function parameter.
     *
     * GRAMMAR:
     *   parameter := ('ad' | 'de' | 'in' | 'ex')? (typeAnnotation IDENTIFIER | IDENTIFIER)
     *
     * WHY: Type-first syntax: "textus name" or "ad textus recipientem"
     *      Prepositional prefixes indicate semantic roles:
     *      ad = toward/to, de = from/concerning (borrowed),
     *      in = in/into (mutable), ex = from/out of
     *
     * EDGE: Preposition comes first (if present), then type (if present), then identifier.
     *
     * TYPE DETECTION: Uses lookahead to detect type annotations for user-defined types.
     *   - Builtin type names (textus, numerus, etc.) are recognized directly
     *   - IDENT IDENT pattern: first is type, second is name (e.g., "coordinate point")
     *   - IDENT< pattern: generic type (e.g., "lista<textus>")
     */
    function parseParameter(): Parameter {
        const position = peek().position;

        let preposition: string | undefined;

        if (isPreposition(peek())) {
            preposition = advance().keyword;
        }

        // Check for rest parameter: ceteri [type] name
        let rest = false;
        if (checkKeyword('ceteri')) {
            advance(); // consume 'ceteri'
            rest = true;
        }

        let typeAnnotation: TypeAnnotation | undefined;

        // WHY: Use lookahead to detect user-defined types, not just builtins.
        // If we see IDENT followed by IDENT, first is type, second is name.
        // If we see IDENT followed by <, it's a generic type.
        const hasTypeAnnotation =
            isTypeName(peek()) || (check('IDENTIFIER') && peek(1).type === 'IDENTIFIER') || (check('IDENTIFIER') && peek(1).type === 'LESS');

        if (hasTypeAnnotation) {
            typeAnnotation = parseTypeAnnotation();
        }

        const name = parseIdentifier();

        return { type: 'Parameter', name, typeAnnotation, preposition, rest, position };
    }

    /**
     * Parse type alias declaration.
     *
     * GRAMMAR:
     *   typeAliasDecl := 'typus' IDENTIFIER '=' typeAnnotation
     *
     * WHY: Enables creating named type aliases for complex types.
     *
     * Examples:
     *   typus ID = textus
     *   typus UserID = numerus<32, Naturalis>
     */
    function parseTypeAliasDeclaration(): TypeAliasDeclaration {
        const position = peek().position;

        expectKeyword('typus', ParserErrorCode.ExpectedKeywordTypus);

        const name = parseIdentifier();

        expect('EQUAL', ParserErrorCode.ExpectedEqual);

        const typeAnnotation = parseTypeAnnotation();

        return { type: 'TypeAliasDeclaration', name, typeAnnotation, position };
    }

    // ---------------------------------------------------------------------------
    // Enum Declarations
    // ---------------------------------------------------------------------------

    /**
     * Parse enum declaration.
     *
     * GRAMMAR:
     *   enumDecl := 'ordo' IDENTIFIER '{' enumMember (',' enumMember)* ','? '}'
     *   enumMember := IDENTIFIER ('=' (NUMBER | STRING))?
     *
     * WHY: Latin 'ordo' (order/rank) for enumerated constants.
     *
     * Examples:
     *   ordo color { rubrum, viridis, caeruleum }
     *   ordo status { pendens = 0, actum = 1, finitum = 2 }
     */
    function parseEnumDeclaration(): EnumDeclaration {
        const position = peek().position;

        expectKeyword('ordo', ParserErrorCode.ExpectedKeywordOrdo);

        const name = parseIdentifier();

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const members: EnumMember[] = [];

        while (!check('RBRACE') && !isAtEnd()) {
            const memberPosition = peek().position;
            const memberName = parseIdentifier();

            let value: Literal | undefined;

            if (match('EQUAL')) {
                // Expect a literal value (number or string)
                const valueTok = advance();

                if (valueTok.type === 'NUMBER') {
                    value = {
                        type: 'Literal',
                        value: Number(valueTok.value),
                        position: valueTok.position,
                    };
                } else if (valueTok.type === 'STRING') {
                    value = {
                        type: 'Literal',
                        value: valueTok.value,
                        position: valueTok.position,
                    };
                } else {
                    errors.push({
                        code: ParserErrorCode.UnexpectedToken,
                        message: `Expected number or string for enum value, got ${valueTok.type}`,
                        position: valueTok.position,
                    });
                }
            }

            members.push({
                type: 'EnumMember',
                name: memberName,
                value,
                position: memberPosition,
            });

            // Allow trailing comma
            if (!check('RBRACE')) {
                match('COMMA');
            }
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return { type: 'EnumDeclaration', name, members, position };
    }

    // ---------------------------------------------------------------------------
    // Genus (Struct) Declarations
    // ---------------------------------------------------------------------------

    /**
     * Parse genus (struct) declaration.
     *
     * GRAMMAR:
     *   genusDecl := 'genus' IDENTIFIER typeParams? ('implet' IDENTIFIER (',' IDENTIFIER)*)? '{' genusMember* '}'
     *   typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
     *   genusMember := fieldDecl | methodDecl
     *
     * WHY: Latin 'genus' (kind/type) for data structures.
     *      'implet' (fulfills) for implementing pactum interfaces.
     */
    function parseGenusDeclaration(): GenusDeclaration {
        const position = peek().position;

        expectKeyword('genus', ParserErrorCode.ExpectedKeywordGenus);

        const name = parseIdentifier();

        // Parse optional type parameters <T, U>
        let typeParameters: Identifier[] | undefined;

        if (match('LESS')) {
            typeParameters = [];

            do {
                typeParameters.push(parseIdentifier());
            } while (match('COMMA'));

            expect('GREATER', ParserErrorCode.ExpectedClosingAngle);
        }

        // Parse optional 'implet' clause
        let implementsList: Identifier[] | undefined;

        if (matchKeyword('implet')) {
            implementsList = [];

            do {
                implementsList.push(parseIdentifier());
            } while (match('COMMA'));
        }

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const fields: FieldDeclaration[] = [];
        const methods: FunctionDeclaration[] = [];
        let constructorMethod: FunctionDeclaration | undefined;

        // True while there are unparsed members (not at '}' or EOF)
        const hasMoreMembers = () => !check('RBRACE') && !isAtEnd();

        while (hasMoreMembers()) {
            const startPosition = current;
            const member = parseGenusMember();

            // EDGE: If parser didn't advance, synchronize to avoid infinite loop.
            // This happens with malformed syntax like `fixum name: textus` (TS-style).
            if (current === startPosition) {
                synchronizeGenusMember();
                continue;
            }

            switch (member.type) {
                case 'FieldDeclaration':
                    fields.push(member);
                    break;

                case 'FunctionDeclaration':
                    if (member.isConstructor) {
                        constructorMethod = member;
                    } else {
                        methods.push(member);
                    }
                    break;

                default: {
                    const _exhaustive: never = member;
                    throw new Error(`Unknown genus member type: ${(_exhaustive as any).type}`);
                }
            }
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return {
            type: 'GenusDeclaration',
            name,
            typeParameters,
            implements: implementsList,
            fields,
            constructor: constructorMethod,
            methods,
            position,
        };
    }

    /**
     * Parse a member of a genus (field or method).
     *
     * GRAMMAR:
     *   genusMember := fieldDecl | methodDecl
     *   fieldDecl := 'privatus'? 'generis'? typeAnnotation IDENTIFIER (':' expression)?
     *   methodDecl := 'privatus'? 'generis'? ('futura' | 'cursor')* 'functio' ...
     *
     * WHY: Distinguishes between fields and methods by looking for 'functio' keyword.
     * WHY: Fields are public by default (struct semantics), use 'privatus' for private.
     */
    function parseGenusMember(): FieldDeclaration | FunctionDeclaration {
        const position = peek().position;

        // Parse modifiers
        let isPrivate = false;
        let isStatic = false;
        let isReactive = false;
        let prefixAsync = false;
        let prefixGenerator = false;

        // Skip 'publicus' (no-op, fields are public by default)
        matchKeyword('publicus');

        if (matchKeyword('privatus')) {
            isPrivate = true;
        }

        if (matchKeyword('generis')) {
            isStatic = true;
        }

        if (matchKeyword('nexum')) {
            isReactive = true;
        }

        // Parse function modifiers (futura = async, cursor = generator)
        while (checkKeyword('futura') || checkKeyword('cursor')) {
            if (matchKeyword('futura')) {
                prefixAsync = true;
            } else if (matchKeyword('cursor')) {
                prefixGenerator = true;
            }
        }

        // If we see 'functio', it's a method
        if (checkKeyword('functio')) {
            expectKeyword('functio', ParserErrorCode.ExpectedKeywordFunctio);

            const methodName = parseIdentifier();

            expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);

            const params = parseParameterList();

            expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

            let returnType: TypeAnnotation | undefined;
            let verbAsync: boolean | undefined;
            let verbGenerator: boolean | undefined;

            // Parse return type with arrow or verb form
            if (match('THIN_ARROW')) {
                returnType = parseTypeAnnotation();
            } else if (matchKeyword('fit')) {
                returnType = parseTypeAnnotation();
                verbAsync = false;
                verbGenerator = false;
            } else if (matchKeyword('fiet')) {
                returnType = parseTypeAnnotation();
                verbAsync = true;
                verbGenerator = false;
            } else if (matchKeyword('fiunt')) {
                returnType = parseTypeAnnotation();
                verbAsync = false;
                verbGenerator = true;
            } else if (matchKeyword('fient')) {
                returnType = parseTypeAnnotation();
                verbAsync = true;
                verbGenerator = true;
            }

            // Validate prefix/verb agreement
            if (verbAsync === false && prefixAsync) {
                errors.push({
                    code: ParserErrorCode.GenericError,
                    message: 'fit/fiunt (sync) contradicts futura (async)',
                    position,
                });
            }
            if (verbGenerator === false && prefixGenerator) {
                errors.push({
                    code: ParserErrorCode.GenericError,
                    message: 'fit/fiet (single) contradicts cursor (generator)',
                    position,
                });
            }

            const body = parseBlockStatement();

            const method: FunctionDeclaration = {
                type: 'FunctionDeclaration',
                name: methodName,
                params,
                returnType,
                body,
                async: verbAsync ?? prefixAsync,
                generator: verbGenerator ?? prefixGenerator,
                position,
            };

            if (methodName.name === 'creo') {
                method.isConstructor = true;
            }

            return method;
        }

        // Otherwise it's a field: type name with optional ':' default
        const fieldType = parseTypeAnnotation();
        const fieldName = parseIdentifier();

        let init: Expression | undefined;

        // WHY: Field defaults use ':' (declarative "has value") not '=' (imperative "assign")
        // This aligns with object literal syntax: { nomen: "Marcus" }
        if (match('COLON')) {
            init = parseExpression();
        }

        return {
            type: 'FieldDeclaration',
            name: fieldName,
            fieldType,
            init,
            isPrivate,
            isStatic,
            isReactive,
            position,
        };
    }

    // ---------------------------------------------------------------------------
    // Pactum (Interface) Declarations
    // ---------------------------------------------------------------------------

    /**
     * Parse interface declaration.
     *
     * GRAMMAR:
     *   pactumDecl := 'pactum' IDENTIFIER typeParams? '{' pactumMethod* '}'
     *   typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
     *
     * WHY: Latin 'pactum' (agreement/contract) for interfaces.
     *      Defines method signatures that genus types can implement via 'implet'.
     *
     * Examples:
     *   pactum Legibilis { functio lege() -> textus }
     *   pactum Mappabilis<T, U> { functio mappa(T valor) -> U }
     */
    function parsePactumDeclaration(): PactumDeclaration {
        const position = peek().position;

        expectKeyword('pactum', ParserErrorCode.ExpectedKeywordPactum);

        const name = parseIdentifier();

        let typeParameters: Identifier[] | undefined;

        if (match('LESS')) {
            typeParameters = [];

            do {
                typeParameters.push(parseIdentifier());
            } while (match('COMMA'));

            expect('GREATER', ParserErrorCode.ExpectedClosingAngle);
        }

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const methods: PactumMethod[] = [];

        // True while there are unparsed methods (not at '}' or EOF)
        const hasMoreMethods = () => !check('RBRACE') && !isAtEnd();

        while (hasMoreMethods()) {
            methods.push(parsePactumMethod());
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return { type: 'PactumDeclaration', name, typeParameters, methods, position };
    }

    /**
     * Parse interface method signature.
     *
     * GRAMMAR:
     *   pactumMethod := ('futura' | 'cursor')* 'functio' IDENTIFIER '(' paramList ')' returnClause?
     *   returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
     *
     * WHY: Method signatures without bodies. Same syntax as function declarations
     *      but terminates after return type (no block).
     */
    function parsePactumMethod(): PactumMethod {
        const position = peek().position;

        // Parse optional prefixes (futura = async, cursor = generator)
        let prefixAsync = false;
        let prefixGenerator = false;

        while (checkKeyword('futura') || checkKeyword('cursor')) {
            if (matchKeyword('futura')) {
                prefixAsync = true;
            } else if (matchKeyword('cursor')) {
                prefixGenerator = true;
            }
        }

        expectKeyword('functio', ParserErrorCode.ExpectedKeywordFunctio);

        const name = parseIdentifier();

        expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);

        const params = parseParameterList();

        expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

        let returnType: TypeAnnotation | undefined;
        let verbAsync: boolean | undefined;
        let verbGenerator: boolean | undefined;

        // Parse return type with arrow or verb form
        if (match('THIN_ARROW')) {
            returnType = parseTypeAnnotation();
        } else if (matchKeyword('fit')) {
            returnType = parseTypeAnnotation();
            verbAsync = false;
            verbGenerator = false;
        } else if (matchKeyword('fiet')) {
            returnType = parseTypeAnnotation();
            verbAsync = true;
            verbGenerator = false;
        } else if (matchKeyword('fiunt')) {
            returnType = parseTypeAnnotation();
            verbAsync = false;
            verbGenerator = true;
        } else if (matchKeyword('fient')) {
            returnType = parseTypeAnnotation();
            verbAsync = true;
            verbGenerator = true;
        }

        // Validate prefix/verb agreement
        if (verbAsync === false && prefixAsync) {
            errors.push({
                code: ParserErrorCode.GenericError,
                message: 'fit/fiunt (sync) contradicts futura (async)',
                position,
            });
        }
        if (verbGenerator === false && prefixGenerator) {
            errors.push({
                code: ParserErrorCode.GenericError,
                message: 'fit/fiet (single) contradicts cursor (generator)',
                position,
            });
        }

        return {
            type: 'PactumMethod',
            name,
            params,
            returnType,
            async: verbAsync ?? prefixAsync,
            generator: verbGenerator ?? prefixGenerator,
            position,
        };
    }

    // ---------------------------------------------------------------------------
    // Control Flow Statements
    // ---------------------------------------------------------------------------

    /**
     * Parse if statement.
     *
     * GRAMMAR:
     *   ifStmt := 'si' expression (blockStmt | 'ergo' statement) ('cape' IDENTIFIER blockStmt)? (elseClause | 'sin' ifStmt)?
     *   elseClause := ('aliter' | 'secus') (ifStmt | blockStmt | statement)
     *
     * WHY: 'cape' (catch/seize) clause allows error handling within conditionals.
     *      'ergo' (therefore) for one-liner consequents.
     *
     * TWO STYLE OPTIONS (both supported, can be mixed within the same chain):
     *
     *   Literal style: si / aliter si / aliter
     *      si x > 0 { positive() }
     *      aliter si x < 0 { negative() }
     *      aliter { zero() }
     *
     *   Poetic style: si / sin / secus
     *      si x > 0 { positive() }
     *      sin x < 0 { negative() }    // "sin" = "but if" (classical Latin)
     *      secus { zero() }            // "secus" = "otherwise"
     *
     * Keywords are interchangeable at each branch point:
     *      - 'aliter si' ≡ 'sin' (else-if)
     *      - 'aliter' ≡ 'secus' (else)
     *      - Mixed: si ... sin ... aliter { } is valid
     *
     * Examples:
     *   si x > 5 ergo scribe("big")
     *   si x > 5 { scribe("big") } aliter scribe("small")
     *   si x < 0 { ... } sin x == 0 { ... } secus { ... }
     */
    function parseIfStatement(skipSiKeyword = false): IfStatement {
        const position = peek().position;

        if (!skipSiKeyword) {
            expectKeyword('si', ParserErrorCode.ExpectedKeywordSi);
        }

        const test = parseExpression();

        // Parse consequent: block or ergo one-liner
        let consequent: BlockStatement;
        if (matchKeyword('ergo')) {
            const stmtPos = peek().position;
            const stmt = parseStatement();
            consequent = { type: 'BlockStatement', body: [stmt], position: stmtPos };
        } else {
            consequent = parseBlockStatement();
        }

        // Check for cape (catch) clause
        let catchClause: CatchClause | undefined;

        if (checkKeyword('cape')) {
            catchClause = parseCatchClause();
        }

        // Check for alternate: aliter/secus (else) or sin (else-if)
        let alternate: BlockStatement | IfStatement | undefined;

        if (matchKeyword('aliter') || matchKeyword('secus')) {
            if (checkKeyword('si')) {
                alternate = parseIfStatement();
            } else if (check('LBRACE')) {
                alternate = parseBlockStatement();
            } else {
                // One-liner: aliter/secus statement (no ergo needed)
                const stmtPos = peek().position;
                const stmt = parseStatement();
                alternate = { type: 'BlockStatement', body: [stmt], position: stmtPos };
            }
        } else if (matchKeyword('sin')) {
            // "sin" (but if) is classical Latin for else-if
            alternate = parseIfStatement(true);
        }

        return { type: 'IfStatement', test, consequent, alternate, catchClause, position };
    }

    /**
     * Parse while loop statement.
     *
     * GRAMMAR:
     *   whileStmt := 'dum' expression (blockStmt | 'ergo' statement) ('cape' IDENTIFIER blockStmt)?
     *
     * WHY: 'dum' (while/until) for while loops.
     *
     * Examples:
     *   dum x > 0 { x = x - 1 }
     *   dum x > 0 ergo x = x - 1
     */
    function parseWhileStatement(): WhileStatement {
        const position = peek().position;

        expectKeyword('dum', ParserErrorCode.ExpectedKeywordDum);

        const test = parseExpression();

        // Parse body: block or ergo one-liner
        let body: BlockStatement;
        if (matchKeyword('ergo')) {
            const stmtPos = peek().position;
            const stmt = parseStatement();
            body = { type: 'BlockStatement', body: [stmt], position: stmtPos };
        } else {
            body = parseBlockStatement();
        }

        let catchClause: CatchClause | undefined;

        if (checkKeyword('cape')) {
            catchClause = parseCatchClause();
        }

        return { type: 'WhileStatement', test, body, catchClause, position };
    }

    /**
     * Parse statement starting with 'ex'.
     *
     * Dispatches to either:
     * - ForStatement: ex SOURCE (pro|fit|fiet) VARIABLE { }
     * - VariableDeclaration (destructuring): ex SOURCE (fixum|varia|figendum|variandum) { props }
     *
     * GRAMMAR:
     *   exStmt := 'ex' expression (forBinding | destructBinding)
     *   forBinding := ('pro' | 'fit' | 'fiet') IDENTIFIER (blockStmt | 'ergo' statement) catchClause?
     *   destructBinding := ('fixum' | 'varia' | 'figendum' | 'variandum') objectPattern
     *
     * WHY: 'ex' (from/out of) introduces both iteration and extraction:
     *      - Iteration: ex items pro item { ... } (for each item from items)
     *      - Destructuring: ex response fixum { data } (extract data from response)
     *      - Async destructuring: ex promise figendum { result } (await + extract)
     *
     * The binding keywords encode mutability and async semantics:
     *      - fixum: immutable binding (const)
     *      - varia: mutable binding (let)
     *      - figendum: immutable + await (const with await)
     *      - variandum: mutable + await (let with await)
     *
     * Examples:
     *   ex numeri pro n { ... }              // for-loop (sync)
     *   ex numeri fiet n { ... }             // for-await-of loop (async)
     *   ex response fixum { status, data }   // destructuring (sync)
     *   ex fetchData() figendum { result }   // destructuring (async, awaits first)
     */
    function parseExStatement(): ForStatement | VariableDeclaration {
        const position = peek().position;

        expectKeyword('ex', ParserErrorCode.InvalidForLoopStart);

        const source = parseExpression();

        // Dispatch based on what follows the expression
        if (checkKeyword('fixum') || checkKeyword('varia') || checkKeyword('figendum') || checkKeyword('variandum')) {
            // Destructuring: ex source fixum { ... } or async: ex source figendum { ... }
            const kind = advance().keyword as 'varia' | 'fixum' | 'figendum' | 'variandum';
            const name = parseObjectPattern();

            return { type: 'VariableDeclaration', kind, name, init: source, position };
        }

        // Otherwise it's a for-loop: ex source pro/fit/fiet variable { }
        let async = false;
        if (matchKeyword('pro')) {
            async = false;
        } else if (matchKeyword('fit')) {
            async = false;
        } else if (matchKeyword('fiet')) {
            async = true;
        } else {
            error(ParserErrorCode.ExpectedKeywordPro);
        }

        const variable = parseIdentifier();

        // Parse body: block or ergo one-liner
        let body: BlockStatement;
        if (matchKeyword('ergo')) {
            const stmtPos = peek().position;
            const stmt = parseStatement();
            body = { type: 'BlockStatement', body: [stmt], position: stmtPos };
        } else {
            body = parseBlockStatement();
        }

        let catchClause: CatchClause | undefined;
        if (checkKeyword('cape')) {
            catchClause = parseCatchClause();
        }

        return {
            type: 'ForStatement',
            kind: 'ex',
            variable,
            iterable: source,
            body,
            async,
            catchClause,
            position,
        };
    }

    /**
     * Parse 'de' statement (for-in loop).
     *
     * GRAMMAR:
     *   deStmt := 'de' expression ('pro' | 'fit' | 'fiet') IDENTIFIER
     *             (blockStmt | 'ergo' statement) catchClause?
     *
     * WHY: 'de' (from/concerning) for extracting keys from an object.
     *      Semantically read-only - contrasts with 'in' for mutation.
     *
     * Examples:
     *   de tabula pro clavis { ... }  // from table, for each key
     *   de object pro k ergo scribe k // one-liner form
     */
    function parseDeStatement(): ForStatement {
        const position = peek().position;

        expectKeyword('de', ParserErrorCode.ExpectedKeywordDe);

        const expr = parseExpression();

        // Require binding keyword
        let async = false;
        if (matchKeyword('pro')) {
            async = false;
        } else if (matchKeyword('fit')) {
            async = false;
        } else if (matchKeyword('fiet')) {
            async = true;
        } else {
            error(ParserErrorCode.ExpectedKeywordPro);
        }

        const variable = parseIdentifier();

        // Parse body: block or ergo one-liner
        let body: BlockStatement;
        if (matchKeyword('ergo')) {
            const stmtPos = peek().position;
            const stmt = parseStatement();
            body = { type: 'BlockStatement', body: [stmt], position: stmtPos };
        } else {
            body = parseBlockStatement();
        }

        let catchClause: CatchClause | undefined;
        if (checkKeyword('cape')) {
            catchClause = parseCatchClause();
        }

        return {
            type: 'ForStatement',
            kind: 'in', // Still 'in' kind for codegen (for-in loop)
            variable,
            iterable: expr,
            body,
            async,
            catchClause,
            position,
        };
    }

    /**
     * Parse 'in' statement (mutation block).
     *
     * GRAMMAR:
     *   inStmt := 'in' expression blockStmt
     *
     * WHY: 'in' (into) for reaching into an object to modify it.
     *      Semantically mutable - contrasts with 'de' for read-only iteration.
     *
     * Example:
     *   in user { nomen = "Marcus" }  // mutation block
     */
    function parseInStatement(): WithStatement {
        const position = peek().position;

        expectKeyword('in', ParserErrorCode.ExpectedKeywordIn);

        const expr = parseExpression();
        const body = parseBlockStatement();

        return { type: 'WithStatement', object: expr, body, position };
    }

    /**
     * Parse switch statement.
     *
     * GRAMMAR:
     *   switchStmt := 'elige' expression '{' switchCase* defaultCase? '}' catchClause?
     *   switchCase := 'si' expression (blockStmt | 'ergo' expression)
     *   defaultCase := ('aliter' | 'secus') (blockStmt | statement)
     *
     * WHY: 'elige' (choose) for switch, 'si' (if) for cases, 'ergo' (therefore) for one-liners.
     *      'aliter'/'secus' (otherwise) doesn't need 'ergo' - it's already the consequence.
     *
     * Examples:
     *   elige status {
     *       si "pending" ergo scribe("waiting")
     *       si "active" { processActive() }
     *       aliter iace "Unknown status"
     *   }
     */
    function parseSwitchStatement(): SwitchStatement {
        const position = peek().position;

        expectKeyword('elige', ParserErrorCode.ExpectedKeywordElige);

        const discriminant = parseExpression();

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const cases: SwitchCase[] = [];
        let defaultCase: BlockStatement | undefined;

        // Helper: parse 'si' case body (requires ergo or block)
        function parseSiBody(): BlockStatement {
            if (matchKeyword('ergo')) {
                const stmtPos = peek().position;
                const stmt = parseStatement();
                return {
                    type: 'BlockStatement',
                    body: [stmt],
                    position: stmtPos,
                };
            }
            return parseBlockStatement();
        }

        // Helper: parse 'aliter' body (block or direct statement, no ergo needed)
        function parseAliterBody(): BlockStatement {
            if (check('LBRACE')) {
                return parseBlockStatement();
            }
            const stmtPos = peek().position;
            const stmt = parseStatement();
            return {
                type: 'BlockStatement',
                body: [stmt],
                position: stmtPos,
            };
        }

        // True while there are unparsed cases (not at '}' or EOF)
        const hasMoreCases = () => !check('RBRACE') && !isAtEnd();

        while (hasMoreCases()) {
            if (checkKeyword('si')) {
                const casePosition = peek().position;

                expectKeyword('si', ParserErrorCode.ExpectedKeywordSi);

                const test = parseExpression();
                const consequent = parseSiBody();

                cases.push({ type: 'SwitchCase', test, consequent, position: casePosition });
            } else if (checkKeyword('aliter') || checkKeyword('secus')) {
                advance(); // consume aliter or secus

                defaultCase = parseAliterBody();
                break; // Default must be last
            } else {
                error(ParserErrorCode.InvalidSwitchCaseStart);
                break;
            }
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        let catchClause: CatchClause | undefined;

        if (checkKeyword('cape')) {
            catchClause = parseCatchClause();
        }

        return { type: 'SwitchStatement', discriminant, cases, defaultCase, catchClause, position };
    }

    /**
     * Parse guard statement.
     *
     * GRAMMAR:
     *   guardStmt := 'custodi' '{' guardClause+ '}'
     *   guardClause := 'si' expression blockStmt
     *
     * WHY: 'custodi' (guard!) groups early-exit conditions.
     *
     * Example:
     *   custodi {
     *       si user == nihil { redde nihil }
     *       si useri age < 0 { iace "Invalid age" }
     *   }
     */
    function parseGuardStatement(): GuardStatement {
        const position = peek().position;

        expectKeyword('custodi', ParserErrorCode.ExpectedKeywordCustodi);

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const clauses: GuardClause[] = [];

        // True while there are unparsed clauses (not at '}' or EOF)
        const hasMoreClauses = () => !check('RBRACE') && !isAtEnd();

        while (hasMoreClauses()) {
            if (checkKeyword('si')) {
                const clausePosition = peek().position;

                expectKeyword('si', ParserErrorCode.ExpectedKeywordSi);

                const test = parseExpression();
                const consequent = parseBlockStatement();

                clauses.push({ type: 'GuardClause', test, consequent, position: clausePosition });
            } else {
                error(ParserErrorCode.InvalidGuardClauseStart);
                break;
            }
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return { type: 'GuardStatement', clauses, position };
    }

    /**
     * Parse assert statement.
     *
     * GRAMMAR:
     *   assertStmt := 'adfirma' expression (',' expression)?
     *
     * WHY: 'adfirma' (affirm/assert) for runtime invariant checks.
     *
     * Example:
     *   adfirma x > 0
     *   adfirma x > 0, "x must be positive"
     */
    function parseAssertStatement(): AssertStatement {
        const position = peek().position;

        expectKeyword('adfirma', ParserErrorCode.ExpectedKeywordAdfirma);

        const test = parseExpression();

        let message: Expression | undefined;

        if (match('COMMA')) {
            message = parseExpression();
        }

        return { type: 'AssertStatement', test, message, position };
    }

    /**
     * Parse return statement.
     *
     * GRAMMAR:
     *   returnStmt := 'redde' expression?
     *
     * WHY: 'redde' (give back/return) for return statements.
     */
    function parseReturnStatement(): ReturnStatement {
        const position = peek().position;

        expectKeyword('redde', ParserErrorCode.ExpectedKeywordRedde);

        let argument: Expression | undefined;

        if (!check('RBRACE') && !isAtEnd()) {
            argument = parseExpression();
        }

        return { type: 'ReturnStatement', argument, position };
    }

    /**
     * Parse break statement.
     *
     * GRAMMAR:
     *   breakStmt := 'rumpe'
     *
     * WHY: 'rumpe' (break!) exits the innermost loop.
     */
    function parseBreakStatement(): BreakStatement {
        const position = peek().position;

        // Consume the 'rumpe' keyword (already validated by checkKeyword in parseStatement)
        advance();

        return { type: 'BreakStatement', position };
    }

    /**
     * Parse continue statement.
     *
     * GRAMMAR:
     *   continueStmt := 'perge'
     *
     * WHY: 'perge' (continue/proceed!) skips to the next loop iteration.
     */
    function parseContinueStatement(): ContinueStatement {
        const position = peek().position;

        // Consume the 'perge' keyword (already validated by checkKeyword in parseStatement)
        advance();

        return { type: 'ContinueStatement', position };
    }

    /**
     * Parse throw/panic statement.
     *
     * GRAMMAR:
     *   throwStmt := ('iace' | 'mori') expression
     *
     * WHY: Two error severity levels:
     *   iace (throw!) → recoverable, can be caught
     *   mori (die!)   → fatal/panic, unrecoverable
     */
    function parseThrowStatement(fatal: boolean): ThrowStatement {
        const position = peek().position;

        // Consume the keyword (already validated by checkKeyword in parseStatement)
        advance();

        const argument = parseExpression();

        return { type: 'ThrowStatement', fatal, argument, position };
    }

    /**
     * Parse output statement (scribe/vide/mone).
     *
     * GRAMMAR:
     *   outputStmt := ('scribe' | 'vide' | 'mone') expression (',' expression)*
     *
     * WHY: Latin output keywords as statement forms:
     *   scribe (write!) → console.log
     *   vide (see!)     → console.debug
     *   mone (warn!)    → console.warn
     *
     * Examples:
     *   scribe "hello"
     *   vide "debugging:", value
     *   mone "warning:", message
     */
    function parseScribeStatement(level: OutputLevel): ScribeStatement {
        const position = peek().position;

        // Consume the keyword (already validated by checkKeyword in parseStatement)
        advance();

        const args: Expression[] = [];

        // Parse first argument (required)
        args.push(parseExpression());

        // Parse additional comma-separated arguments
        while (match('COMMA')) {
            args.push(parseExpression());
        }

        return { type: 'ScribeStatement', level, arguments: args, position };
    }

    /**
     * Parse try-catch-finally statement.
     *
     * GRAMMAR:
     *   tryStmt := 'tempta' blockStmt ('cape' IDENTIFIER blockStmt)? ('demum' blockStmt)?
     *
     * WHY: 'tempta' (attempt/try), 'cape' (catch/seize), 'demum' (finally/at last).
     */
    function parseTryStatement(): Statement {
        const position = peek().position;

        expectKeyword('tempta', ParserErrorCode.ExpectedKeywordTempta);

        const block = parseBlockStatement();

        let handler: CatchClause | undefined;

        if (checkKeyword('cape')) {
            handler = parseCatchClause();
        }

        let finalizer: BlockStatement | undefined;

        if (matchKeyword('demum')) {
            finalizer = parseBlockStatement();
        }

        return { type: 'TryStatement', block, handler, finalizer, position };
    }

    /**
     * Parse catch clause.
     *
     * GRAMMAR:
     *   catchClause := 'cape' IDENTIFIER blockStmt
     */
    function parseCatchClause(): CatchClause {
        const position = peek().position;

        expectKeyword('cape', ParserErrorCode.ExpectedKeywordCape);

        const param = parseIdentifier();
        const body = parseBlockStatement();

        return { type: 'CatchClause', param, body, position };
    }

    /**
     * Parse fac block statement (explicit scope block).
     *
     * GRAMMAR:
     *   facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)?
     *
     * WHY: 'fac' (do/make) creates an explicit scope boundary for grouping
     *      statements with optional error handling via 'cape' (catch).
     *      Useful for:
     *      - Scoped variable declarations
     *      - Grouping related operations with shared error handling
     *      - Creating IIFE-like constructs
     *
     * Examples:
     *   fac { fixum x = computeValue() }
     *   fac { riskyOperation() } cape e { scribe e }
     */
    function parseFacBlockStatement(): FacBlockStatement {
        const position = peek().position;

        expectKeyword('fac', ParserErrorCode.ExpectedKeywordFac);

        const body = parseBlockStatement();

        let catchClause: CatchClause | undefined;

        if (checkKeyword('cape')) {
            catchClause = parseCatchClause();
        }

        return { type: 'FacBlockStatement', body, catchClause, position };
    }

    /**
     * Parse block statement.
     *
     * GRAMMAR:
     *   blockStmt := '{' statement* '}'
     */
    function parseBlockStatement(): BlockStatement {
        const position = peek().position;

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const body: Statement[] = [];

        // True while there are unparsed statements (not at '}' or EOF)
        const hasMoreStatements = () => !check('RBRACE') && !isAtEnd();

        while (hasMoreStatements()) {
            // Consume optional semicolons between statements
            while (match('SEMICOLON')) {
                // do nothing - avoids linter no-empty
            }

            if (!hasMoreStatements()) {
                break;
            }

            body.push(parseStatement());
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return { type: 'BlockStatement', body, position };
    }

    /**
     * Parse expression statement.
     *
     * GRAMMAR:
     *   exprStmt := expression
     */
    function parseExpressionStatement(): ExpressionStatement {
        const position = peek().position;
        const expression = parseExpression();

        return { type: 'ExpressionStatement', expression, position };
    }

    // =============================================================================
    // EXPRESSION PARSING (Precedence Climbing)
    // =============================================================================

    /**
     * Parse expression (entry point for precedence climbing).
     *
     * GRAMMAR:
     *   expression := assignment
     *
     * WHY: Top-level expression delegates to assignment (lowest precedence).
     */
    function parseExpression(): Expression {
        return parseAssignment();
    }

    /**
     * Parse assignment expression.
     *
     * GRAMMAR:
     *   assignment := ternary ('=' assignment)?
     *
     * PRECEDENCE: Lowest (right-associative via recursion).
     *
     * ERROR RECOVERY: Reports error if left side is not valid lvalue.
     */
    function parseAssignment(): Expression {
        const expr = parseTernary();

        if (match('EQUAL')) {
            const position = peek().position;
            const value = parseAssignment();

            if (expr.type === 'Identifier' || expr.type === 'MemberExpression') {
                return {
                    type: 'AssignmentExpression',
                    operator: '=',
                    left: expr,
                    right: value,
                    position,
                };
            }

            error(ParserErrorCode.InvalidAssignmentTarget);
        }

        return expr;
    }

    /**
     * Parse ternary conditional expression.
     *
     * GRAMMAR:
     *   ternary := or (('?' expression ':' | 'sic' expression 'secus') ternary)?
     *
     * PRECEDENCE: Between assignment and logical OR (right-associative).
     *
     * WHY: Supports both symbolic (? :) and Latin (sic secus) syntax.
     *      The two forms cannot be mixed: use either ? : or sic secus.
     *
     * Examples:
     *   verum ? 1 : 0              // symbolic style
     *   verum sic 1 secus 0        // Latin style
     *   a ? b ? c : d : e          // nested (right-associative)
     */
    function parseTernary(): Expression {
        const test = parseOr();
        const position = test.position;

        // Check for symbolic ternary: condition ? consequent : alternate
        if (match('QUESTION')) {
            const consequent = parseExpression();

            if (!match('COLON')) {
                error(ParserErrorCode.ExpectedColon, `got '${peek().value}'`);
            }

            const alternate = parseTernary();

            return {
                type: 'ConditionalExpression',
                test,
                consequent,
                alternate,
                position,
            } as ConditionalExpression;
        }

        // Check for Latin ternary: condition sic consequent secus alternate
        if (matchKeyword('sic')) {
            const consequent = parseExpression();

            if (!matchKeyword('secus')) {
                error(ParserErrorCode.ExpectedKeywordSecus, `got '${peek().value}'`);
            }

            const alternate = parseTernary();

            return {
                type: 'ConditionalExpression',
                test,
                consequent,
                alternate,
                position,
            } as ConditionalExpression;
        }

        return test;
    }

    /**
     * Parse logical OR and nullish coalescing expressions.
     *
     * GRAMMAR:
     *   or := and (('||' | 'aut') and)* | and ('vel' and)*
     *
     * PRECEDENCE: Lower than AND, higher than assignment.
     *
     * WHY: Latin 'aut' (or) for logical OR, 'vel' (or) for nullish coalescing.
     *      Mixing aut/|| with vel without parentheses is a syntax error
     *      (same as JavaScript's ?? and || restriction).
     */
    function parseOr(): Expression {
        let left = parseAnd();

        // Track which operator family we're using to prevent mixing
        let operatorKind: 'logical' | 'nullish' | null = null;

        while (true) {
            let isLogicalOr = false;
            let isNullish = false;

            if (match('OR') || matchKeyword('aut')) {
                isLogicalOr = true;
            } else if (matchKeyword('vel')) {
                isNullish = true;
            } else {
                break;
            }

            const currentKind = isLogicalOr ? 'logical' : 'nullish';

            // WHY: Like JavaScript, mixing ?? and || without parens is ambiguous
            if (operatorKind !== null && operatorKind !== currentKind) {
                errors.push({
                    message: `Cannot mix 'vel' (nullish) and 'aut'/'||' (logical) without parentheses`,
                    position: peek().position,
                });
            }

            operatorKind = currentKind;
            const position = peek().position;
            const operator = isLogicalOr ? '||' : '??';
            const right = parseAnd();

            left = { type: 'BinaryExpression', operator, left, right, position };
        }

        return left;
    }

    /**
     * Parse logical AND expression.
     *
     * GRAMMAR:
     *   and := equality ('&&' equality | 'et' equality)*
     *
     * PRECEDENCE: Lower than equality, higher than OR.
     *
     * WHY: Latin 'et' (and) supported alongside '&&'.
     */
    function parseAnd(): Expression {
        let left = parseEquality();

        while (match('AND') || matchKeyword('et')) {
            const position = peek().position;
            const right = parseEquality();

            left = { type: 'BinaryExpression', operator: '&&', left, right, position };
        }

        return left;
    }

    /**
     * Parse equality expression.
     *
     * GRAMMAR:
     *   equality := comparison (('==' | '!=' | 'est' | 'non' 'est') comparison)*
     *
     * PRECEDENCE: Lower than comparison, higher than AND.
     *
     * WHY: 'est' (is) is the Latin copula for strict equality (===).
     *      'non est' (is not) is strict inequality (!==).
     *      Allows natural syntax: si x est nihil { ... }
     */
    function parseEquality(): Expression {
        let left = parseComparison();

        while (true) {
            let operator: string;
            let position: Position;

            if (match('EQUAL_EQUAL', 'BANG_EQUAL', 'TRIPLE_EQUAL', 'BANG_DOUBLE_EQUAL')) {
                operator = tokens[current - 1].value;
                position = tokens[current - 1].position;
            } else if (checkKeyword('non') && peek(1)?.type === 'KEYWORD' && peek(1)?.value.toLowerCase() === 'est') {
                // 'non est' maps to strict inequality (!==)
                position = peek().position;
                advance(); // consume 'non'
                advance(); // consume 'est'
                operator = '!==';
            } else if (matchKeyword('est')) {
                // 'est' maps to strict equality (===)
                operator = '===';
                position = tokens[current - 1].position;
            } else {
                break;
            }

            const right = parseComparison();
            left = { type: 'BinaryExpression', operator, left, right, position };
        }

        return left;
    }

    /**
     * Parse comparison expression.
     *
     * GRAMMAR:
     *   comparison := range (('<' | '>' | '<=' | '>=') range)*
     *
     * PRECEDENCE: Lower than range, higher than equality.
     */
    function parseComparison(): Expression {
        let left = parseRange();

        while (match('LESS', 'LESS_EQUAL', 'GREATER', 'GREATER_EQUAL')) {
            const operator = tokens[current - 1].value;
            const position = tokens[current - 1].position;
            const right = parseRange();

            left = { type: 'BinaryExpression', operator, left, right, position };
        }

        return left;
    }

    /**
     * Parse range expression.
     *
     * GRAMMAR:
     *   range := additive (('..' | 'ante' | 'usque') additive ('per' additive)?)?
     *
     * PRECEDENCE: Lower than additive, higher than comparison.
     *
     * WHY: Range expressions provide concise numeric iteration.
     *      Three operators with different end semantics:
     *      - '..' and 'ante': exclusive (0..10 / 0 ante 10 = 0-9)
     *      - 'usque': inclusive (0 usque 10 = 0-10)
     *      Optional step via 'per' keyword.
     *
     * Examples:
     *   0..10           -> RangeExpression(0, 10, inclusive=false)
     *   0 ante 10       -> RangeExpression(0, 10, inclusive=false)
     *   0 usque 10      -> RangeExpression(0, 10, inclusive=true)
     *   0..10 per 2     -> RangeExpression(0, 10, 2, inclusive=false)
     */
    function parseRange(): Expression {
        const start = parseAdditive();

        // Check for range operators: .., ante (exclusive), usque (inclusive)
        let inclusive = false;

        if (match('DOT_DOT')) {
            inclusive = false;
        } else if (matchKeyword('ante')) {
            inclusive = false;
        } else if (matchKeyword('usque')) {
            inclusive = true;
        } else {
            return start;
        }

        const position = tokens[current - 1].position;
        const end = parseAdditive();

        let step: Expression | undefined;

        if (matchKeyword('per')) {
            step = parseAdditive();
        }

        return { type: 'RangeExpression', start, end, step, inclusive, position } as RangeExpression;
    }

    /**
     * Parse additive expression.
     *
     * GRAMMAR:
     *   additive := multiplicative (('+' | '-') multiplicative)*
     *
     * PRECEDENCE: Lower than multiplicative, higher than range.
     */
    function parseAdditive(): Expression {
        let left = parseMultiplicative();

        while (match('PLUS', 'MINUS')) {
            const operator = tokens[current - 1].value;
            const position = tokens[current - 1].position;
            const right = parseMultiplicative();

            left = { type: 'BinaryExpression', operator, left, right, position };
        }

        return left;
    }

    /**
     * Parse multiplicative expression.
     *
     * GRAMMAR:
     *   multiplicative := unary (('*' | '/' | '%') unary)*
     *
     * PRECEDENCE: Lower than unary, higher than additive.
     */
    function parseMultiplicative(): Expression {
        let left = parseUnary();

        while (match('STAR', 'SLASH', 'PERCENT')) {
            const operator = tokens[current - 1].value;
            const position = tokens[current - 1].position;
            const right = parseUnary();

            left = { type: 'BinaryExpression', operator, left, right, position };
        }

        return left;
    }

    /**
     * Parse unary expression.
     *
     * GRAMMAR:
     *   unary := ('!' | '-' | 'non' | 'nulla' | 'nonnulla' | 'negativum' | 'positivum' | 'cede' | 'novum') unary | cast
     *
     * PRECEDENCE: Higher than binary operators, lower than cast/call/member access.
     *
     * WHY: Latin 'non' (not), 'nulla' (none/empty), 'nonnulla' (some/non-empty),
     *      'negativum' (< 0), 'positivum' (> 0), 'cede' (await), 'novum' (new).
     */
    function parseUnary(): Expression {
        // WHY: Prefix ! is removed to make room for non-null assertion (postfix !.)
        //      Use 'non' for logical not: "si non x" instead of "si !x"
        if (matchKeyword('non')) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return { type: 'UnaryExpression', operator: '!', argument, prefix: true, position };
        }

        if (match('MINUS')) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return { type: 'UnaryExpression', operator: '-', argument, prefix: true, position };
        }

        if (matchKeyword('nulla')) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return { type: 'UnaryExpression', operator: 'nulla', argument, prefix: true, position };
        }

        if (matchKeyword('nonnulla')) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return {
                type: 'UnaryExpression',
                operator: 'nonnulla',
                argument,
                prefix: true,
                position,
            };
        }

        if (matchKeyword('negativum')) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return {
                type: 'UnaryExpression',
                operator: 'negativum',
                argument,
                prefix: true,
                position,
            };
        }

        if (matchKeyword('positivum')) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return {
                type: 'UnaryExpression',
                operator: 'positivum',
                argument,
                prefix: true,
                position,
            };
        }

        if (matchKeyword('cede')) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return { type: 'AwaitExpression', argument, position };
        }

        if (matchKeyword('novum')) {
            return parseNewExpression();
        }

        return parseCast();
    }

    /**
     * Parse type cast expression.
     *
     * GRAMMAR:
     *   castExpr := call ('ut' typeAnnotation)*
     *
     * PRECEDENCE: Between unary and call. This means:
     *   -x ut T     parses as -(x ut T)    — unary binds looser
     *   x.y ut T    parses as (x.y) ut T   — member access binds tighter
     *   x ut A ut B parses as (x ut A) ut B — left-associative
     *
     * WHY: Latin 'ut' (as, in the capacity of) for type assertions.
     *      Compile-time only — no runtime checking. Maps to:
     *      - TypeScript: x as T
     *      - Python: x (ignored, dynamic typing)
     *      - Zig: @as(T, x)
     *      - Rust: x as T
     *      - C++: static_cast<T>(x)
     */
    function parseCast(): Expression {
        let expr = parseCall();

        while (matchKeyword('ut')) {
            const position = tokens[current - 1].position;
            const targetType = parseTypeAnnotation();

            expr = {
                type: 'TypeCastExpression',
                expression: expr,
                targetType,
                position,
            } as TypeCastExpression;
        }

        return expr;
    }

    /**
     * Parse new expression (object construction).
     *
     * GRAMMAR:
     *   newExpr := 'novum' IDENTIFIER ('(' argumentList ')')? (objectLiteral | 'de' expression)?
     *
     * WHY: Two forms for property overrides:
     *      - Inline literal: `novum Persona { nomen: "Marcus" }`
     *      - From expression: `novum Persona de props` (props is variable/call/etc.)
     *
     *      The `de` (from) form allows dynamic overrides from variables or function results.
     */
    function parseNewExpression(): NewExpression {
        const position = tokens[current - 1].position;
        const callee = parseIdentifier();

        let args: Expression[] = [];

        if (match('LPAREN')) {
            args = parseArgumentList();

            expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
        }

        let withExpression: Expression | undefined;

        // Check for property overrides: novum X { ... } or novum X de expr
        if (check('LBRACE')) {
            withExpression = parsePrimary();
        } else if (matchKeyword('de')) {
            withExpression = parseExpression();
        }

        return { type: 'NewExpression', callee, arguments: args, withExpression, position };
    }

    /**
     * Parse call expression with postfix operators.
     *
     * GRAMMAR:
     *   call := primary (callSuffix | memberSuffix | optionalSuffix | nonNullSuffix)*
     *   callSuffix := '(' argumentList ')'
     *   memberSuffix := '.' IDENTIFIER | '[' expression ']'
     *   optionalSuffix := '?.' IDENTIFIER | '?[' expression ']' | '?(' argumentList ')'
     *   nonNullSuffix := '!.' IDENTIFIER | '![' expression ']' | '!(' argumentList ')'
     *
     * PRECEDENCE: Highest (binds tightest after primary).
     *
     * WHY: Handles function calls, member access, and computed member access.
     *      Left-associative via loop (obj.a.b parsed as (obj.a).b).
     *
     * OPTIONAL CHAINING: ?. ?[ ?( return nihil if object is nihil
     * NON-NULL ASSERTION: !. ![ !( assert object is not nihil
     */
    function parseCall(): Expression {
        let expr = parsePrimary();

        while (true) {
            // ---------------------------------------------------------------
            // Optional chaining: ?. ?[ ?(
            // WHY: Check for QUESTION followed by accessor token to disambiguate from ternary
            // ---------------------------------------------------------------
            if (check('QUESTION') && isChainAccessor(peek(1).type)) {
                const position = peek().position;
                advance(); // consume QUESTION

                if (match('DOT')) {
                    const property = parseIdentifier();
                    expr = {
                        type: 'MemberExpression',
                        object: expr,
                        property,
                        computed: false,
                        optional: true,
                        position,
                    };
                } else if (match('LBRACKET')) {
                    const property = parseExpression();
                    expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);
                    expr = {
                        type: 'MemberExpression',
                        object: expr,
                        property,
                        computed: true,
                        optional: true,
                        position,
                    };
                } else if (match('LPAREN')) {
                    const args = parseArgumentList();
                    expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
                    expr = { type: 'CallExpression', callee: expr, arguments: args, optional: true, position };
                }
            }
            // ---------------------------------------------------------------
            // Non-null assertion: !. ![ !(
            // WHY: BANG is no longer used for prefix logical not, so it's unambiguous
            // ---------------------------------------------------------------
            else if (check('BANG') && isChainAccessor(peek(1).type)) {
                const position = peek().position;
                advance(); // consume BANG

                if (match('DOT')) {
                    const property = parseIdentifier();
                    expr = {
                        type: 'MemberExpression',
                        object: expr,
                        property,
                        computed: false,
                        nonNull: true,
                        position,
                    };
                } else if (match('LBRACKET')) {
                    const property = parseExpression();
                    expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);
                    expr = {
                        type: 'MemberExpression',
                        object: expr,
                        property,
                        computed: true,
                        nonNull: true,
                        position,
                    };
                } else if (match('LPAREN')) {
                    const args = parseArgumentList();
                    expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
                    expr = { type: 'CallExpression', callee: expr, arguments: args, nonNull: true, position };
                }
            }
            // ---------------------------------------------------------------
            // Regular accessors
            // ---------------------------------------------------------------
            else if (match('LPAREN')) {
                const position = tokens[current - 1].position;
                const args = parseArgumentList();

                expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

                expr = { type: 'CallExpression', callee: expr, arguments: args, position };
            } else if (match('DOT')) {
                const position = tokens[current - 1].position;
                const property = parseIdentifier();

                expr = {
                    type: 'MemberExpression',
                    object: expr,
                    property,
                    computed: false,
                    position,
                };
            } else if (match('LBRACKET')) {
                const position = tokens[current - 1].position;
                const property = parseExpression();

                expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);

                expr = {
                    type: 'MemberExpression',
                    object: expr,
                    property,
                    computed: true,
                    position,
                };
            } else {
                break;
            }
        }

        return expr;
    }

    /**
     * Check if token type is a chain accessor (for optional chaining / non-null assertion).
     */
    function isChainAccessor(type: TokenType): boolean {
        return type === 'DOT' || type === 'LBRACKET' || type === 'LPAREN';
    }

    /**
     * Parse function call argument list.
     *
     * GRAMMAR:
     *   argumentList := (argument (',' argument)*)?
     *   argument := 'sparge' expression | expression
     */
    function parseArgumentList(): (Expression | SpreadElement)[] {
        const args: (Expression | SpreadElement)[] = [];

        if (check('RPAREN')) {
            return args;
        }

        do {
            // Check for spread: sparge expr
            if (checkKeyword('sparge')) {
                const spreadPos = peek().position;
                advance(); // consume 'sparge'
                const argument = parseExpression();
                args.push({ type: 'SpreadElement', argument, position: spreadPos });
            } else {
                args.push(parseExpression());
            }
        } while (match('COMMA'));

        return args;
    }

    /**
     * Parse primary expression (literals, identifiers, grouped expressions).
     *
     * GRAMMAR:
     *   primary := IDENTIFIER | NUMBER | STRING | TEMPLATE_STRING
     *            | 'ego' | 'verum' | 'falsum' | 'nihil'
     *            | '(' (expression | arrowFunction) ')'
     *
     * PRECEDENCE: Highest (atoms of the language).
     *
     * WHY: Latin literals: verum (true), falsum (false), nihil (null).
     *      'ego' (I/self) is the self-reference keyword (like 'this' in JS).
     *      Parenthesized expressions require lookahead to distinguish from arrow functions.
     */
    function parsePrimary(): Expression {
        const position = peek().position;

        if (matchKeyword('ego')) {
            const thisExpr: ThisExpression = { type: 'ThisExpression', position };
            return thisExpr;
        }

        // Boolean literals
        if (matchKeyword('verum')) {
            return { type: 'Literal', value: true, raw: 'verum', position };
        }

        if (matchKeyword('falsum')) {
            return { type: 'Literal', value: false, raw: 'falsum', position };
        }

        if (matchKeyword('nihil')) {
            return { type: 'Literal', value: null, raw: 'nihil', position };
        }

        // Lambda expression: pro x redde expr, pro x, y redde expr, pro redde expr
        if (checkKeyword('pro')) {
            return parseProExpression();
        }

        // Number literal
        if (check('NUMBER')) {
            const token = advance();
            const value = token.value.includes('.') ? parseFloat(token.value) : parseInt(token.value, 10);

            return { type: 'Literal', value, raw: token.value, position };
        }

        // Bigint literal
        if (check('BIGINT')) {
            const token = advance();

            return { type: 'Literal', value: BigInt(token.value), raw: `${token.value}n`, position };
        }

        // String literal
        if (check('STRING')) {
            const token = advance();

            return { type: 'Literal', value: token.value, raw: `"${token.value}"`, position };
        }

        // Template string
        if (check('TEMPLATE_STRING')) {
            const token = advance();

            return { type: 'TemplateLiteral', raw: token.value, position };
        }

        // Array literal
        if (match('LBRACKET')) {
            const elements: (Expression | SpreadElement)[] = [];

            if (!check('RBRACKET')) {
                do {
                    // Check for spread element: sparge expr
                    if (checkKeyword('sparge')) {
                        const spreadPos = peek().position;
                        advance(); // consume 'sparge'
                        const argument = parseExpression();
                        elements.push({ type: 'SpreadElement', argument, position: spreadPos });
                    } else {
                        elements.push(parseExpression());
                    }
                } while (match('COMMA'));
            }

            expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);

            return { type: 'ArrayExpression', elements, position };
        }

        // Object literal
        if (match('LBRACE')) {
            const properties: (ObjectProperty | SpreadElement)[] = [];

            if (!check('RBRACE')) {
                do {
                    const propPosition = peek().position;

                    // Check for spread: sparge expr
                    if (checkKeyword('sparge')) {
                        advance(); // consume 'sparge'
                        const argument = parseExpression();
                        properties.push({ type: 'SpreadElement', argument, position: propPosition });
                    } else {
                        // Key can be identifier or string
                        let key: Identifier | Literal;

                        if (check('STRING')) {
                            const token = advance();

                            key = {
                                type: 'Literal',
                                value: token.value,
                                raw: `"${token.value}"`,
                                position: propPosition,
                            };
                        } else {
                            key = parseIdentifier();
                        }

                        expect('COLON', ParserErrorCode.ExpectedColon);

                        const value = parseExpression();

                        properties.push({ type: 'ObjectProperty', key, value, position: propPosition });
                    }
                } while (match('COMMA'));
            }

            expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

            return { type: 'ObjectExpression', properties, position };
        }

        // Parenthesized expression or arrow function
        if (match('LPAREN')) {
            // Could be arrow function: (x) => ...
            // Or grouped expression: (a + b)
            // Peek ahead to see if this is an arrow function

            // Simple heuristic: if we see ) => then it's an arrow function
            const startPos = current;
            let parenDepth = 1;

            while (parenDepth > 0 && !isAtEnd()) {
                if (check('LPAREN')) {
                    parenDepth++;
                }

                if (check('RPAREN')) {
                    parenDepth--;
                }

                if (parenDepth > 0) {
                    advance();
                }
            }

            if (check('RPAREN')) {
                advance(); // consume )
                if (check('ARROW')) {
                    // It's an arrow function, backtrack and parse properly
                    current = startPos;

                    return parseArrowFunction(position);
                }
            }

            // Not an arrow function, backtrack and parse as grouped expression
            current = startPos;
            const expr = parseExpression();

            expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

            return expr;
        }

        // Identifier
        if (check('IDENTIFIER')) {
            return parseIdentifier();
        }

        error(ParserErrorCode.UnexpectedToken, `token '${peek().value}'`);
    }

    /**
     * Parse arrow function expression.
     *
     * GRAMMAR:
     *   arrowFunction := '(' paramList ')' '=>' (expression | blockStmt)
     *
     * WHY: Called after detecting '() =>' pattern in parsePrimary.
     */
    function parseArrowFunction(position: Position): ArrowFunctionExpression {
        const params = parseParameterList();

        expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

        expect('ARROW', ParserErrorCode.ExpectedArrow);

        let body: Expression | BlockStatement;

        if (check('LBRACE')) {
            body = parseBlockStatement();
        } else {
            body = parseExpression();
        }

        return { type: 'ArrowFunctionExpression', params, body, async: false, position };
    }

    /**
     * Parse pro expression (lambda/anonymous function).
     *
     * GRAMMAR:
     *   lambdaExpr := 'pro' params? ((':' | 'redde') expression | blockStmt)
     *   params := IDENTIFIER (',' IDENTIFIER)*
     *
     * WHY: Latin 'pro' (for) creates lambda syntax with two equivalent forms:
     *      - 'pro x redde expr' - explicit return keyword
     *      - 'pro x: expr' - colon shorthand (mirrors object literal syntax)
     *
     * The ':' and 'redde' forms are INTERCHANGEABLE - use whichever reads better:
     *      pro x: x * 2        ≡  pro x redde x * 2      -> (x) => x * 2
     *      pro: 42             ≡  pro redde 42           -> () => 42
     *      pro x, y: x + y     ≡  pro x, y redde x + y   -> (x, y) => x + y
     *
     * Block form (for multi-statement bodies):
     *      pro x { redde x * 2 }     -> (x) => { return x * 2; }
     *      pro { scribe "hi" }       -> () => { console.log("hi"); }
     *
     * Style guidance: Use ':' for short expressions, 'redde' for clarity in complex cases.
     */
    function parseProExpression(): LambdaExpression {
        const position = peek().position;

        expectKeyword('pro', ParserErrorCode.ExpectedKeywordPro);

        const params: Identifier[] = [];

        // Check for immediate redde, :, or { (zero-param lambda)
        if (!checkKeyword('redde') && !check('COLON') && !check('LBRACE')) {
            // Parse parameters until we hit redde, :, or {
            do {
                params.push(parseIdentifier());
            } while (match('COMMA'));
        }

        let body: Expression | BlockStatement;

        if (check('LBRACE')) {
            // Block form: pro x { ... }
            body = parseBlockStatement();
        } else if (match('COLON')) {
            // Expression shorthand: pro x: expr
            body = parseExpression();
        } else {
            // Expression form: pro x redde expr
            expectKeyword('redde', ParserErrorCode.ExpectedKeywordRedde);
            body = parseExpression();
        }

        return { type: 'LambdaExpression', params, body, async: false, position };
    }

    /**
     * Parse identifier.
     *
     * GRAMMAR:
     *   identifier := IDENTIFIER
     */
    function parseIdentifier(): Identifier {
        const token = expect('IDENTIFIER', ParserErrorCode.ExpectedIdentifier);

        return { type: 'Identifier', name: token.value, position: token.position };
    }

    // =============================================================================
    // TYPE ANNOTATION PARSING
    // =============================================================================

    /**
     * Check if token is a borrow preposition (de/in for ownership semantics).
     *
     * WHY: These prepositions encode ownership for systems targets (Rust/Zig):
     *      de = borrowed/read-only (&T, []const u8)
     *      in = mutable borrow (&mut T, *T)
     *
     * @returns true if token is 'de' or 'in' keyword
     */
    function isBorrowPreposition(token: Token): boolean {
        return token.type === 'KEYWORD' && ['de', 'in'].includes(token.keyword ?? '');
    }

    /**
     * Parse type annotation.
     *
     * GRAMMAR:
     *   typeAnnotation := ('de' | 'in')? IDENTIFIER typeParams? '?'? arrayBrackets* ('|' typeAnnotation)*
     *   typeParams := '<' typeParameter (',' typeParameter)* '>'
     *   typeParameter := typeAnnotation | NUMBER | MODIFIER
     *   arrayBrackets := '[]' '?'?
     *
     * WHY: Supports generics (lista<textus>), nullable (?), union types (A | B),
     *      and array shorthand (numerus[] desugars to lista<numerus>).
     *
     * EDGE: Numeric parameters for sized types (numerus<32>).
     *       Modifier parameters for ownership/signedness (numerus<Naturalis>).
     *       Array shorthand preserves source form via arrayShorthand flag.
     *       Borrow prepositions (de/in) for systems targets (Rust/Zig).
     */
    function parseTypeAnnotation(): TypeAnnotation {
        const position = peek().position;

        // Check for borrow preposition (de/in for ownership semantics)
        let preposition: string | undefined;
        if (isBorrowPreposition(peek())) {
            preposition = advance().keyword;
        }

        // WHY: Type names can be identifiers OR keywords (nihil, textus, numerus, etc.)
        //      Keywords like 'nihil' are valid return types but tokenize as KEYWORD
        let name: string;
        if (check('IDENTIFIER')) {
            name = advance().value;
        } else if (check('KEYWORD')) {
            name = advance().value;
        } else {
            reportError(ParserErrorCode.ExpectedTypeName, `got '${peek().value}'`);
            name = peek().value;
            advance(); // Skip to avoid infinite loop
        }

        let typeParameters: TypeParameter[] | undefined;

        if (match('LESS')) {
            typeParameters = [];

            do {
                if (check('NUMBER')) {
                    // Numeric parameter (e.g., numerus<32>)
                    const numToken = advance();
                    const value = numToken.value.includes('.') ? parseFloat(numToken.value) : parseInt(numToken.value, 10);

                    typeParameters.push({
                        type: 'Literal',
                        value,
                        raw: numToken.value,
                        position: numToken.position,
                    });
                } else {
                    // Type parameter (e.g., lista<textus>, numerus<i32>)
                    typeParameters.push(parseTypeAnnotation());
                }
            } while (match('COMMA'));

            expect('GREATER', ParserErrorCode.ExpectedClosingAngle);
        }

        let nullable = false;

        if (match('QUESTION')) {
            nullable = true;
        }

        // Build the base type
        let result: TypeAnnotation = {
            type: 'TypeAnnotation',
            name,
            typeParameters,
            nullable,
            preposition,
            position,
        };

        // Handle array shorthand: numerus[] -> lista<numerus>
        // Each [] wraps in lista with arrayShorthand flag for round-trip fidelity
        while (check('LBRACKET') && peek(1).type === 'RBRACKET') {
            advance(); // [
            advance(); // ]

            let arrayNullable = false;
            if (match('QUESTION')) {
                arrayNullable = true;
            }

            result = {
                type: 'TypeAnnotation',
                name: 'lista',
                typeParameters: [result],
                nullable: arrayNullable,
                arrayShorthand: true,
                position,
            };
        }

        // Handle union types
        if (check('PIPE')) {
            const union: TypeAnnotation[] = [result];
            while (match('PIPE')) {
                union.push(parseTypeAnnotation());
            }

            return { type: 'TypeAnnotation', name: 'union', union, position };
        }

        return result;
    }

    // =============================================================================
    // MAIN PARSE EXECUTION
    // =============================================================================

    /**
     * Execute the parse.
     *
     * ERROR RECOVERY: Top-level try-catch ensures parser never crashes.
     *                 Returns null program on catastrophic failure.
     */
    try {
        const program = parseProgram();

        return { program, errors };
    } catch {
        return { program: null, errors };
    }
}

export * from './ast';
