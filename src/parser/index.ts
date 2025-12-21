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
 *   unary          := ('!' | '-' | 'non' | 'exspecta' | 'novum') unary | call
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
 * - functio, esto, fixum (declarations)
 * - si, dum, pro (control flow)
 * - redde, tempta (other statements)
 *
 * WHY: Allows parser to report multiple errors in one pass and produce
 *      partial AST for IDE/tooling use even with syntax errors.
 *
 * @module parser
 */

import type { Token, TokenType, Position } from "../tokenizer/types";
import type {
    Program,
    Statement,
    Expression,
    ImportDeclaration,
    VariableDeclaration,
    FunctionDeclaration,
    IfStatement,
    WhileStatement,
    ForStatement,
    ReturnStatement,
    BlockStatement,
    ThrowStatement,
    ExpressionStatement,
    Identifier,
    ArrowFunctionExpression,
    Parameter,
    TypeAnnotation,
    TypeParameter,
    ModifierParameter,
    CatchClause,
    NewExpression,
    TypeAliasDeclaration,
    Literal,
} from "./ast";
import { builtinTypes, typeModifiers } from "../lexicon/types-builtin";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Parser error with source location.
 *
 * INVARIANT: position always references valid source location.
 */
export interface ParserError {
    message: string
    position: Position
}

/**
 * Parser output containing AST and errors.
 *
 * INVARIANT: If parse succeeds, program is non-null and errors is empty.
 * INVARIANT: If parse fails catastrophically, program is null.
 * INVARIANT: Partial errors (recovered) have non-null program with non-empty errors.
 */
export interface ParserResult {
    program: Program | null
    errors: ParserError[]
}

// =============================================================================
// TYPE NAME LOOKUP
// =============================================================================

/**
 * Set of all builtin type names for quick lookup.
 *
 * PERF: Pre-computed Set enables O(1) type name checking.
 *
 * WHY: Used by isTypeName() to distinguish type names from regular identifiers
 *      in type-first syntax parsing (e.g., "fixum Textus nomen" vs "fixum nomen").
 */
const BUILTIN_TYPE_NAMES = new Set([
    // Primitives
    "Textus", "Numerus", "Bivalens", "Fractus", "Decimus",
    "Signum", "Incertum", "Nihil",
    // Collections
    "Lista", "Tabula", "Copia",
    // Structural
    "Res", "Functio", "Promissum", "Forsitan", "Fors",
    "Tempus", "Erratum", "Vacuum", "Quodlibet", "Ignotum",
    // Iteration
    "Cursor", "Fluxus", "FuturaCursor", "FuturusFluxus",
    // Systems
    "Indicium", "Refera",
    // Utility (TypeScript only)
    "Pars", "Totum", "Lectum", "Registrum",
    "Selectum", "Omissum", "Extractum", "Exclusum",
    "NonNihil", "Reditus", "Parametra",
]);

/**
 * Set of all type modifier names for quick lookup.
 *
 * WHY: Used to distinguish modifiers from regular type parameters.
 */
