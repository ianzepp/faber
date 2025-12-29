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
    ImportaDeclaration,
    VariaDeclaration,
    FunctioDeclaration,
    GenusDeclaration,
    FieldDeclaration,
    PactumDeclaration,
    PactumMethod,
    DiscretioDeclaration,
    VariantDeclaration,
    VariantField,
    VariantCase,
    SiStatement,
    DumStatement,
    IteratioStatement,
    InStatement,
    EligeStatement,
    EligeCasus,
    DiscerneStatement,
    CustodiStatement,
    CustodiClause,
    AdfirmaStatement,
    ReddeStatement,
    RumpeStatement,
    PergeStatement,
    BlockStatement,
    IaceStatement,
    ScribeStatement,
    OutputLevel,
    ExpressionStatement,
    Identifier,
    EgoExpression,
    ArrowFunctionExpression,
    Parameter,
    TypeAnnotation,
    TypeParameter,
    TypeParameterDeclaration,
    CapeClause,
    NovumExpression,
    TypeAliasDeclaration,
    OrdoDeclaration,
    OrdoMember,
    Literal,
    RangeExpression,
    ConditionalExpression,
    ImportSpecifier,
    DestructureDeclaration,
    ObjectPattern,
    ObjectPatternProperty,
    ArrayPattern,
    ArrayPatternElement,
    ObjectExpression,
    ObjectProperty,
    SpreadElement,
    FacBlockStatement,
    LambdaExpression,
    QuaExpression,
    EstExpression,
    ProbandumStatement,
    ProbaStatement,
    ProbaModifier,
    CuraBlock,
    CuraTiming,
    CuratorKind,
    CuraStatement,
    AdStatement,
    AdBinding,
    AdBindingVerb,
    PraefixumExpression,
    CollectionDSLTransform,
    CollectionDSLExpression,
    ScriptumExpression,
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
        // EDGE: Tokenizer always produces at least an EOF token
        return tokens[current + offset] ?? tokens[tokens.length - 1]!;
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

        // EDGE: current is at least 1 after advancing, so current-1 is valid
        return tokens[current - 1]!;
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
     * ERROR RECOVERY: Records error and advances past the unexpected token to
     * prevent infinite loops. Returns a synthetic token of the expected type.
     *
     * WHY: If we don't advance, callers in loops (like parseObjectPattern) will
     * spin forever on the same unexpected token.
     *
     * @returns Matched token if found, synthetic token after advancing if not
     */
    function expect(type: TokenType, code: ParserErrorCode): Token {
        if (check(type)) {
            return advance();
        }

        const token = peek();

        reportError(code, `got '${token.value}'`);

        // Advance past the unexpected token to prevent infinite loops
        if (!isAtEnd()) {
            advance();
        }

        // Return synthetic token with expected type but actual position
        return { type, value: '', position: token.position };
    }

    /**
     * Expect specific keyword or record error.
     *
     * ERROR RECOVERY: Records error and advances past the unexpected token.
     */
    function expectKeyword(keyword: string, code: ParserErrorCode): Token {
        if (checkKeyword(keyword)) {
            return advance();
        }

        const token = peek();

        reportError(code, `got '${token.value}'`);

        // Advance past the unexpected token to prevent infinite loops
        if (!isAtEnd()) {
            advance();
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
    function isPreposition(token: Token): boolean {
        return token.type === 'KEYWORD' && ['de', 'in', 'ex'].includes(token.keyword ?? '');
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
                checkKeyword('tempta') ||
                checkKeyword('probandum') ||
                checkKeyword('proba')
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
                return parseImportaDeclaration();
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
            return parseVariaDeclaration();
        }

        if (checkKeyword('functio') || checkKeyword('futura') || checkKeyword('cursor')) {
            return parseFunctioDeclaration();
        }

        if (checkKeyword('typus')) {
            return parseTypeAliasDeclaration();
        }

        if (checkKeyword('ordo')) {
            return parseOrdoDeclaration();
        }

        if (checkKeyword('genus')) {
            return parseGenusDeclaration();
        }

        if (checkKeyword('pactum')) {
            return parsePactumDeclaration();
        }

        if (checkKeyword('discretio')) {
            return parseDiscretioDeclaration();
        }

        if (checkKeyword('si')) {
            return parseSiStatement();
        }

        if (checkKeyword('dum')) {
            return parseDumStatement();
        }

        if (checkKeyword('elige')) {
            return parseEligeStatement();
        }

        if (checkKeyword('discerne')) {
            return parseDiscerneStatement();
        }

        if (checkKeyword('custodi')) {
            return parseCustodiStatement();
        }

        if (checkKeyword('adfirma')) {
            return parseAdfirmaStatement();
        }

        if (checkKeyword('redde')) {
            return parseReddeStatement();
        }

        if (checkKeyword('rumpe')) {
            return parseRumpeStatement();
        }

        if (checkKeyword('perge')) {
            return parsePergeStatement();
        }

        if (checkKeyword('iace')) {
            return parseIaceStatement(false);
        }

        if (checkKeyword('mori')) {
            return parseIaceStatement(true);
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
            return parseTemptaStatement();
        }

        // fac { } cape { } is block with optional catch (see parseFacBlockStatement)
        if (checkKeyword('fac') && peek(1).type === 'LBRACE') {
            return parseFacBlockStatement();
        }

        // Test suite declaration: probandum "name" { ... }
        if (checkKeyword('probandum')) {
            return parseProbandumStatement();
        }

        // Individual test: proba "name" { ... }
        if (checkKeyword('proba')) {
            return parseProbaStatement();
        }

        // Dispatch statement
        // ad "target" (args) [binding]? [block]? [cape]?
        if (checkKeyword('ad')) {
            return parseAdStatement();
        }

        // Resource management / test setup-teardown
        // Two forms:
        //   cura ante/post [omnia]? { } - test setup/teardown (CuraBlock)
        //   cura [cede]? <expr> fit <id> { } [cape]? - resource management (CuraStatement)
        if (checkKeyword('cura')) {
            return parseCura();
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
     * Parse a single import/destructure specifier with optional alias.
     *
     * GRAMMAR:
     *   specifier := 'ceteri'? IDENTIFIER ('ut' IDENTIFIER)?
     *
     * WHY: Shared between imports and destructuring.
     *      'ceteri' (rest) is only valid in destructuring contexts.
     *      'ut' provides aliasing: nomen ut n
     *
     * Examples:
     *   scribe             -> imported=scribe, local=scribe
     *   scribe ut s        -> imported=scribe, local=s
     *   ceteri rest        -> imported=rest, local=rest, rest=true
     */
    function parseSpecifier(): ImportSpecifier {
        const position = peek().position;

        // Check for rest pattern: ceteri restName
        let rest = false;
        if (checkKeyword('ceteri')) {
            advance(); // consume 'ceteri'
            rest = true;
        }

        // WHY: Names can be keywords (ex norma importa scribe)
        const imported = parseIdentifierOrKeyword();
        let local = imported;

        // Check for alias: name ut alias
        if (checkKeyword('ut')) {
            advance(); // consume 'ut'
            local = parseIdentifierOrKeyword();
        }

        return {
            type: 'ImportSpecifier',
            imported,
            local,
            rest: rest || undefined,
            position,
        };
    }

    /**
     * Parse import declaration.
     *
     * GRAMMAR:
     *   importDecl := 'ex' (STRING | IDENTIFIER) 'importa' (specifierList | '*')
     *   specifierList := specifier (',' specifier)*
     *   specifier := IDENTIFIER ('ut' IDENTIFIER)?
     *
     * Examples:
     *   ex norma importa scribe, lege
     *   ex norma importa scribe ut s, lege ut l
     *   ex "norma/tempus" importa nunc, dormi
     *   ex norma importa *
     */
    function parseImportaDeclaration(): ImportaDeclaration {
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
            return { type: 'ImportaDeclaration', source, specifiers: [], wildcard: true, position };
        }

        const specifiers: ImportSpecifier[] = [];

        do {
            specifiers.push(parseSpecifier());
        } while (match('COMMA'));

        return { type: 'ImportaDeclaration', source, specifiers, wildcard: false, position };
    }

    /**
     * Parse variable declaration.
     *
     * GRAMMAR:
     *   varDecl := ('varia' | 'fixum' | 'figendum' | 'variandum') typeAnnotation? IDENTIFIER ('=' expression)?
     *   arrayDestruct := ('varia' | 'fixum' | 'figendum' | 'variandum') arrayPattern '=' expression
     *
     * WHY: Type-first syntax: "fixum textus nomen = value" or "fixum nomen = value"
     *      Latin 'varia' (let it be) for mutable, 'fixum' (fixed) for immutable.
     *
     * EDGE: If next token after varia/fixum is a type name, parse type first.
     *       Otherwise, parse identifier (type inference case).
     *
     * Async bindings (figendum/variandum) work the same way syntactically,
     * but imply await on the initializer during codegen.
     *
     * OBJECT DESTRUCTURING: Use ex-prefix syntax (produces DestructureDeclaration):
     *   ex persona fixum nomen, aetas
     *   ex persona fixum nomen ut n, aetas ut a
     *
     * ARRAY DESTRUCTURING: Uses bracket pattern (still VariaDeclaration):
     *   fixum [a, b, c] = coords
     *
     * NOT SUPPORTED (will produce parser errors):
     *   - JS spread: { ...rest }
     *   - Python unpack: { *rest } or { **rest }
     *   - TS-style annotation: fixum nomen: textus = "x" (use: fixum textus nomen = "x")
     *   - Brace object destructuring: fixum { a, b } = obj (use: ex obj fixum a, b)
     *   - Increment/decrement: x++, ++x, x--, --x
     *   - Compound assignment: x += 1, x -= 1, x *= 2, x /= 2
     */
    function parseVariaDeclaration(): VariaDeclaration {
        const position = peek().position;
        const kind = peek().keyword as 'varia' | 'fixum' | 'figendum' | 'variandum';

        advance(); // varia, fixum, figendum, or variandum

        let typeAnnotation: TypeAnnotation | undefined;
        let name: Identifier | ArrayPattern;

        // Array destructuring pattern: fixum [a, b] = arr
        if (check('LBRACKET')) {
            name = parseArrayPattern();
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

        return { type: 'VariaDeclaration', kind, name, typeAnnotation, init, position };
    }

    /**
     * Parse object destructuring pattern.
     *
     * GRAMMAR:
     *   objectPattern := '{' patternProperty (',' patternProperty)* '}'
     *   patternProperty := 'ceteri'? IDENTIFIER (':' IDENTIFIER)?
     *
     * Used by two destructuring syntaxes:
     *   1. Direct assignment: fixum { nomen, aetas } = user
     *   2. Ex-prefix (Latin): ex user fixum { nomen, aetas }
     *
     * Examples:
     *   { nomen, aetas }              // extract nomen and aetas
     *   { nomen: localName, aetas }   // rename nomen to localName
     *   { nomen, ceteri rest }        // extract nomen, collect rest
     *
     * NOT SUPPORTED (will produce parser errors):
     *   { ...rest }    // JS spread syntax
     *   { *rest }      // Python unpack syntax
     *   { **rest }     // Python kwargs syntax
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
     * Parse array destructuring pattern.
     *
     * GRAMMAR:
     *   arrayPattern := '[' arrayPatternElement (',' arrayPatternElement)* ']'
     *   arrayPatternElement := '_' | 'ceteri'? IDENTIFIER
     *
     * Used by two destructuring syntaxes:
     *   1. Direct assignment: fixum [a, b, c] = coords
     *   2. Ex-prefix (Latin): ex coords fixum [a, b, c]
     *
     * Examples:
     *   [a, b, c]                 // extract first three elements
     *   [first, ceteri rest]     // extract first, collect rest
     *   [_, second, _]           // skip first and third, extract second
     *
     * NOT SUPPORTED:
     *   [...rest]                // JS spread syntax
     *   [*rest]                  // Python unpack syntax
     */
    function parseArrayPattern(): ArrayPattern {
        const position = peek().position;

        expect('LBRACKET', ParserErrorCode.ExpectedOpeningBracket);

        const elements: ArrayPatternElement[] = [];

        // True while there are unparsed elements (not at ']' or EOF)
        const hasMoreElements = () => !check('RBRACKET') && !isAtEnd();

        while (hasMoreElements()) {
            const elemPosition = peek().position;

            // Check for rest pattern: ceteri restName
            let rest = false;
            if (checkKeyword('ceteri')) {
                advance(); // consume 'ceteri'
                rest = true;
            }

            // Check for skip pattern: _
            let skip = false;
            if (!rest && check('IDENTIFIER') && peek().value === '_') {
                advance(); // consume '_'
                skip = true;
                elements.push({
                    type: 'ArrayPatternElement',
                    name: { type: 'Identifier', name: '_', position: elemPosition },
                    skip: true,
                    position: elemPosition,
                });
            } else {
                // Regular binding or rest binding
                const name = parseIdentifier();
                elements.push({
                    type: 'ArrayPatternElement',
                    name,
                    rest,
                    position: elemPosition,
                });
            }

            if (!check('RBRACKET')) {
                expect('COMMA', ParserErrorCode.ExpectedComma);
            }
        }

        expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);

        return { type: 'ArrayPattern', elements, position };
    }

    /**
     * Parse function declaration.
     *
     * GRAMMAR:
     *   funcDecl := ('futura' | 'cursor')* 'functio' IDENTIFIER '(' paramList ')' returnClause? blockStmt
     *   paramList := (typeParamDecl ',')* (parameter (',' parameter)*)?
     *   typeParamDecl := 'prae' 'typus' IDENTIFIER
     *   returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
     *
     * WHY: Arrow syntax for return types: "functio greet(textus name) -> textus"
     *      'futura' prefix marks async functions (future/promise-based).
     *      'cursor' prefix marks generator functions (yield-based).
     *
     * TYPE PARAMETERS: 'prae typus T' declares compile-time type parameters.
     *      functio max(prae typus T, T a, T b) -> T { ... }
     *      Maps to: <T> (TS/Rust), TypeVar (Py), comptime T: type (Zig)
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
     *
     * NOT SUPPORTED (will produce parser errors):
     *   - TS-style param annotation: functio f(x: textus) (use: functio f(textus x))
     *   - TS-style return type: functio f(): textus (use: functio f() -> textus)
     *   - Trailing comma in params: functio f(a, b,)
     */
    function parseFunctioDeclaration(): FunctioDeclaration {
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

        // Parse type parameters and regular parameters
        const { typeParams, params } = parseTypeAndParameterList();

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
            type: 'FunctioDeclaration',
            name,
            typeParams: typeParams.length > 0 ? typeParams : undefined,
            params,
            returnType,
            body,
            async,
            generator,
            position,
        };
    }

    /**
     * Parse type parameters and regular parameters from function signature.
     *
     * GRAMMAR:
     *   paramList := (typeParamDecl ',')* (parameter (',' parameter)*)?
     *   typeParamDecl := 'prae' 'typus' IDENTIFIER
     *
     * WHY: Type parameters (prae typus T) must come first, followed by regular params.
     *      This matches the conventions of TypeScript, Rust, and Zig.
     *
     * Examples:
     *   (prae typus T, T a, T b)     -> typeParams=[T], params=[a, b]
     *   (prae typus T, prae typus U) -> typeParams=[T, U], params=[]
     *   (numerus a, numerus b)       -> typeParams=[], params=[a, b]
     */
    function parseTypeAndParameterList(): { typeParams: TypeParameterDeclaration[]; params: Parameter[] } {
        const typeParams: TypeParameterDeclaration[] = [];
        const params: Parameter[] = [];

        if (check('RPAREN')) {
            return { typeParams, params };
        }

        // Parse leading type parameters: prae typus T
        while (checkKeyword('prae')) {
            const typeParamPos = peek().position;
            advance(); // consume 'prae'
            expectKeyword('typus', ParserErrorCode.ExpectedKeywordTypus);
            const typeParamName = parseIdentifier();
            typeParams.push({
                type: 'TypeParameterDeclaration',
                name: typeParamName,
                position: typeParamPos,
            });

            // If no comma after type param, we're done with type params
            if (!match('COMMA')) {
                return { typeParams, params };
            }

            // Check if next is another type param or a regular param
            if (!checkKeyword('prae')) {
                break; // Switch to parsing regular params
            }
        }

        // Parse remaining regular parameters
        if (!check('RPAREN')) {
            do {
                params.push(parseParameter());
            } while (match('COMMA'));
        }

        return { typeParams, params };
    }

    /**
     * Parse function parameter list (simple form without type params).
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
     *   parameter := ('de' | 'in' | 'ex')? (typeAnnotation IDENTIFIER | IDENTIFIER)
     *
     * WHY: Type-first syntax: "textus name" or "de textus source"
     *      Prepositional prefixes indicate semantic roles:
     *      de = from/concerning (borrowed, read-only),
     *      in = in/into (mutable borrow),
     *      ex = from/out of (source)
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
     *   typus ConfigTypus = typus config    // typeof
     */
    function parseTypeAliasDeclaration(): TypeAliasDeclaration {
        const position = peek().position;

        expectKeyword('typus', ParserErrorCode.ExpectedKeywordTypus);

        const name = parseIdentifier();

        expect('EQUAL', ParserErrorCode.ExpectedEqual);

        // Check for typeof: `typus X = typus y`
        if (checkKeyword('typus')) {
            advance(); // consume 'typus'
            const typeofTarget = parseIdentifier();
            // WHY: When RHS is `typus identifier`, we extract the type of a value.
            // typeAnnotation is set to a placeholder; codegen uses typeofTarget.
            return {
                type: 'TypeAliasDeclaration',
                name,
                typeAnnotation: { type: 'TypeAnnotation', name: 'ignotum', position },
                typeofTarget,
                position,
            };
        }

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
     *   enumMember := IDENTIFIER ('=' ('-'? NUMBER | STRING))?
     *
     * WHY: Latin 'ordo' (order/rank) for enumerated constants.
     *
     * Examples:
     *   ordo color { rubrum, viridis, caeruleum }
     *   ordo status { pendens = 0, actum = 1, finitum = 2 }
     *   ordo offset { ante = -1, ad = 0, post = 1 }
     */
    function parseOrdoDeclaration(): OrdoDeclaration {
        const position = peek().position;

        expectKeyword('ordo', ParserErrorCode.ExpectedKeywordOrdo);

        const name = parseIdentifier();

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const members: OrdoMember[] = [];

        while (!check('RBRACE') && !isAtEnd()) {
            const memberPosition = peek().position;
            const memberName = parseIdentifier();

            let value: Literal | undefined;

            if (match('EQUAL')) {
                // Expect a literal value (number or string), with optional leading minus
                const valuePosition = peek().position;
                const isNegative = match('MINUS');
                const valueTok = advance();

                if (valueTok.type === 'NUMBER') {
                    const numValue = Number(valueTok.value);
                    value = {
                        type: 'Literal',
                        value: isNegative ? -numValue : numValue,
                        raw: isNegative ? `-${valueTok.value}` : valueTok.value,
                        position: valuePosition,
                    };
                } else if (valueTok.type === 'STRING') {
                    if (isNegative) {
                        errors.push({
                            code: ParserErrorCode.UnexpectedToken,
                            message: `Cannot use minus sign with string enum value`,
                            position: valuePosition,
                        });
                    }
                    value = {
                        type: 'Literal',
                        value: valueTok.value,
                        raw: valueTok.value,
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
                type: 'OrdoMember',
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

        return { type: 'OrdoDeclaration', name, members, position };
    }

    // ---------------------------------------------------------------------------
    // Discretio (Tagged Union) Declarations
    // ---------------------------------------------------------------------------

    /**
     * Parse discretio (tagged union) declaration.
     *
     * GRAMMAR:
     *   discretioDecl := 'discretio' IDENTIFIER typeParams? '{' variant (',' variant)* ','? '}'
     *   variant := IDENTIFIER ('{' variantFields '}')?
     *   variantFields := (typeAnnotation IDENTIFIER (',' typeAnnotation IDENTIFIER)*)?
     *
     * WHY: Latin 'discretio' (distinction) for tagged unions.
     *      Each variant has a compiler-managed tag for exhaustive pattern matching.
     *
     * Examples:
     *   discretio Event {
     *       Click { numerus x, numerus y }
     *       Keypress { textus key }
     *       Quit
     *   }
     *
     *   discretio Option<T> {
     *       Some { T value }
     *       None
     *   }
     */
    function parseDiscretioDeclaration(): DiscretioDeclaration {
        const position = peek().position;

        expectKeyword('discretio', ParserErrorCode.ExpectedKeywordDiscretio);

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

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const variants: VariantDeclaration[] = [];

        while (!check('RBRACE') && !isAtEnd()) {
            variants.push(parseVariantDeclaration());

            // Allow trailing comma or no separator between variants
            match('COMMA');
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return { type: 'DiscretioDeclaration', name, typeParameters, variants, position };
    }

    /**
     * Parse a single variant within a discretio.
     *
     * GRAMMAR:
     *   variant := IDENTIFIER ('{' variantFields '}')?
     *   variantFields := (typeAnnotation IDENTIFIER (',' typeAnnotation IDENTIFIER)*)?
     *
     * WHY: Variant names are capitalized by convention (like type names).
     *      Fields use type-first syntax like genus fields.
     *
     * Examples:
     *   Click { numerus x, numerus y }  -> fields with payload
     *   Quit                            -> unit variant (no payload)
     */
    function parseVariantDeclaration(): VariantDeclaration {
        const position = peek().position;

        const name = parseIdentifier();

        const fields: VariantField[] = [];

        // Check for payload: Variant { fields }
        if (match('LBRACE')) {
            // Parse fields until closing brace
            while (!check('RBRACE') && !isAtEnd()) {
                const fieldPosition = peek().position;
                const fieldType = parseTypeAnnotation();
                const fieldName = parseIdentifier();

                fields.push({
                    type: 'VariantField',
                    name: fieldName,
                    fieldType,
                    position: fieldPosition,
                });

                // Allow comma between fields
                if (!check('RBRACE')) {
                    match('COMMA');
                }
            }

            expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);
        }

        return { type: 'VariantDeclaration', name, fields, position };
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
        const methods: FunctioDeclaration[] = [];
        let constructorMethod: FunctioDeclaration | undefined;

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

                case 'FunctioDeclaration':
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
    function parseGenusMember(): FieldDeclaration | FunctioDeclaration {
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

            const method: FunctioDeclaration = {
                type: 'FunctioDeclaration',
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
    function parseSiStatement(skipSiKeyword = false): SiStatement {
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
        let catchClause: CapeClause | undefined;

        if (checkKeyword('cape')) {
            catchClause = parseCapeClause();
        }

        // Check for alternate: aliter/secus (else) or sin (else-if)
        let alternate: BlockStatement | SiStatement | undefined;

        if (matchKeyword('aliter') || matchKeyword('secus')) {
            if (checkKeyword('si')) {
                alternate = parseSiStatement();
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
            alternate = parseSiStatement(true);
        }

        return { type: 'SiStatement', test, consequent, alternate, catchClause, position };
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
    function parseDumStatement(): DumStatement {
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

        let catchClause: CapeClause | undefined;

        if (checkKeyword('cape')) {
            catchClause = parseCapeClause();
        }

        return { type: 'DumStatement', test, body, catchClause, position };
    }

    /**
     * Parse statement starting with 'ex'.
     *
     * Dispatches to either:
     * - IteratioStatement: ex SOURCE (pro|fit|fiet) VARIABLE { }
     * - VariaDeclaration (destructuring): ex SOURCE (fixum|varia|figendum|variandum) { props }
     *
     * GRAMMAR:
     *   exStmt := 'ex' expression (forBinding | destructBinding | arrayDestructBinding)
     *   forBinding := ('pro' | 'fit' | 'fiet') IDENTIFIER (blockStmt | 'ergo' statement) catchClause?
     *   destructBinding := ('fixum' | 'varia' | 'figendum' | 'variandum') specifierList
     *   arrayDestructBinding := ('fixum' | 'varia' | 'figendum' | 'variandum') arrayPattern
     *   specifierList := specifier (',' specifier)*
     *   specifier := 'ceteri'? IDENTIFIER ('ut' IDENTIFIER)?
     *
     * WHY: 'ex' (from/out of) introduces both iteration and extraction:
     *      - Iteration: ex items pro item { ... } (for each item from items)
     *      - Object destructuring: ex persona fixum nomen, aetas (extract properties)
     *      - Array destructuring: ex coords fixum [x, y, z] (extract by position)
     *      - Async destructuring: ex promise figendum result (await + extract)
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
     *   ex persona fixum nomen, aetas        // object destructuring
     *   ex persona fixum nomen ut n          // object destructuring with alias
     *   ex persona fixum nomen, ceteri rest  // object destructuring with rest
     *   ex coords fixum [x, y, z]            // array destructuring
     *   ex fetchData() figendum result       // async destructuring
     *
     * Collection DSL forms:
     *   ex items prima 5 pro item { }        // iteration with transforms
     *   ex items prima 5, ultima 2 pro x {}  // multiple transforms
     */
    function parseExStatement(): IteratioStatement | VariaDeclaration | DestructureDeclaration {
        const position = peek().position;

        expectKeyword('ex', ParserErrorCode.InvalidExDeStart);

        const source = parseExpression();

        // Dispatch based on what follows the expression
        if (checkKeyword('fixum') || checkKeyword('varia') || checkKeyword('figendum') || checkKeyword('variandum')) {
            const kind = advance().keyword as 'varia' | 'fixum' | 'figendum' | 'variandum';

            // Array destructuring: ex coords fixum [x, y, z]
            if (check('LBRACKET')) {
                const name = parseArrayPattern();
                return { type: 'VariaDeclaration', kind, name, init: source, position };
            }

            // Object destructuring: ex persona fixum nomen, aetas
            // WHY: Brace-less syntax matches import pattern: ex norma importa scribe, lege
            const specifiers: ImportSpecifier[] = [];
            do {
                specifiers.push(parseSpecifier());
            } while (match('COMMA'));

            return { type: 'DestructureDeclaration', source, kind, specifiers, position };
        }

        // Check for DSL transforms before pro/fit/fiet
        const transforms = parseDSLTransforms();

        // Now expect for-loop binding: ex source [transforms] pro/fit/fiet variable { }
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

        let catchClause: CapeClause | undefined;
        if (checkKeyword('cape')) {
            catchClause = parseCapeClause();
        }

        return {
            type: 'IteratioStatement',
            kind: 'ex',
            variable,
            iterable: source,
            body,
            async,
            catchClause,
            transforms: transforms.length > 0 ? transforms : undefined,
            position,
        };
    }

    /**
     * Check if current token is a DSL verb.
     *
     * WHY: DSL verbs are collection transform operations.
     */
    function isDSLVerb(): boolean {
        return checkKeyword('prima') || checkKeyword('ultima') || checkKeyword('summa');
    }

    /**
     * Parse collection DSL transforms.
     *
     * GRAMMAR:
     *   dslTransforms := dslTransform (',' dslTransform)*
     *   dslTransform := dslVerb expression?
     *   dslVerb := 'prima' | 'ultima' | 'summa'
     *
     * WHY: DSL provides concise syntax for common collection operations.
     *      Transforms chain with commas: prima 5, ultima 3
     *
     * Examples:
     *   prima 5           -> first 5 elements
     *   ultima 3          -> last 3 elements
     *   summa             -> sum (no argument)
     *   prima 5, ultima 2 -> first 5, then last 2 of those
     */
    function parseDSLTransforms(): CollectionDSLTransform[] {
        const transforms: CollectionDSLTransform[] = [];

        while (isDSLVerb()) {
            const transformPos = peek().position;
            const verb = advance().keyword!;

            // Check if this verb takes an argument
            // prima and ultima require numeric argument
            // summa takes no argument
            let argument: Expression | undefined;
            if (verb === 'prima' || verb === 'ultima') {
                // These verbs require a numeric argument
                argument = parseExpression();
            }
            // summa takes no argument

            transforms.push({
                type: 'CollectionDSLTransform',
                verb,
                argument,
                position: transformPos,
            });

            // Check for comma to continue chain
            if (!match('COMMA')) {
                break;
            }
        }

        return transforms;
    }

    /**
     * Parse collection DSL expression (expression context).
     *
     * GRAMMAR:
     *   dslExpr := 'ex' expression dslTransform (',' dslTransform)*
     *
     * WHY: When 'ex' appears in expression context with DSL verbs (not pro/fit/fiet),
     *      it creates a collection pipeline expression that can be assigned.
     *
     * Examples:
     *   fixum top5 = ex items prima 5
     *   fixum total = ex prices summa
     *   fixum result = ex items prima 10, ultima 3
     */
    function parseCollectionDSLExpression(): CollectionDSLExpression {
        const position = peek().position;

        expectKeyword('ex', ParserErrorCode.InvalidExDeStart);

        const source = parseExpression();

        // Parse DSL transforms (at least one required for expression form)
        const transforms = parseDSLTransforms();

        if (transforms.length === 0) {
            // No transforms means this shouldn't be parsed as DSL expression
            // This case shouldn't happen if called correctly from parsePrimary
            error(ParserErrorCode.UnexpectedToken, 'expected DSL verb after ex');
        }

        return {
            type: 'CollectionDSLExpression',
            source,
            transforms,
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
    function parseDeStatement(): IteratioStatement {
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

        let catchClause: CapeClause | undefined;
        if (checkKeyword('cape')) {
            catchClause = parseCapeClause();
        }

        return {
            type: 'IteratioStatement',
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
    function parseInStatement(): InStatement {
        const position = peek().position;

        expectKeyword('in', ParserErrorCode.ExpectedKeywordIn);

        const expr = parseExpression();
        const body = parseBlockStatement();

        return { type: 'InStatement', object: expr, body, position };
    }

    /**
     * Parse switch statement (value matching).
     *
     * GRAMMAR:
     *   eligeStmt := 'elige' expression '{' eligeCase* defaultCase? '}' catchClause?
     *   eligeCase := 'si' expression (blockStmt | 'ergo' expression)
     *   defaultCase := ('aliter' | 'secus') (blockStmt | statement)
     *
     * WHY: 'elige' (choose) for value-based switch.
     *      'ergo' (therefore) for one-liners, 'aliter'/'secus' (otherwise) for default.
     *      For variant matching on discretio types, use 'discerne' instead.
     *
     * Example:
     *   elige status {
     *       si "pending" ergo scribe("waiting")
     *       si "active" { processActive() }
     *       aliter iace "Unknown status"
     *   }
     */
    function parseEligeStatement(): EligeStatement {
        const position = peek().position;

        expectKeyword('elige', ParserErrorCode.ExpectedKeywordElige);

        const discriminant = parseExpression();

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const cases: EligeCasus[] = [];
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
                // Value case: si expression { ... }
                const casePosition = peek().position;

                expectKeyword('si', ParserErrorCode.ExpectedKeywordSi);

                const test = parseExpression();
                const consequent = parseSiBody();

                cases.push({ type: 'EligeCasus', test, consequent, position: casePosition });
            } else if (checkKeyword('aliter') || checkKeyword('secus')) {
                advance(); // consume aliter or secus

                defaultCase = parseAliterBody();
                break; // Default must be last
            } else {
                error(ParserErrorCode.InvalidEligeCaseStart);
                break;
            }
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        let catchClause: CapeClause | undefined;

        if (checkKeyword('cape')) {
            catchClause = parseCapeClause();
        }

        return { type: 'EligeStatement', discriminant, cases, defaultCase, catchClause, position };
    }

    /**
     * Parse variant matching statement (for discretio types).
     *
     * GRAMMAR:
     *   discerneStmt := 'discerne' expression '{' variantCase* '}'
     *   variantCase := 'si' IDENTIFIER ('pro' IDENTIFIER (',' IDENTIFIER)*)? blockStmt
     *
     * WHY: 'discerne' (distinguish!) pairs with 'discretio' (the tagged union type).
     *      Uses 'si' for conditional match, 'pro' to introduce bindings.
     *
     * Example:
     *   discerne event {
     *       si Click pro x, y { scribe "clicked at " + x + ", " + y }
     *       si Keypress pro key { scribe "pressed " + key }
     *       si Quit { mori "goodbye" }
     *   }
     */
    function parseDiscerneStatement(): DiscerneStatement {
        const position = peek().position;

        expectKeyword('discerne', ParserErrorCode.ExpectedKeywordDiscerne);

        const discriminant = parseExpression();

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const cases: VariantCase[] = [];

        // True while there are unparsed cases (not at '}' or EOF)
        const hasMoreCases = () => !check('RBRACE') && !isAtEnd();

        while (hasMoreCases()) {
            if (checkKeyword('si')) {
                // Variant case: si VariantName pro bindings { ... }
                const casePosition = peek().position;

                advance(); // consume 'si'

                const variant = parseIdentifier();

                // Parse optional bindings: pro x, y, z
                const bindings: Identifier[] = [];
                if (matchKeyword('pro')) {
                    do {
                        bindings.push(parseIdentifier());
                    } while (match('COMMA'));
                }

                const consequent = parseBlockStatement();

                cases.push({ type: 'VariantCase', variant, bindings, consequent, position: casePosition });
            } else {
                error(ParserErrorCode.InvalidDiscerneCaseStart);
                break;
            }
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return { type: 'DiscerneStatement', discriminant, cases, position };
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
    function parseCustodiStatement(): CustodiStatement {
        const position = peek().position;

        expectKeyword('custodi', ParserErrorCode.ExpectedKeywordCustodi);

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const clauses: CustodiClause[] = [];

        // True while there are unparsed clauses (not at '}' or EOF)
        const hasMoreClauses = () => !check('RBRACE') && !isAtEnd();

        while (hasMoreClauses()) {
            if (checkKeyword('si')) {
                const clausePosition = peek().position;

                expectKeyword('si', ParserErrorCode.ExpectedKeywordSi);

                const test = parseExpression();
                const consequent = parseBlockStatement();

                clauses.push({ type: 'CustodiClause', test, consequent, position: clausePosition });
            } else {
                error(ParserErrorCode.InvalidCustodiClauseStart);
                break;
            }
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return { type: 'CustodiStatement', clauses, position };
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
    function parseAdfirmaStatement(): AdfirmaStatement {
        const position = peek().position;

        expectKeyword('adfirma', ParserErrorCode.ExpectedKeywordAdfirma);

        const test = parseExpression();

        let message: Expression | undefined;

        if (match('COMMA')) {
            message = parseExpression();
        }

        return { type: 'AdfirmaStatement', test, message, position };
    }

    /**
     * Parse return statement.
     *
     * GRAMMAR:
     *   returnStmt := 'redde' expression?
     *
     * WHY: 'redde' (give back/return) for return statements.
     */
    function parseReddeStatement(): ReddeStatement {
        const position = peek().position;

        expectKeyword('redde', ParserErrorCode.ExpectedKeywordRedde);

        let argument: Expression | undefined;

        if (!check('RBRACE') && !isAtEnd()) {
            argument = parseExpression();
        }

        return { type: 'ReddeStatement', argument, position };
    }

    /**
     * Parse break statement.
     *
     * GRAMMAR:
     *   breakStmt := 'rumpe'
     *
     * WHY: 'rumpe' (break!) exits the innermost loop.
     */
    function parseRumpeStatement(): RumpeStatement {
        const position = peek().position;

        // Consume the 'rumpe' keyword (already validated by checkKeyword in parseStatement)
        advance();

        return { type: 'RumpeStatement', position };
    }

    /**
     * Parse continue statement.
     *
     * GRAMMAR:
     *   continueStmt := 'perge'
     *
     * WHY: 'perge' (continue/proceed!) skips to the next loop iteration.
     */
    function parsePergeStatement(): PergeStatement {
        const position = peek().position;

        // Consume the 'perge' keyword (already validated by checkKeyword in parseStatement)
        advance();

        return { type: 'PergeStatement', position };
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
    function parseIaceStatement(fatal: boolean): IaceStatement {
        const position = peek().position;

        // Consume the keyword (already validated by checkKeyword in parseStatement)
        advance();

        const argument = parseExpression();

        return { type: 'IaceStatement', fatal, argument, position };
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
    function parseTemptaStatement(): Statement {
        const position = peek().position;

        expectKeyword('tempta', ParserErrorCode.ExpectedKeywordTempta);

        const block = parseBlockStatement();

        let handler: CapeClause | undefined;

        if (checkKeyword('cape')) {
            handler = parseCapeClause();
        }

        let finalizer: BlockStatement | undefined;

        if (matchKeyword('demum')) {
            finalizer = parseBlockStatement();
        }

        return { type: 'TemptaStatement', block, handler, finalizer, position };
    }

    /**
     * Parse catch clause.
     *
     * GRAMMAR:
     *   catchClause := 'cape' IDENTIFIER blockStmt
     */
    function parseCapeClause(): CapeClause {
        const position = peek().position;

        expectKeyword('cape', ParserErrorCode.ExpectedKeywordCape);

        const param = parseIdentifier();
        const body = parseBlockStatement();

        return { type: 'CapeClause', param, body, position };
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

        let catchClause: CapeClause | undefined;

        if (checkKeyword('cape')) {
            catchClause = parseCapeClause();
        }

        return { type: 'FacBlockStatement', body, catchClause, position };
    }

    // ---------------------------------------------------------------------------
    // Test Declarations (Proba)
    // ---------------------------------------------------------------------------

    /**
     * Parse test suite declaration.
     *
     * GRAMMAR:
     *   probandumDecl := 'probandum' STRING '{' probandumBody '}'
     *   probandumBody := (curaBlock | probandumDecl | probaStmt)*
     *
     * WHY: Latin "probandum" (gerundive of probare) = "that which must be tested".
     *      Analogous to describe() in Jest/Vitest.
     *
     * Example:
     *   probandum "Tokenizer" {
     *       cura ante { lexer = init() }
     *       proba "parses numbers" { ... }
     *   }
     */
    function parseProbandumStatement(): ProbandumStatement {
        const position = peek().position;

        expectKeyword('probandum', ParserErrorCode.ExpectedKeywordProbandum);

        // Parse suite name string
        const nameToken = expect('STRING', ParserErrorCode.ExpectedString);
        const name = nameToken.value;

        expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

        const body: (CuraBlock | ProbandumStatement | ProbaStatement)[] = [];

        // True while there are unparsed members (not at '}' or EOF)
        const hasMoreMembers = () => !check('RBRACE') && !isAtEnd();

        while (hasMoreMembers()) {
            if (checkKeyword('probandum')) {
                body.push(parseProbandumStatement());
            } else if (checkKeyword('proba')) {
                body.push(parseProbaStatement());
            } else if (checkKeyword('cura')) {
                body.push(parseCuraBlock());
            } else {
                // Unknown token in probandum body
                reportError(ParserErrorCode.UnexpectedToken, `got '${peek().value}'`);
                advance(); // Skip to prevent infinite loop
            }
        }

        expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return { type: 'ProbandumStatement', name, body, position };
    }

    /**
     * Parse individual test case.
     *
     * GRAMMAR:
     *   probaStmt := 'proba' probaModifier? STRING blockStmt
     *   probaModifier := 'omitte' STRING | 'futurum' STRING
     *
     * WHY: Latin "proba" (imperative of probare) = "test!" / "prove!".
     *      Analogous to test() or it() in Jest/Vitest.
     *
     * Examples:
     *   proba "parses integers" { adfirma parse("42") est 42 }
     *   proba omitte "blocked by #42" { ... }
     *   proba futurum "needs async support" { ... }
     */
    function parseProbaStatement(): ProbaStatement {
        const position = peek().position;

        expectKeyword('proba', ParserErrorCode.ExpectedKeywordProba);

        // Check for modifier: omitte or futurum
        let modifier: ProbaModifier | undefined;
        let modifierReason: string | undefined;

        if (matchKeyword('omitte')) {
            modifier = 'omitte';
            const reasonToken = expect('STRING', ParserErrorCode.ExpectedString);
            modifierReason = reasonToken.value;
        } else if (matchKeyword('futurum')) {
            modifier = 'futurum';
            const reasonToken = expect('STRING', ParserErrorCode.ExpectedString);
            modifierReason = reasonToken.value;
        }

        // Parse test name string
        const nameToken = expect('STRING', ParserErrorCode.ExpectedString);
        const name = nameToken.value;

        // Parse test body
        const body = parseBlockStatement();

        return { type: 'ProbaStatement', name, modifier, modifierReason, body, position };
    }

    /**
     * Parse ad statement (dispatch).
     *
     * GRAMMAR:
     *   adStmt := 'ad' STRING '(' argumentList ')' adBinding? blockStmt? catchClause?
     *   adBinding := adBindingVerb typeAnnotation? 'pro' IDENTIFIER ('ut' IDENTIFIER)?
     *   adBindingVerb := 'fit' | 'fiet' | 'fiunt' | 'fient'
     *   argumentList := (expression (',' expression)*)?
     *
     * WHY: Latin 'ad' (to/toward) dispatches to named endpoints:
     *      - Stdlib syscalls: "fasciculus:lege", "console:log"
     *      - External packages: "hono/Hono"
     *      - Remote services: "https://api.example.com/users"
     *
     * Binding verbs encode sync/async and single/plural:
     *      - fit: sync, single ("it becomes")
     *      - fiet: async, single ("it will become")
     *      - fiunt: sync, plural ("they become")
     *      - fient: async, plural ("they will become")
     *
     * Examples:
     *   ad "console:log" ("hello")                           // fire-and-forget
     *   ad "fasciculus:lege" ("file.txt") fit textus pro c { }  // sync binding
     *   ad "http:get" (url) fiet Response pro r { }          // async binding
     *   ad "http:batch" (urls) fient Response[] pro rs { }   // async plural
     */
    function parseAdStatement(): AdStatement {
        const position = peek().position;

        expectKeyword('ad', ParserErrorCode.ExpectedKeywordAd);

        // Parse target string
        const targetToken = expect('STRING', ParserErrorCode.ExpectedString);
        const target = targetToken.value;

        // Parse argument list: (args...)
        expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);
        const args: (Expression | SpreadElement)[] = [];
        if (!check('RPAREN')) {
            do {
                if (matchKeyword('sparge')) {
                    const argument = parseExpression();
                    args.push({ type: 'SpreadElement', argument, position: argument.position });
                }
                else {
                    args.push(parseExpression());
                }
            } while (match('COMMA'));
        }
        expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

        // Parse optional binding clause
        // Binding starts with: fit | fiet | fiunt | fient | pro (type inference)
        let binding: AdBinding | undefined;
        if (checkKeyword('fit') || checkKeyword('fiet') || checkKeyword('fiunt') || checkKeyword('fient') || checkKeyword('pro')) {
            const bindingPosition = peek().position;

            // Parse binding verb (default to 'fit' if only 'pro' is used)
            let verb: AdBindingVerb = 'fit';
            if (matchKeyword('fit')) {
                verb = 'fit';
            }
            else if (matchKeyword('fiet')) {
                verb = 'fiet';
            }
            else if (matchKeyword('fiunt')) {
                verb = 'fiunt';
            }
            else if (matchKeyword('fient')) {
                verb = 'fient';
            }
            // If 'pro' is next without verb, verb defaults to 'fit'

            // Parse optional type annotation before 'pro'
            // Detection: if identifier before 'pro', it's a type
            let typeAnnotation: TypeAnnotation | undefined;
            if (check('IDENTIFIER') && !checkKeyword('pro')) {
                typeAnnotation = parseTypeAnnotation();
            }

            // Expect 'pro' keyword
            expectKeyword('pro', ParserErrorCode.ExpectedKeywordPro);

            // Parse binding name
            const name = parseIdentifier();

            // Parse optional alias: ut alias
            let alias: Identifier | undefined;
            if (matchKeyword('ut')) {
                alias = parseIdentifier();
            }

            binding = { type: 'AdBinding', verb, typeAnnotation, name, alias, position: bindingPosition };
        }

        // Parse optional body block
        let body: BlockStatement | undefined;
        if (check('LBRACE')) {
            body = parseBlockStatement();
        }

        // Parse optional catch clause
        let catchClause: CapeClause | undefined;
        if (checkKeyword('cape')) {
            catchClause = parseCapeClause();
        }

        return { type: 'AdStatement', target, arguments: args, binding, body, catchClause, position };
    }

    /**
     * Parse cura - dispatches to block (test) or statement (resource) form.
     *
     * Two forms:
     *   cura ante/post [omnia]? { } - test setup/teardown (CuraBlock)
     *   cura [cede]? <expr> fit <id> { } [cape]? - resource management (CuraStatement)
     *
     * WHY: 'cura' is unified keyword for resource management.
     *      Test form uses ante/post timing modifiers.
     *      Resource form uses 'fit' binding syntax.
     */
    function parseCura(): CuraBlock | CuraStatement {
        // Lookahead to determine which form
        // cura ante ... or cura post ... -> CuraBlock (test)
        // cura <anything else> -> CuraStatement (resource)
        if (checkKeyword('cura')) {
            const next = peek(1);
            if (next.keyword === 'ante' || (next.type === 'IDENTIFIER' && next.value === 'post')) {
                return parseCuraBlock();
            }
            return parseCuraStatement();
        }

        // Should not reach here, but handle gracefully
        return parseCuraStatement();
    }

    /**
     * Parse cura block (test setup-teardown).
     *
     * GRAMMAR:
     *   curaBlock := 'cura' ('ante' | 'post') 'omnia'? blockStmt
     *
     * WHY: Latin "cura" (care, concern) for test resource management.
     *      In test context:
     *        cura ante { } = beforeEach (care before each test)
     *        cura ante omnia { } = beforeAll (care before all tests)
     *        cura post { } = afterEach (care after each test)
     *        cura post omnia { } = afterAll (care after all tests)
     *
     * Examples:
     *   cura ante { lexer = init() }
     *   cura ante omnia { db = connect() }
     *   cura post { cleanup() }
     *   cura post omnia { db.close() }
     */
    function parseCuraBlock(): CuraBlock {
        const position = peek().position;

        expectKeyword('cura', ParserErrorCode.ExpectedKeywordCura);

        // Parse timing: ante or post
        // WHY: 'ante' is a keyword (also used for ranges), but 'post' is an identifier
        //      to avoid conflicts with method names like HTTP post()
        let timing: CuraTiming;
        if (matchKeyword('ante')) {
            timing = 'ante';
        } else if (check('IDENTIFIER') && peek().value === 'post') {
            advance();
            timing = 'post';
        } else {
            reportError(ParserErrorCode.ExpectedKeywordAnteOrPost, `got '${peek().value}'`);
            timing = 'ante'; // Default to ante for error recovery
        }

        // Check for 'omnia' modifier
        const omnia = matchKeyword('omnia');

        const body = parseBlockStatement();

        return { type: 'CuraBlock', timing, omnia, body, position };
    }

    /**
     * Parse cura statement (resource management).
     *
     * GRAMMAR:
     *   curaStmt := 'cura' curatorKind? expression? ('pro' | 'fit' | 'fiet') typeAnnotation? IDENTIFIER blockStmt catchClause?
     *   curatorKind := 'arena' | 'page'
     *
     * WHY: Latin "cura" (care) + binding verb for scoped resources.
     *      - pro: neutral binding ("for")
     *      - fit: sync binding ("it becomes")
     *      - fiet: async binding ("it will become")
     *      Curator kinds declare explicit allocator types (arena, page).
     *      Guarantees cleanup via solve() on scope exit.
     *
     * Examples:
     *   cura arena fit mem { ... }                    // arena allocator
     *   cura page fit mem { ... }                     // page allocator
     *   cura aperi("data.bin") fit fd { lege(fd) }   // generic resource
     *   cura connect(url) fiet conn { ... }          // async resource
     */
    function parseCuraStatement(): CuraStatement {
        const position = peek().position;

        expectKeyword('cura', ParserErrorCode.ExpectedKeywordCura);

        // Check for curator kind: arena or page
        // WHY: Explicit curator kinds declare allocator type for non-GC targets
        let curatorKind: CuratorKind | undefined;
        if (matchKeyword('arena')) {
            curatorKind = 'arena';
        } else if (matchKeyword('page')) {
            curatorKind = 'page';
        }

        // Parse optional resource acquisition expression
        // WHY: For arena/page, expression is optional (they create their own allocator)
        //      For generic resources, expression is required
        let resource: Expression | undefined;
        if (!checkKeyword('pro') && !checkKeyword('fit') && !checkKeyword('fiet')) {
            resource = parseExpression();
        }

        // Expect binding verb: pro, fit, or fiet
        // WHY: pro is neutral, fit is sync, fiet is async (matches lambda syntax)
        let async = false;
        if (matchKeyword('pro')) {
            async = false;
        } else if (matchKeyword('fit')) {
            async = false;
        } else if (matchKeyword('fiet')) {
            async = true;
        } else {
            reportError(ParserErrorCode.ExpectedKeywordFit, `expected 'pro', 'fit', or 'fiet', got '${peek().value}'`);
        }

        // Optional type annotation before binding identifier
        // WHY: Allows explicit typing: cura aperi("file") fit File fd { ... }
        // Detection: if two identifiers before '{', first is type, second is binding
        let typeAnnotation: TypeAnnotation | undefined;
        if (check('IDENTIFIER') && peek(1).type === 'IDENTIFIER') {
            typeAnnotation = parseTypeAnnotation();
        }

        // Parse binding identifier
        const binding = parseIdentifier();

        // Parse body block
        const body = parseBlockStatement();

        // Optional catch clause
        let catchClause: CapeClause | undefined;
        if (checkKeyword('cape')) {
            catchClause = parseCapeClause();
        }

        return { type: 'CuraStatement', curatorKind, resource, binding, typeAnnotation, async, body, catchClause, position };
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
     *   assignment := ternary (('=' | '+=' | '-=' | '*=' | '/=' | '&=' | '|=') assignment)?
     *
     * PRECEDENCE: Lowest (right-associative via recursion).
     *
     * ERROR RECOVERY: Reports error if left side is not valid lvalue.
     */
    function parseAssignment(): Expression {
        const expr = parseTernary();

        if (match('EQUAL', 'PLUS_EQUAL', 'MINUS_EQUAL', 'STAR_EQUAL', 'SLASH_EQUAL', 'PERCENT_EQUAL', 'AMPERSAND_EQUAL', 'PIPE_EQUAL')) {
            const operator = tokens[current - 1]!.value;
            const position = tokens[current - 1]!.position;
            const value = parseAssignment();

            if (expr.type === 'Identifier' || expr.type === 'MemberExpression') {
                return {
                    type: 'AssignmentExpression',
                    operator,
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
                    code: ParserErrorCode.GenericError,
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
     *   equality := comparison (('==' | '!=' | '===' | '!==' | 'est' | 'non' 'est') comparison)*
     *
     * PRECEDENCE: Lower than comparison, higher than AND.
     *
     * WHY: 'est' always means type check (instanceof/typeof).
     *      Use '===' or '!==' for value equality.
     *      Use 'nihil x' or 'nonnihil x' for null checks.
     */
    function parseEquality(): Expression {
        let left = parseComparison();

        while (true) {
            let operator: string;
            let position: Position;

            if (match('EQUAL_EQUAL', 'BANG_EQUAL', 'TRIPLE_EQUAL', 'BANG_DOUBLE_EQUAL')) {
                operator = tokens[current - 1]!.value;
                position = tokens[current - 1]!.position;
            } else if (checkKeyword('non') && peek(1)?.type === 'KEYWORD' && peek(1)?.value === 'est') {
                // 'non est' - negated type check
                position = peek().position;
                advance(); // consume 'non'
                advance(); // consume 'est'

                const targetType = parseTypeAnnotation();
                left = {
                    type: 'EstExpression',
                    expression: left,
                    targetType,
                    negated: true,
                    position,
                } as EstExpression;
                continue;
            } else if (matchKeyword('est')) {
                // 'est' - type check (instanceof/typeof)
                position = tokens[current - 1]!.position;

                const targetType = parseTypeAnnotation();
                left = {
                    type: 'EstExpression',
                    expression: left,
                    targetType,
                    negated: false,
                    position,
                } as EstExpression;
                continue;
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
     *   comparison := bitwiseOr (('<' | '>' | '<=' | '>=') bitwiseOr)*
     *
     * PRECEDENCE: Lower than bitwise OR, higher than equality.
     */
    function parseComparison(): Expression {
        let left = parseBitwiseOr();

        while (match('LESS', 'LESS_EQUAL', 'GREATER', 'GREATER_EQUAL')) {
            const operator = tokens[current - 1]!.value;
            const position = tokens[current - 1]!.position;
            const right = parseBitwiseOr();

            left = { type: 'BinaryExpression', operator, left, right, position };
        }

        return left;
    }

    /**
     * Parse bitwise OR expression.
     *
     * GRAMMAR:
     *   bitwiseOr := bitwiseXor ('|' bitwiseXor)*
     *
     * PRECEDENCE: Lower than bitwise XOR, higher than comparison.
     *
     * WHY: Bitwise precedence is above comparison (unlike C), so
     *      `flags & MASK == 0` parses as `(flags & MASK) == 0`.
     */
    function parseBitwiseOr(): Expression {
        let left = parseBitwiseXor();

        while (match('PIPE')) {
            const operator = tokens[current - 1]!.value;
            const position = tokens[current - 1]!.position;
            const right = parseBitwiseXor();

            left = { type: 'BinaryExpression', operator, left, right, position };
        }

        return left;
    }

    /**
     * Parse bitwise XOR expression.
     *
     * GRAMMAR:
     *   bitwiseXor := bitwiseAnd ('^' bitwiseAnd)*
     *
     * PRECEDENCE: Lower than bitwise AND, higher than bitwise OR.
     */
    function parseBitwiseXor(): Expression {
        let left = parseBitwiseAnd();

        while (match('CARET')) {
            const operator = tokens[current - 1]!.value;
            const position = tokens[current - 1]!.position;
            const right = parseBitwiseAnd();

            left = { type: 'BinaryExpression', operator, left, right, position };
        }

        return left;
    }

    /**
     * Parse bitwise AND expression.
     *
     * GRAMMAR:
     *   bitwiseAnd := shift ('&' shift)*
     *
     * PRECEDENCE: Lower than shift, higher than bitwise XOR.
     */
    function parseBitwiseAnd(): Expression {
        let left = parseShift();

        while (match('AMPERSAND')) {
            const operator = tokens[current - 1]!.value;
            const position = tokens[current - 1]!.position;
            const right = parseShift();

            left = { type: 'BinaryExpression', operator, left, right, position };
        }

        return left;
    }

    /**
     * Parse shift expression.
     *
     * GRAMMAR:
     *   shift := range (('<<' | '>>') range)*
     *
     * PRECEDENCE: Lower than range, higher than bitwise AND.
     */
    function parseShift(): Expression {
        let left = parseRange();

        while (match('LEFT_SHIFT', 'RIGHT_SHIFT')) {
            const operator = tokens[current - 1]!.value;
            const position = tokens[current - 1]!.position;
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

        const position = tokens[current - 1]!.position;
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
            const operator = tokens[current - 1]!.value;
            const position = tokens[current - 1]!.position;
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
     *
     * NOT SUPPORTED (will produce parser errors):
     *   - Exponentiation: a ** b (use function call or explicit multiplication)
     *   - Floor division: a // b (// starts a comment in Faber)
     *   - Increment/decrement: x++, ++x, x--, --x
     *   - Compound assignment: x += 1, x -= 1, x *= 2, x /= 2
     */
    function parseMultiplicative(): Expression {
        let left = parseUnary();

        while (match('STAR', 'SLASH', 'PERCENT')) {
            const operator = tokens[current - 1]!.value;
            const position = tokens[current - 1]!.position;
            const right = parseUnary();

            left = { type: 'BinaryExpression', operator, left, right, position };
        }

        return left;
    }

    /**
     * Parse unary expression.
     *
     * GRAMMAR:
     *   unary := ('!' | '-' | 'non' | 'nulla' | 'nonnulla' | 'nihil' | 'nonnihil' | 'negativum' | 'positivum' | 'cede' | 'novum') unary | cast
     *
     * PRECEDENCE: Higher than binary operators, lower than cast/call/member access.
     *
     * WHY: Latin 'non' (not), 'nulla' (none/empty), 'nonnulla' (some/non-empty),
     *      'nihil' (is null), 'nonnihil' (is not null),
     *      'negativum' (< 0), 'positivum' (> 0), 'cede' (await), 'novum' (new).
     */
    function parseUnary(): Expression {
        // WHY: Prefix ! is removed to make room for non-null assertion (postfix !.)
        //      Use 'non' for logical not: "si non x" instead of "si !x"
        if (matchKeyword('non')) {
            const position = tokens[current - 1]!.position;
            const argument = parseUnary();

            return { type: 'UnaryExpression', operator: '!', argument, prefix: true, position };
        }

        if (match('MINUS')) {
            const position = tokens[current - 1]!.position;
            const argument = parseUnary();

            return { type: 'UnaryExpression', operator: '-', argument, prefix: true, position };
        }

        if (match('TILDE')) {
            const position = tokens[current - 1]!.position;
            const argument = parseUnary();

            return { type: 'UnaryExpression', operator: '~', argument, prefix: true, position };
        }

        if (matchKeyword('nulla')) {
            const position = tokens[current - 1]!.position;
            const argument = parseUnary();

            return { type: 'UnaryExpression', operator: 'nulla', argument, prefix: true, position };
        }

        if (matchKeyword('nonnulla')) {
            const position = tokens[current - 1]!.position;
            const argument = parseUnary();

            return {
                type: 'UnaryExpression',
                operator: 'nonnulla',
                argument,
                prefix: true,
                position,
            };
        }

        // WHY: 'nihil x' checks if x is null, parallels 'nulla x' for emptiness
        //      But 'nihil' alone is the null literal (handled in parsePrimary)
        //      Check if followed by identifier or expression-starting keyword
        if (checkKeyword('nihil')) {
            const next = peek(1);
            const isUnaryOperand =
                next?.type === 'IDENTIFIER' ||
                (next?.type === 'KEYWORD' &&
                    ['verum', 'falsum', 'nihil', 'ego', 'non', 'nulla', 'nonnulla', 'negativum', 'positivum', 'novum', 'cede'].includes(next.value));
            if (isUnaryOperand) {
                advance(); // consume 'nihil'
                const position = tokens[current - 1]!.position;
                const argument = parseUnary();

                return { type: 'UnaryExpression', operator: 'nihil', argument, prefix: true, position };
            }
        }

        // WHY: 'nonnihil x' checks if x is not null (always unary, no ambiguity)
        if (matchKeyword('nonnihil')) {
            const position = tokens[current - 1]!.position;
            const argument = parseUnary();

            return { type: 'UnaryExpression', operator: 'nonnihil', argument, prefix: true, position };
        }

        if (matchKeyword('negativum')) {
            const position = tokens[current - 1]!.position;
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
            const position = tokens[current - 1]!.position;
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
            const position = tokens[current - 1]!.position;
            const argument = parseUnary();

            return { type: 'CedeExpression', argument, position };
        }

        if (matchKeyword('novum')) {
            return parseNovumExpression();
        }

        if (matchKeyword('praefixum')) {
            return parsePraefixumExpression();
        }

        if (matchKeyword('scriptum')) {
            return parseScriptumExpression();
        }

        return parseQuaExpression();
    }

    /**
     * Parse compile-time evaluation expression.
     *
     * GRAMMAR:
     *   praefixumExpr := 'praefixum' (blockStmt | '(' expression ')')
     *
     * WHY: Latin 'praefixum' (pre-fixed) extends fixum vocabulary.
     *      Block form: praefixum { ... } for multi-statement computation
     *      Expression form: praefixum(expr) for simple expressions
     *
     * TARGET SUPPORT:
     *   Zig:    comptime { } or comptime (expr)
     *   C++:    constexpr
     *   Rust:   const (in const context)
     *   TS/Py:  Semantic error - not supported
     *
     * Examples:
     *   fixum size = praefixum(256 * 4)
     *   fixum table = praefixum {
     *       varia result = []
     *       ex 0..10 pro i { result.adde(i * i) }
     *       redde result
     *   }
     */
    function parsePraefixumExpression(): PraefixumExpression {
        const position = tokens[current - 1]!.position; // Position of 'praefixum' we just consumed

        let body: Expression | BlockStatement;

        if (check('LBRACE')) {
            // Block form: praefixum { ... }
            body = parseBlockStatement();
        } else if (match('LPAREN')) {
            // Expression form: praefixum(expr)
            body = parseExpression();
            expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
        } else {
            // Error: expected { or (
            error(ParserErrorCode.ExpectedOpeningBraceOrParen, `got '${peek().value}'`);
        }

        return { type: 'PraefixumExpression', body, position };
    }

    /**
     * Parse format string expression.
     *
     * GRAMMAR:
     *   scriptumExpr := 'scriptum' '(' STRING (',' expression)* ')'
     *
     * WHY: "scriptum" (that which has been written) is the perfect passive participle
     *      of scribere. While scribe outputs to console, scriptum returns a formatted string.
     *
     * WHY: Format string is passed through verbatim to target. User must use target-appropriate
     *      placeholders ({} for Zig/Rust, %s/%d for C++, etc.). Faber does not translate.
     *
     * Examples:
     *   scriptum("Hello, {}", name)
     *   scriptum("{} + {} = {}", a, b, a + b)
     */
    function parseScriptumExpression(): ScriptumExpression {
        const position = tokens[current - 1]!.position; // Position of 'scriptum' we just consumed

        expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);

        // First argument must be the format string literal
        const formatToken = peek();
        if (formatToken.type !== 'STRING') {
            error(ParserErrorCode.ExpectedString, 'scriptum requires a format string literal as first argument');
        }
        advance();
        const format: Literal = {
            type: 'Literal',
            value: formatToken.value,
            raw: `"${formatToken.value}"`,
            position: formatToken.position,
        };

        // Parse remaining arguments
        const args: Expression[] = [];
        while (match('COMMA')) {
            args.push(parseExpression());
        }

        expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

        return { type: 'ScriptumExpression', format, arguments: args, position };
    }

    /**
     * Parse type cast expression.
     *
     * GRAMMAR:
     *   castExpr := call ('qua' typeAnnotation)*
     *
     * PRECEDENCE: Between unary and call. This means:
     *   -x qua T     parses as -(x qua T)    — unary binds looser
     *   x.y qua T    parses as (x.y) qua T   — member access binds tighter
     *   x qua A qua B parses as (x qua A) qua B — left-associative
     *
     * WHY: Latin 'qua' (as, in the capacity of) for type assertions.
     *      Compile-time only — no runtime checking. Maps to:
     *      - TypeScript: x as T
     *      - Python: x (ignored, dynamic typing)
     *      - Zig: @as(T, x)
     *      - Rust: x as T
     *      - C++: static_cast<T>(x)
     */
    function parseQuaExpression(): Expression {
        let expr = parseCall();

        while (matchKeyword('qua')) {
            const position = tokens[current - 1]!.position;
            const targetType = parseTypeAnnotation();

            expr = {
                type: 'QuaExpression',
                expression: expr,
                targetType,
                position,
            } as QuaExpression;
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
    function parseNovumExpression(): NovumExpression {
        const position = tokens[current - 1]!.position;
        const callee = parseIdentifier();

        let args: (Expression | SpreadElement)[] = [];

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

        return { type: 'NovumExpression', callee, arguments: args, withExpression, position };
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
                    // WHY: Allow keywords as property names (e.g., items.omitte)
                    const property = parseIdentifierOrKeyword();
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
                    // WHY: Allow keywords as property names (e.g., items.omitte)
                    const property = parseIdentifierOrKeyword();
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
                const position = tokens[current - 1]!.position;
                const args = parseArgumentList();

                expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

                expr = { type: 'CallExpression', callee: expr, arguments: args, position };
            } else if (match('DOT')) {
                const position = tokens[current - 1]!.position;
                // WHY: Allow keywords as property names (e.g., items.omitte)
                const property = parseIdentifierOrKeyword();

                expr = {
                    type: 'MemberExpression',
                    object: expr,
                    property,
                    computed: false,
                    position,
                };
            } else if (match('LBRACKET')) {
                const position = tokens[current - 1]!.position;
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
            const thisExpr: EgoExpression = { type: 'EgoExpression', position };
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
        // Also: fit x: expr (sync, explicit), fiet x: expr (async)
        if (checkKeyword('pro') || checkKeyword('fit')) {
            return parseLambdaExpression(false);
        }

        if (checkKeyword('fiet')) {
            return parseLambdaExpression(true);
        }

        // Collection DSL expression: ex items prima 5
        // WHY: When ex appears in expression context with DSL verbs,
        //      it's a collection pipeline expression (no iteration)
        if (checkKeyword('ex')) {
            return parseCollectionDSLExpression();
        }

        // Number literal (decimal or hex)
        if (check('NUMBER')) {
            const token = advance();
            // WHY: Number() handles both decimal and hex (0x) prefixes correctly
            const value = Number(token.value);

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
                } while (match('COMMA') && !check('RBRACKET'));
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
     * Parse lambda expression (anonymous function).
     *
     * GRAMMAR:
     *   lambdaExpr := ('pro' | 'fit' | 'fiet') params? ((':' | 'redde') expression | blockStmt)
     *   params := IDENTIFIER (',' IDENTIFIER)*
     *
     * Three keyword forms with different semantics:
     *   - 'pro' / 'fit': sync lambda (pro is casual, fit is explicit verb form)
     *   - 'fiet': async lambda (future tense verb form)
     *
     * The ':' and 'redde' forms are INTERCHANGEABLE - use whichever reads better:
     *      pro x: x * 2        ≡  pro x redde x * 2      -> (x) => x * 2
     *      fiet x: expr        ≡  fiet x redde expr      -> async (x) => expr
     *      pro: 42             ≡  pro redde 42           -> () => 42
     *      pro x, y: x + y     ≡  pro x, y redde x + y   -> (x, y) => x + y
     *
     * Block form (for multi-statement bodies):
     *      pro x { redde x * 2 }     -> (x) => { return x * 2; }
     *      fiet c { cede fetch() }   -> async (c) => { await fetch(); }
     *
     * Style guidance: Use ':' for short expressions, 'redde' for clarity in complex cases.
     */
    function parseLambdaExpression(async: boolean): LambdaExpression {
        const position = peek().position;

        // Consume the keyword (pro, fit, or fiet)
        advance();

        const params: Identifier[] = [];

        // Check for immediate redde, :, ->, or { (zero-param lambda)
        if (!checkKeyword('redde') && !check('COLON') && !check('LBRACE') && !check('THIN_ARROW')) {
            // Parse parameters until we hit redde, :, ->, or {
            do {
                params.push(parseIdentifier());
            } while (match('COMMA'));
        }

        // Parse optional return type annotation: -> Type
        let returnType: TypeAnnotation | undefined;

        if (match('THIN_ARROW')) {
            returnType = parseTypeAnnotation();
        }

        let body: Expression | BlockStatement;

        if (check('LBRACE')) {
            // Block form: pro x { ... } or pro x -> T { ... }
            body = parseBlockStatement();
        } else if (match('COLON')) {
            // Expression shorthand: pro x: expr or pro x -> T: expr
            body = parseExpression();
        } else {
            // Expression form: pro x redde expr or pro x -> T redde expr
            expectKeyword('redde', ParserErrorCode.ExpectedKeywordRedde);
            body = parseExpression();
        }

        return { type: 'LambdaExpression', params, returnType, body, async, position };
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

    /**
     * Parse identifier or keyword as a name.
     *
     * WHY: Import specifiers can be keywords (ex norma importa scribe).
     *      In this context, 'scribe' is a valid name, not a statement keyword.
     */
    function parseIdentifierOrKeyword(): Identifier {
        const token = peek();

        if (token.type === 'IDENTIFIER' || token.type === 'KEYWORD') {
            advance();
            return { type: 'Identifier', name: token.value, position: token.position };
        }

        // Fall back to normal identifier parsing (will report error and advance)
        return parseIdentifier();
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
     *   typeAnnotation := ('de' | 'in')? IDENTIFIER typeParams? '?'? arrayBrackets*
     *   typeParams := '<' typeParameter (',' typeParameter)* '>'
     *   typeParameter := typeAnnotation | NUMBER | MODIFIER
     *   arrayBrackets := '[]' '?'?
     *
     * WHY: Supports generics (lista<textus>), nullable (?), union types (unio<A, B>),
     *      and array shorthand (numerus[] desugars to lista<numerus>).
     *
     * EDGE: Numeric parameters for sized types (numerus<32>).
     *       Modifier parameters for ownership/signedness (numerus<Naturalis>).
     *       Array shorthand preserves source form via arrayShorthand flag.
     *       Borrow prepositions (de/in) for systems targets (Rust/Zig).
     *       Union types use unio<A, B> syntax (pipe reserved for bitwise OR).
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

        // Handle unio<A, B> -> union type with type parameters as union members
        // WHY: unio<A, B> syntax frees pipe for bitwise OR
        if (name === 'unio' && typeParameters && typeParameters.length > 0) {
            // Convert type parameters to union members (must all be TypeAnnotations)
            const union: TypeAnnotation[] = typeParameters.filter((p): p is TypeAnnotation => p.type === 'TypeAnnotation');

            return {
                type: 'TypeAnnotation',
                name: 'union',
                union,
                nullable,
                preposition,
                position,
            };
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
