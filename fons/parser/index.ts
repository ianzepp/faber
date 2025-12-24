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
    ComputedFieldDeclaration,
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
    Literal,
    RangeExpression,
    ConditionalExpression,
    ObjectPattern,
    ObjectPatternProperty,
    ObjectExpression,
    ObjectProperty,
    FacBlockStatement,
    FacExpression,
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
const BUILTIN_TYPE_NAMES = new Set(
    builtinTypes.map(t => t.nominative ?? computeNominative(t.stem, t.declension, t.gender)),
);

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
     * WHY: Prepositions indicate semantic roles (ad, cum, in, ex).
     *
     * @returns true if token is a preposition keyword
     */
    function isPreposition(token: Token): boolean {
        return token.type === 'KEYWORD' && ['ad', 'cum', 'de', 'in', 'ex'].includes(token.keyword ?? '');
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
                checkKeyword('typus') ||
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
     * Parse any statement by dispatching to specific parser.
     *
     * GRAMMAR:
     *   statement := importDecl | varDecl | funcDecl | typeAliasDecl | ifStmt | whileStmt | forStmt
     *              | returnStmt | throwStmt | tryStmt | blockStmt | exprStmt
     *
     * WHY: Uses lookahead to determine statement type via keyword inspection.
     */
    function parseStatement(): Statement {
        // Distinguish 'ex norma importa' (import) from 'ex numeri pro n' (for-loop)
        if (checkKeyword('ex')) {
            // Look ahead: ex IDENTIFIER importa -> import, otherwise for-loop
            if (peek(1).type === 'IDENTIFIER' && peek(2).keyword === 'importa') {
                return parseImportDeclaration();
            }

            return parseForStatement();
        }

        // 'in tabula pro k' is for-in loop
        if (checkKeyword('in')) {
            return parseForStatement();
        }

        if (checkKeyword('varia') || checkKeyword('fixum')) {
            return parseVariableDeclaration();
        }

        if (checkKeyword('functio') || checkKeyword('futura') || checkKeyword('cursor')) {
            return parseFunctionDeclaration();
        }

        if (checkKeyword('typus')) {
            return parseTypeAliasDeclaration();
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

        if (checkKeyword('cum')) {
            return parseWithStatement();
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

        if (checkKeyword('fac')) {
            // fac { } is block statement, fac x fit expr is lambda (expression)
            if (peek(1).type === 'LBRACE') {
                return parseFacBlockStatement();
            }
            // Lambda falls through to expression statement parsing
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
     *   importDecl := 'ex' IDENTIFIER 'importa' (identifierList | '*')
     *   identifierList := IDENTIFIER (',' IDENTIFIER)*
     *
     * Examples:
     *   ex norma importa scribe, lege
     *   ex norma importa *
     */
    function parseImportDeclaration(): ImportDeclaration {
        const position = peek().position;

        expectKeyword('ex', ParserErrorCode.ExpectedKeywordEx);

        const sourceToken = expect('IDENTIFIER', ParserErrorCode.ExpectedModuleName);
        const source = sourceToken.value;

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
     */
    function parseVariableDeclaration(): VariableDeclaration {
        const position = peek().position;
        const kind = peek().keyword as 'varia' | 'fixum';

        advance(); // varia or fixum

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
            const key = parseIdentifier();

            let value = key;

            // Check for rename: { nomen: localName }
            if (match('COLON')) {
                value = parseIdentifier();
            }

            properties.push({
                type: 'ObjectPatternProperty',
                key,
                value,
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
     *      Verb forms encode semantics: fit (sync), fiet (async), fiunt (generator), fient (async generator).
     */
    function parseFunctionDeclaration(): FunctionDeclaration {
        const position = peek().position;

        // Parse optional prefixes (futura = async, cursor = generator)
        let prefixAsync = false;
        let prefixGenerator = false;

        while (checkKeyword('futura') || checkKeyword('cursor')) {
            if (matchKeyword('futura')) {
                prefixAsync = true;
            }
            else if (matchKeyword('cursor')) {
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
        }
        else if (matchKeyword('fit')) {
            returnType = parseTypeAnnotation();
            verbAsync = false;
            verbGenerator = false;
        }
        else if (matchKeyword('fiet')) {
            returnType = parseTypeAnnotation();
            verbAsync = true;
            verbGenerator = false;
        }
        else if (matchKeyword('fiunt')) {
            returnType = parseTypeAnnotation();
            verbAsync = false;
            verbGenerator = true;
        }
        else if (matchKeyword('fient')) {
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

        return { type: 'FunctionDeclaration', name, params, returnType, body, async, generator, position };
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
     *   parameter := ('ad' | 'cum' | 'de' | 'in' | 'ex')? (typeAnnotation IDENTIFIER | IDENTIFIER)
     *
     * WHY: Type-first syntax: "textus name" or "ad textus recipientem"
     *      Prepositional prefixes indicate semantic roles:
     *      ad = toward/to, cum = with, de = from/concerning (borrowed),
     *      in = in/into (mutable), ex = from/out of
     *
     * EDGE: Preposition comes first (if present), then type (if present), then identifier.
     */
    function parseParameter(): Parameter {
        const position = peek().position;

        let preposition: string | undefined;

        if (isPreposition(peek())) {
            preposition = advance().keyword;
        }

        let typeAnnotation: TypeAnnotation | undefined;

        if (isTypeName(peek())) {
            typeAnnotation = parseTypeAnnotation();
        }

        const name = parseIdentifier();

        return { type: 'Parameter', name, typeAnnotation, preposition, position };
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
        const computedFields: ComputedFieldDeclaration[] = [];
        const methods: FunctionDeclaration[] = [];
        let constructorMethod: FunctionDeclaration | undefined;

        // True while there are unparsed members (not at '}' or EOF)
        const hasMoreMembers = () => !check('RBRACE') && !isAtEnd();

        while (hasMoreMembers()) {
            const member = parseGenusMember();

            switch (member.type) {
                case 'FieldDeclaration':
                    fields.push(member);
                    break;

                case 'ComputedFieldDeclaration':
                    computedFields.push(member);
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
            computedFields,
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
     *   fieldDecl := 'publicus'? 'generis'? typeAnnotation IDENTIFIER (':' expression)?
     *   methodDecl := 'publicus'? 'generis'? ('futura' | 'cursor')* 'functio' ...
     *
     * WHY: Distinguishes between fields and methods by looking for 'functio' keyword.
     */
    function parseGenusMember(): FieldDeclaration | ComputedFieldDeclaration | FunctionDeclaration {
        const position = peek().position;

        // Parse modifiers
        let isPublic = false;
        let isStatic = false;
        let prefixAsync = false;
        let prefixGenerator = false;

        if (matchKeyword('publicus')) {
            isPublic = true;
        }

        if (matchKeyword('generis')) {
            isStatic = true;
        }

        // Parse function modifiers (futura = async, cursor = generator)
        while (checkKeyword('futura') || checkKeyword('cursor')) {
            if (matchKeyword('futura')) {
                prefixAsync = true;
            }
            else if (matchKeyword('cursor')) {
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
            }
            else if (matchKeyword('fit')) {
                returnType = parseTypeAnnotation();
                verbAsync = false;
                verbGenerator = false;
            }
            else if (matchKeyword('fiet')) {
                returnType = parseTypeAnnotation();
                verbAsync = true;
                verbGenerator = false;
            }
            else if (matchKeyword('fiunt')) {
                returnType = parseTypeAnnotation();
                verbAsync = false;
                verbGenerator = true;
            }
            else if (matchKeyword('fient')) {
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

        // Otherwise it's a field: type name with ':' default or '=> computed value'
        const fieldType = parseTypeAnnotation();
        const fieldName = parseIdentifier();

        if (match('ARROW')) {
            const expression = parseExpression();

            return {
                type: 'ComputedFieldDeclaration',
                name: fieldName,
                fieldType,
                expression,
                isPublic,
                isStatic,
                position,
            };
        }

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
            isPublic,
            isStatic,
            position,
        };
    }

    // ---------------------------------------------------------------------------
    // Pactum (Interface) Declarations
    // ---------------------------------------------------------------------------

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

    function parsePactumMethod(): PactumMethod {
        const position = peek().position;

        // Parse optional prefixes (futura = async, cursor = generator)
        let prefixAsync = false;
        let prefixGenerator = false;

        while (checkKeyword('futura') || checkKeyword('cursor')) {
            if (matchKeyword('futura')) {
                prefixAsync = true;
            }
            else if (matchKeyword('cursor')) {
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
        }
        else if (matchKeyword('fit')) {
            returnType = parseTypeAnnotation();
            verbAsync = false;
            verbGenerator = false;
        }
        else if (matchKeyword('fiet')) {
            returnType = parseTypeAnnotation();
            verbAsync = true;
            verbGenerator = false;
        }
        else if (matchKeyword('fiunt')) {
            returnType = parseTypeAnnotation();
            verbAsync = false;
            verbGenerator = true;
        }
        else if (matchKeyword('fient')) {
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
     *      Literal style: si / aliter si / aliter
     *      Poetic style:  si / sin / secus
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
     * Parse for loop statement.
     *
     * GRAMMAR:
     *   forStmt := ('ex' | 'in') expression ('pro' | 'fit' | 'fiet') IDENTIFIER
     *              (blockStmt | 'ergo' statement) ('cape' IDENTIFIER blockStmt)?
     *
     * WHY: Source-first syntax consistent with import: 'ex norma importa x'
     *      'ex' for for-of (values from collection), 'in' for for-in (keys).
     *
     * The binding keyword uses verb conjugation to encode sync/async:
     *      'pro' = sync iteration (traditional preposition)
     *      'fit' = sync iteration (verb: "becomes")
     *      'fiet' = async iteration (verb: "will become")
     *
     * Examples:
     *   ex numeri pro n { ... }       // sync: from numbers, for each n
     *   ex numeri fit n { ... }       // sync: from numbers, n becomes each
     *   ex stream fiet chunk { ... }  // async: from stream, chunk will become each
     *   ex numeri pro n ergo run(n)   // one-liner
     *   in tabula pro clavis { ... }  // in table, for each key
     */
    function parseForStatement(): ForStatement {
        const position = peek().position;

        let kind: 'in' | 'ex' = 'ex';

        if (matchKeyword('ex')) {
            kind = 'ex';
        } else if (matchKeyword('in')) {
            kind = 'in';
        } else {
            error(ParserErrorCode.InvalidForLoopStart);
        }

        const iterable = parseExpression();

        // WHY: Accept 'pro', 'fit', or 'fiet' as the binding keyword
        // 'pro' and 'fit' = sync iteration, 'fiet' = async iteration
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

        return { type: 'ForStatement', kind, variable, iterable, body, async, catchClause, position };
    }

    /**
     * Parse with statement (context block).
     *
     * GRAMMAR:
     *   withStmt := 'cum' expression blockStmt
     *
     * WHY: 'cum' (with) establishes context for property access.
     *      Inside the block, bare identifier assignments become
     *      property assignments on the context object.
     *
     * Example:
     *   cum user {
     *       nomen = "Marcus"
     *   }
     */
    function parseWithStatement(): WithStatement {
        const position = peek().position;

        expectKeyword('cum', ParserErrorCode.ExpectedKeywordCum);

        const object = parseExpression();
        const body = parseBlockStatement();

        return { type: 'WithStatement', object, body, position };
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
     * WHY: Creates explicit scope boundary with optional error handling.
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
     * Parse logical OR expression.
     *
     * GRAMMAR:
     *   or := and ('||' and | 'aut' and)*
     *
     * PRECEDENCE: Lower than AND, higher than assignment.
     *
     * WHY: Latin 'aut' (or) supported alongside '||'.
     */
    function parseOr(): Expression {
        let left = parseAnd();

        while (match('OR') || matchKeyword('aut')) {
            const position = peek().position;
            const right = parseAnd();

            left = { type: 'BinaryExpression', operator: '||', left, right, position };
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

            if (match('EQUAL_EQUAL', 'BANG_EQUAL')) {
                operator = tokens[current - 1].value;
                position = tokens[current - 1].position;
            }
            else if (checkKeyword('non') && peek(1)?.type === 'KEYWORD' && peek(1)?.value.toLowerCase() === 'est') {
                // 'non est' maps to strict inequality (!==)
                position = peek().position;
                advance(); // consume 'non'
                advance(); // consume 'est'
                operator = '!==';
            }
            else if (matchKeyword('est')) {
                // 'est' maps to strict equality (===)
                operator = '===';
                position = tokens[current - 1].position;
            }
            else {
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
     *   range := additive ('..' additive ('per' additive)?)?
     *
     * PRECEDENCE: Lower than additive, higher than comparison.
     *
     * WHY: Range expressions provide concise numeric iteration.
     *      End is inclusive: 0..10 includes 10.
     *      Optional step via 'per' keyword.
     *
     * Examples:
     *   0..10           -> RangeExpression(0, 10)
     *   0..10 per 2     -> RangeExpression(0, 10, 2)
     *   start..end      -> RangeExpression(start, end)
     */
    function parseRange(): Expression {
        const start = parseAdditive();

        if (!match('DOT_DOT')) {
            return start;
        }

        const position = tokens[current - 1].position;
        const end = parseAdditive();

        let step: Expression | undefined;

        if (matchKeyword('per')) {
            step = parseAdditive();
        }

        return { type: 'RangeExpression', start, end, step, position } as RangeExpression;
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
     *   unary := ('!' | '-' | 'non' | 'nulla' | 'nonnulla' | 'negativum' | 'positivum' | 'cede' | 'novum') unary | call
     *
     * PRECEDENCE: Higher than binary operators, lower than call/member access.
     *
     * WHY: Latin 'non' (not), 'nulla' (none/empty), 'nonnulla' (some/non-empty),
     *      'negativum' (< 0), 'positivum' (> 0), 'cede' (await), 'novum' (new).
     */
    function parseUnary(): Expression {
        if (match('BANG') || matchKeyword('non')) {
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

            return { type: 'UnaryExpression', operator: 'negativum', argument, prefix: true, position };
        }

        if (matchKeyword('positivum')) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return { type: 'UnaryExpression', operator: 'positivum', argument, prefix: true, position };
        }

        if (matchKeyword('cede')) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return { type: 'AwaitExpression', argument, position };
        }

        if (matchKeyword('novum')) {
            return parseNewExpression();
        }

        return parseCall();
    }

    /**
     * Parse new expression (object construction).
     *
     * GRAMMAR:
     *   newExpr := 'novum' IDENTIFIER '(' argumentList ')'
     */
    function parseNewExpression(): NewExpression {
        const position = tokens[current - 1].position;
        const callee = parseIdentifier();

        let args: Expression[] = [];

        if (match('LPAREN')) {
            args = parseArgumentList();

            expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
        }

        let withExpression: ObjectExpression | undefined;

        if (matchKeyword('cum')) {
            const override = parsePrimary();

            if (override.type !== 'ObjectExpression') {
                error(ParserErrorCode.ExpectedObjectAfterCum);
            }

            withExpression = override as ObjectExpression;
        }

        return { type: 'NewExpression', callee, arguments: args, withExpression, position };
    }

    /**
     * Parse call expression with postfix operators.
     *
     * GRAMMAR:
     *   call := primary ('(' argumentList ')' | '.' IDENTIFIER | '[' expression ']')*
     *
     * PRECEDENCE: Highest (binds tightest after primary).
     *
     * WHY: Handles function calls, member access, and computed member access.
     *      Left-associative via loop (obj.a.b parsed as (obj.a).b).
     */
    function parseCall(): Expression {
        let expr = parsePrimary();

        while (true) {
            if (match('LPAREN')) {
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
     * Parse function call argument list.
     *
     * GRAMMAR:
     *   argumentList := (expression (',' expression)*)?
     */
    function parseArgumentList(): Expression[] {
        const args: Expression[] = [];

        if (check('RPAREN')) {
            return args;
        }

        do {
            args.push(parseExpression());
        } while (match('COMMA'));

        return args;
    }

    /**
     * Parse primary expression (literals, identifiers, grouped expressions).
     *
     * GRAMMAR:
     *   primary := IDENTIFIER | NUMBER | STRING | TEMPLATE_STRING
     *            | 'verum' | 'falsum' | 'nihil'
     *            | '(' (expression | arrowFunction) ')'
     *
     * PRECEDENCE: Highest (atoms of the language).
     *
     * WHY: Latin literals: verum (true), falsum (false), nihil (null).
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

        // Lambda expression: fac x fit expr, fac x, y fit expr, fac fit expr
        if (checkKeyword('fac')) {
            return parseFacExpression();
        }

        // Number literal
        if (check('NUMBER')) {
            const token = advance();
            const value = token.value.includes('.')
                ? parseFloat(token.value)
                : parseInt(token.value, 10);

            return { type: 'Literal', value, raw: token.value, position };
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
            const elements: Expression[] = [];

            if (!check('RBRACKET')) {
                do {
                    elements.push(parseExpression());
                } while (match('COMMA'));
            }

            expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);

            return { type: 'ArrayExpression', elements, position };
        }

        // Object literal
        if (match('LBRACE')) {
            const properties: ObjectProperty[] = [];

            if (!check('RBRACE')) {
                do {
                    const propPosition = peek().position;

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
     * Parse fac expression (lambda/anonymous function).
     *
     * GRAMMAR:
     *   facExpr := 'fac' params? ('fit' | 'fiet') expression
     *   params := IDENTIFIER (',' IDENTIFIER)*
     *
     * WHY: Latin 'fac' (do) + 'fit' (becomes) creates lambda syntax.
     *      'fiet' (will become) for async lambdas.
     *      Zero-param: fac fit expr -> () => expr
     *      Single param: fac x fit expr -> (x) => expr
     *      Multi param: fac x, y fit expr -> (x, y) => expr
     */
    function parseFacExpression(): FacExpression {
        const position = peek().position;

        expectKeyword('fac', ParserErrorCode.ExpectedKeywordFac);

        const params: Identifier[] = [];
        let async = false;

        // Check for immediate fit/fiet (zero-param lambda)
        if (!checkKeyword('fit') && !checkKeyword('fiet')) {
            // Parse parameters until we hit fit/fiet
            do {
                params.push(parseIdentifier());
            } while (match('COMMA'));
        }

        // Check for fit (sync) or fiet (async)
        if (matchKeyword('fiet')) {
            async = true;
        }
        else {
            expectKeyword('fit', ParserErrorCode.ExpectedKeywordFit);
        }

        // Parse the body expression
        const body = parseExpression();

        return { type: 'FacExpression', params, body, async, position };
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

        const token = expect('IDENTIFIER', ParserErrorCode.ExpectedTypeName);
        const name = token.value;

        let typeParameters: TypeParameter[] | undefined;

        if (match('LESS')) {
            typeParameters = [];

            do {
                if (check('NUMBER')) {
                    // Numeric parameter (e.g., numerus<32>)
                    const numToken = advance();
                    const value = numToken.value.includes('.')
                        ? parseFloat(numToken.value)
                        : parseInt(numToken.value, 10);

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