const TYPE_MODIFIER_NAMES = new Set([
    "Naturalis", "Proprius", "Alienus", "Mutabilis",
]);

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
        return peek().type === "EOF";
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
        return peek().type === "KEYWORD" && peek().keyword === keyword;
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
   * Expect specific token type or record error.
   *
   * ERROR RECOVERY: Records error but returns current token (possibly wrong type).
   *
   * @returns Matched token if found, current token if not
   */
    function expect(type: TokenType, message: string): Token {
        if (check(type)) {
            return advance();
        }

        const token = peek();

        errors.push({ message: `${message}, got '${token.value}'`, position: token.position });

        return token;
    }

    /**
   * Expect specific keyword or record error.
   *
   * ERROR RECOVERY: Records error but returns current token.
   */
    function expectKeyword(keyword: string, message: string): Token {
        if (checkKeyword(keyword)) {
            return advance();
        }

        const token = peek();

        errors.push({ message: `${message}, got '${token.value}'`, position: token.position });

        return token;
    }

    /**
   * Record error and throw for error recovery.
   *
   * WHY: Used in expression parsing where we can't easily recover locally.
   *      Caught by statement parser which calls synchronize().
   */
    function error(message: string): never {
        const token = peek();

        errors.push({ message, position: token.position });
        throw new Error(message);
    }

    // ---------------------------------------------------------------------------
    // Type Name Helpers
    // ---------------------------------------------------------------------------

    /**
   * Check if token is a builtin type name.
   *
   * WHY: Type-first syntax requires distinguishing type names from identifiers.
   *      "fixum Textus nomen" (type-first) vs "fixum nomen" (type inference).
   *
   * @returns true if token is an identifier and a known builtin type
   */
    function isTypeName(token: Token): boolean {
        return token.type === "IDENTIFIER" && BUILTIN_TYPE_NAMES.has(token.value);
    }

    /**
   * Check if token is a type modifier name.
   *
   * WHY: Used to distinguish modifier parameters from regular type parameters.
   *
   * @returns true if token is an identifier and a known type modifier
   */
    function isModifier(token: Token): boolean {
        return token.type === "IDENTIFIER" && TYPE_MODIFIER_NAMES.has(token.value);
    }

    /**
   * Check if token is a preposition used in parameters.
   *
   * WHY: Prepositions indicate semantic roles (ad, cum, in, ex).
   *
   * @returns true if token is a preposition keyword
   */
    function isPreposition(token: Token): boolean {
        return token.type === "KEYWORD" && ["ad", "cum", "in", "ex"].includes(token.keyword ?? "");
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
            }
            catch {
                synchronize();
            }
        }

        return { type: "Program", body, position };
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
            if (checkKeyword("functio") || checkKeyword("esto") || checkKeyword("fixum") ||
          checkKeyword("typus") || checkKeyword("si") || checkKeyword("dum") ||
          checkKeyword("pro") || checkKeyword("redde") || checkKeyword("tempta")) {
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
        if (checkKeyword("ex")) {
            return parseImportDeclaration();
        }

        if (checkKeyword("esto") || checkKeyword("fixum")) {
            return parseVariableDeclaration();
        }

        if (checkKeyword("functio") || checkKeyword("futura")) {
            return parseFunctionDeclaration();
        }

        if (checkKeyword("typus")) {
            return parseTypeAliasDeclaration();
        }

        if (checkKeyword("si")) {
            return parseIfStatement();
        }

        if (checkKeyword("dum")) {
            return parseWhileStatement();
        }

        if (checkKeyword("pro")) {
            return parseForStatement();
        }

        if (checkKeyword("redde")) {
            return parseReturnStatement();
        }

        if (checkKeyword("iace")) {
            return parseThrowStatement();
        }

        if (checkKeyword("tempta")) {
            return parseTryStatement();
        }

        if (check("LBRACE")) {
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

        expectKeyword("ex", "Expected 'ex'");

        const sourceToken = expect("IDENTIFIER", "Expected module name after 'ex'");
        const source = sourceToken.value;

        expectKeyword("importa", "Expected 'importa' after module name");

        if (match("STAR")) {
            return { type: "ImportDeclaration", source, specifiers: [], wildcard: true, position };
        }

        const specifiers: Identifier[] = [];

        do {
            specifiers.push(parseIdentifier());
        } while (match("COMMA"));

        return { type: "ImportDeclaration", source, specifiers, wildcard: false, position };
    }

    /**
   * Parse variable declaration.
   *
   * GRAMMAR:
   *   varDecl := ('esto' | 'fixum') (typeAnnotation IDENTIFIER | IDENTIFIER) ('=' expression)?
   *
   * WHY: Type-first syntax: "fixum Textus nomen = value" or "fixum nomen = value"
   *      Latin 'esto' (let it be) for mutable, 'fixum' (fixed) for immutable.
   *
   * EDGE: If next token after esto/fixum is a type name, parse type first.
   *       Otherwise, parse identifier (type inference case).
   */
    function parseVariableDeclaration(): VariableDeclaration {
        const position = peek().position;
        const kind = peek().keyword as "esto" | "fixum";

        advance(); // esto or fixum

        let typeAnnotation: TypeAnnotation | undefined;
        let name: Identifier;

        if (isTypeName(peek())) {
            typeAnnotation = parseTypeAnnotation();
            name = parseIdentifier();
        }
        else {
            name = parseIdentifier();
        }

        let init: Expression | undefined;

        if (match("EQUAL")) {
            init = parseExpression();
        }

        return { type: "VariableDeclaration", kind, name, typeAnnotation, init, position };
    }

    /**
   * Parse function declaration.
   *
   * GRAMMAR:
   *   funcDecl := 'futura'? 'functio' typeAnnotation? IDENTIFIER '(' paramList ')' blockStmt
   *
   * WHY: Type-first syntax: "functio Textus greet(Textus name)"
   *      'futura' prefix marks async functions (future/promise-based).
   *      Return type comes before function name (optional).
   *
   * EDGE: If token after functio is a type name, parse return type first.
   */
    function parseFunctionDeclaration(): FunctionDeclaration {
        const position = peek().position;
        let async = false;

        if (matchKeyword("futura")) {
            async = true;
        }

        expectKeyword("functio", "Expected 'functio'");

        let returnType: TypeAnnotation | undefined;

        if (isTypeName(peek())) {
            returnType = parseTypeAnnotation();
        }

        const name = parseIdentifier();

        expect("LPAREN", "Expected '(' after function name");
        const params = parseParameterList();

        expect("RPAREN", "Expected ')' after parameters");

        const body = parseBlockStatement();

        return { type: "FunctionDeclaration", name, params, returnType, body, async, position };
    }

    /**
   * Parse function parameter list.
   *
   * GRAMMAR:
   *   paramList := (parameter (',' parameter)*)?
   */
    function parseParameterList(): Parameter[] {
        const params: Parameter[] = [];

        if (check("RPAREN")) {
            return params;
        }

        do {
            params.push(parseParameter());
        } while (match("COMMA"));

        return params;
    }

    /**
   * Parse single function parameter.
   *
   * GRAMMAR:
   *   parameter := ('ad' | 'cum' | 'in' | 'ex')? (typeAnnotation IDENTIFIER | IDENTIFIER)
   *
   * WHY: Type-first syntax: "Textus name" or "ad Textus recipientem"
   *      Prepositional prefixes indicate semantic roles:
   *      ad = toward/to, cum = with, in = in/into, ex = from/out of
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

        return { type: "Parameter", name, typeAnnotation, preposition, position };
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
   *   typus ID = Textus
   *   typus UserID = Numerus<32, Naturalis>
   */
    function parseTypeAliasDeclaration(): TypeAliasDeclaration {
        const position = peek().position;

        expectKeyword("typus", "Expected 'typus'");

        const name = parseIdentifier();

        expect("EQUAL", "Expected '=' after type alias name");

        const typeAnnotation = parseTypeAnnotation();

        return { type: "TypeAliasDeclaration", name, typeAnnotation, position };
    }

    // ---------------------------------------------------------------------------
    // Control Flow Statements
    // ---------------------------------------------------------------------------

    /**
   * Parse if statement.
   *
   * GRAMMAR:
   *   ifStmt := 'si' expression blockStmt ('cape' IDENTIFIER blockStmt)? ('aliter' (ifStmt | blockStmt))?
   *
   * WHY: 'cape' (catch/seize) clause allows error handling within conditionals.
   *      'aliter' (otherwise) for else clause.
   */
    function parseIfStatement(): IfStatement {
        const position = peek().position;

        expectKeyword("si", "Expected 'si'");

        const test = parseExpression();
        const consequent = parseBlockStatement();

        // Check for cape (catch) clause
        let catchClause: CatchClause | undefined;

        if (checkKeyword("cape")) {
            catchClause = parseCatchClause();
        }

        // Check for alternate (aliter)
        let alternate: BlockStatement | IfStatement | undefined;

        if (matchKeyword("aliter")) {
            if (checkKeyword("si")) {
                alternate = parseIfStatement();
            }
            else {
                alternate = parseBlockStatement();
            }
        }

        return { type: "IfStatement", test, consequent, alternate, catchClause, position };
    }

    /**
   * Parse while loop statement.
   *
   * GRAMMAR:
   *   whileStmt := 'dum' expression blockStmt ('cape' IDENTIFIER blockStmt)?
   *
   * WHY: 'dum' (while/until) for while loops.
   */
    function parseWhileStatement(): WhileStatement {
        const position = peek().position;

        expectKeyword("dum", "Expected 'dum'");

        const test = parseExpression();
        const body = parseBlockStatement();

        let catchClause: CatchClause | undefined;

        if (checkKeyword("cape")) {
            catchClause = parseCatchClause();
        }

        return { type: "WhileStatement", test, body, catchClause, position };
    }

    /**
   * Parse for loop statement.
   *
   * GRAMMAR:
   *   forStmt := 'pro' IDENTIFIER ('in' | 'ex') expression blockStmt ('cape' IDENTIFIER blockStmt)?
   *
   * WHY: 'pro' (for/on behalf of) for for loops.
   *      'in' for for-in (keys), 'ex' for for-of (values from collection).
   */
    function parseForStatement(): ForStatement {
        const position = peek().position;

        expectKeyword("pro", "Expected 'pro'");

        const variable = parseIdentifier();

        let kind: "in" | "ex" = "in";

        if (matchKeyword("in")) {
            kind = "in";
        }
        else if (matchKeyword("ex")) {
            kind = "ex";
        }
        else {
            error("Expected 'in' or 'ex' after variable in for loop");
        }

        const iterable = parseExpression();
        const body = parseBlockStatement();

        let catchClause: CatchClause | undefined;

        if (checkKeyword("cape")) {
            catchClause = parseCatchClause();
        }

        return { type: "ForStatement", kind, variable, iterable, body, catchClause, position };
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

        expectKeyword("redde", "Expected 'redde'");

        let argument: Expression | undefined;

        if (!check("RBRACE") && !isAtEnd()) {
            argument = parseExpression();
        }

        return { type: "ReturnStatement", argument, position };
    }

    /**
   * Parse throw statement.
   *
   * GRAMMAR:
   *   throwStmt := 'iace' expression
   *
   * WHY: 'iace' (throw/hurl) for throwing exceptions.
   */
    function parseThrowStatement(): ThrowStatement {
        const position = peek().position;

        expectKeyword("iace", "Expected 'iace'");

        const argument = parseExpression();

        return { type: "ThrowStatement", argument, position };
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

        expectKeyword("tempta", "Expected 'tempta'");

        const block = parseBlockStatement();

        let handler: CatchClause | undefined;

        if (checkKeyword("cape")) {
            handler = parseCatchClause();
        }

        let finalizer: BlockStatement | undefined;

        if (matchKeyword("demum")) {
            finalizer = parseBlockStatement();
        }

        return { type: "TryStatement", block, handler, finalizer, position };
    }

    /**
   * Parse catch clause.
   *
   * GRAMMAR:
   *   catchClause := 'cape' IDENTIFIER blockStmt
   */
    function parseCatchClause(): CatchClause {
        const position = peek().position;

        expectKeyword("cape", "Expected 'cape'");

        const param = parseIdentifier();
        const body = parseBlockStatement();

        return { type: "CatchClause", param, body, position };
    }

    /**
   * Parse block statement.
   *
   * GRAMMAR:
   *   blockStmt := '{' statement* '}'
   */
    function parseBlockStatement(): BlockStatement {
        const position = peek().position;

        expect("LBRACE", "Expected '{'");

        const body: Statement[] = [];

        while (!check("RBRACE") && !isAtEnd()) {
            body.push(parseStatement());
        }

        expect("RBRACE", "Expected '}'");

        return { type: "BlockStatement", body, position };
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

        return { type: "ExpressionStatement", expression, position };
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
   *   assignment := or ('=' assignment)?
   *
   * PRECEDENCE: Lowest (right-associative via recursion).
   *
   * ERROR RECOVERY: Reports error if left side is not valid lvalue.
   */
    function parseAssignment(): Expression {
        const expr = parseOr();

        if (match("EQUAL")) {
            const position = peek().position;
            const value = parseAssignment();

            if (expr.type === "Identifier" || expr.type === "MemberExpression") {
                return { type: "AssignmentExpression", operator: "=", left: expr, right: value, position };
            }

            error("Invalid assignment target");
        }

        return expr;
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

        while (match("OR") || matchKeyword("aut")) {
            const position = peek().position;
            const right = parseAnd();

            left = { type: "BinaryExpression", operator: "||", left, right, position };
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

        while (match("AND") || matchKeyword("et")) {
            const position = peek().position;
            const right = parseEquality();

            left = { type: "BinaryExpression", operator: "&&", left, right, position };
        }

        return left;
    }

    /**
   * Parse equality expression.
   *
   * GRAMMAR:
   *   equality := comparison (('==' | '!=') comparison)*
   *
   * PRECEDENCE: Lower than comparison, higher than AND.
   */
    function parseEquality(): Expression {
        let left = parseComparison();

        while (match("EQUAL_EQUAL", "BANG_EQUAL")) {
            const operator = tokens[current - 1].value;
            const position = tokens[current - 1].position;
            const right = parseComparison();

            left = { type: "BinaryExpression", operator, left, right, position };
        }

        return left;
    }

    /**
   * Parse comparison expression.
   *
   * GRAMMAR:
   *   comparison := additive (('<' | '>' | '<=' | '>=') additive)*
   *
   * PRECEDENCE: Lower than additive, higher than equality.
   */
    function parseComparison(): Expression {
        let left = parseAdditive();

        while (match("LESS", "LESS_EQUAL", "GREATER", "GREATER_EQUAL")) {
            const operator = tokens[current - 1].value;
            const position = tokens[current - 1].position;
            const right = parseAdditive();

            left = { type: "BinaryExpression", operator, left, right, position };
        }

        return left;
    }

    /**
   * Parse additive expression.
   *
   * GRAMMAR:
   *   additive := multiplicative (('+' | '-') multiplicative)*
   *
   * PRECEDENCE: Lower than multiplicative, higher than comparison.
   */
    function parseAdditive(): Expression {
        let left = parseMultiplicative();

        while (match("PLUS", "MINUS")) {
            const operator = tokens[current - 1].value;
            const position = tokens[current - 1].position;
            const right = parseMultiplicative();

            left = { type: "BinaryExpression", operator, left, right, position };
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

        while (match("STAR", "SLASH", "PERCENT")) {
            const operator = tokens[current - 1].value;
            const position = tokens[current - 1].position;
            const right = parseUnary();

            left = { type: "BinaryExpression", operator, left, right, position };
        }

        return left;
    }

    /**
   * Parse unary expression.
   *
   * GRAMMAR:
   *   unary := ('!' | '-' | 'non' | 'exspecta' | 'novum') unary | call
   *
   * PRECEDENCE: Higher than binary operators, lower than call/member access.
   *
   * WHY: Latin 'non' (not), 'exspecta' (await), 'novum' (new).
   */
    function parseUnary(): Expression {
        if (match("BANG") || matchKeyword("non")) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return { type: "UnaryExpression", operator: "!", argument, prefix: true, position };
        }

        if (match("MINUS")) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return { type: "UnaryExpression", operator: "-", argument, prefix: true, position };
        }

        if (matchKeyword("exspecta")) {
            const position = tokens[current - 1].position;
            const argument = parseUnary();

            return { type: "AwaitExpression", argument, position };
        }

        if (matchKeyword("novum")) {
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

        expect("LPAREN", "Expected '(' after constructor");
        const args = parseArgumentList();

        expect("RPAREN", "Expected ')' after arguments");

        return { type: "NewExpression", callee, arguments: args, position };
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
            if (match("LPAREN")) {
                const position = tokens[current - 1].position;
                const args = parseArgumentList();

                expect("RPAREN", "Expected ')' after arguments");
                expr = { type: "CallExpression", callee: expr, arguments: args, position };
            }
            else if (match("DOT")) {
                const position = tokens[current - 1].position;
                const property = parseIdentifier();

                expr = { type: "MemberExpression", object: expr, property, computed: false, position };
            }
            else if (match("LBRACKET")) {
                const position = tokens[current - 1].position;
                const property = parseExpression() as Identifier;

                expect("RBRACKET", "Expected ']'");
                expr = { type: "MemberExpression", object: expr, property, computed: true, position };
            }
            else {
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

        if (check("RPAREN")) {
            return args;
        }

        do {
            args.push(parseExpression());
        } while (match("COMMA"));

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

        // Boolean literals
        if (matchKeyword("verum")) {
            return { type: "Literal", value: true, raw: "verum", position };
        }

        if (matchKeyword("falsum")) {
            return { type: "Literal", value: false, raw: "falsum", position };
        }

        if (matchKeyword("nihil")) {
            return { type: "Literal", value: null, raw: "nihil", position };
        }

        // Number literal
        if (check("NUMBER")) {
            const token = advance();
            const value = token.value.includes(".") ? parseFloat(token.value) : parseInt(token.value, 10);

            return { type: "Literal", value, raw: token.value, position };
        }

        // String literal
        if (check("STRING")) {
            const token = advance();

            return { type: "Literal", value: token.value, raw: `"${token.value}"`, position };
        }

        // Template string
        if (check("TEMPLATE_STRING")) {
            const token = advance();

            return { type: "TemplateLiteral", raw: token.value, position };
        }

        // Parenthesized expression or arrow function
        if (match("LPAREN")) {
            // Could be arrow function: (x) => ...
            // Or grouped expression: (a + b)
            // Peek ahead to see if this is an arrow function

            // Simple heuristic: if we see ) => then it's an arrow function
            const startPos = current;
            let parenDepth = 1;

            while (parenDepth > 0 && !isAtEnd()) {
                if (check("LPAREN")) {
                    parenDepth++;
                }

                if (check("RPAREN")) {
                    parenDepth--;
                }

                if (parenDepth > 0) {
                    advance();
                }
            }

            if (check("RPAREN")) {
                advance(); // consume )
                if (check("ARROW")) {
                    // It's an arrow function, backtrack and parse properly
                    current = startPos;

                    return parseArrowFunction(position);
                }
            }

            // Not an arrow function, backtrack and parse as grouped expression
            current = startPos;
            const expr = parseExpression();

            expect("RPAREN", "Expected ')'");

            return expr;
        }

        // Identifier
        if (check("IDENTIFIER")) {
            return parseIdentifier();
        }

        error(`Unexpected token: ${peek().value}`);
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

        expect("RPAREN", "Expected ')' after arrow function parameters");
        expect("ARROW", "Expected '=>'");

        let body: Expression | BlockStatement;

        if (check("LBRACE")) {
            body = parseBlockStatement();
        }
        else {
            body = parseExpression();
        }

        return { type: "ArrowFunctionExpression", params, body, async: false, position };
    }

    /**
   * Parse identifier.
   *
   * GRAMMAR:
   *   identifier := IDENTIFIER
   */
    function parseIdentifier(): Identifier {
        const token = expect("IDENTIFIER", "Expected identifier");

        return { type: "Identifier", name: token.value, position: token.position };
    }

    // =============================================================================
    // TYPE ANNOTATION PARSING
    // =============================================================================

    /**
   * Parse type annotation.
   *
   * GRAMMAR:
   *   typeAnnotation := IDENTIFIER typeParams? '?'? ('|' typeAnnotation)*
   *   typeParams := '<' typeParameter (',' typeParameter)* '>'
   *   typeParameter := typeAnnotation | NUMBER | MODIFIER
   *
   * WHY: Supports generics (Lista<Textus>), nullable (?), and union types (A | B).
   *      Type parameters can be types, numeric literals, or modifiers.
   *
   * EDGE: Numeric parameters for sized types (Numerus<32>).
   *       Modifier parameters for ownership/signedness (Numerus<Naturalis>).
   */
    function parseTypeAnnotation(): TypeAnnotation {
        const position = peek().position;
        const token = expect("IDENTIFIER", "Expected type name");
        const name = token.value;

        let typeParameters: TypeParameter[] | undefined;

        if (match("LESS")) {
            typeParameters = [];
            do {
                if (check("NUMBER")) {
                    const numToken = advance();
                    const value = numToken.value.includes(".") ? parseFloat(numToken.value) : parseInt(numToken.value, 10);

                    typeParameters.push({
                        type: "Literal",
                        value,
                        raw: numToken.value,
                        position: numToken.position,
                    });
                }
                else if (isModifier(peek())) {
                    const modToken = advance();

                    typeParameters.push({
                        type: "ModifierParameter",
                        name: modToken.value as "Naturalis" | "Proprius" | "Alienus" | "Mutabilis",
                        position: modToken.position,
                    });
                }
                else {
                    typeParameters.push(parseTypeAnnotation());
                }
            } while (match("COMMA"));

            expect("GREATER", "Expected '>' after type parameters");
        }

        let nullable = false;

        if (match("QUESTION")) {
            nullable = true;
        }

        let union: TypeAnnotation[] | undefined;

        if (check("PIPE")) {
            union = [{ type: "TypeAnnotation", name, typeParameters, nullable, position }];
            while (match("PIPE")) {
                union.push(parseTypeAnnotation());
            }

            return { type: "TypeAnnotation", name: "union", union, position };
        }

        return { type: "TypeAnnotation", name, typeParameters, nullable, position };
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
    }
    catch {
        return { program: null, errors };
    }
}

export * from "./ast";
