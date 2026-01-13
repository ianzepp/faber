/**
 * Unary Expression Parsing
 *
 * Handles parsing of unary operators and prefix expressions.
 *
 * GRAMMAR: See `EBNF.md` "Expressions" section
 *
 * @module parser/expressions/unary
 */

import type { Resolver } from '../resolver';
import type {
    Expression,
    UnaryExpression,
    CedeExpression,
    NovumExpression,
    FingeExpression,
    PraefixumExpression,
    ScriptumExpression,
    LegeExpression,
    Literal,
    BlockStatement,
} from '../ast';
import { ParserErrorCode } from '../errors';

// =============================================================================
// UNARY EXPRESSION PARSING
// =============================================================================

/**
 * Parse unary expression.
 *
 * GRAMMAR: unary (see `EBNF.md` "Expressions")
 *
 * PRECEDENCE: Higher than binary operators, lower than cast/call/member access.
 *
 * WHY: Latin 'non' (not), 'nulla' (none/empty), 'nonnulla' (some/non-empty),
 *      'nihil' (is null), 'nonnihil' (is not null),
 *      'negativum' (< 0), 'positivum' (> 0),
 *      'cede' (await), 'novum' (new), 'finge' (form variant).
 */
export function parseUnary(r: Resolver): Expression {
    const ctx = r.ctx();

    // WHY: Prefix ! is removed to make room for non-null assertion (postfix !.)
    //      Use 'non' for logical not: "si non x" instead of "si !x"
    if (ctx.matchKeyword('non')) {
        const position = ctx.tokens[ctx.current - 1]!.position;
        const argument = parseUnary(r);

        return { type: 'UnaryExpression', operator: '!', argument, prefix: true, position };
    }

    if (ctx.match('MINUS')) {
        const position = ctx.tokens[ctx.current - 1]!.position;
        const argument = parseUnary(r);

        return { type: 'UnaryExpression', operator: '-', argument, prefix: true, position };
    }

    if (ctx.match('TILDE')) {
        const position = ctx.tokens[ctx.current - 1]!.position;
        const argument = parseUnary(r);

        return { type: 'UnaryExpression', operator: '~', argument, prefix: true, position };
    }

    if (ctx.matchKeyword('nulla')) {
        const position = ctx.tokens[ctx.current - 1]!.position;
        const argument = parseUnary(r);

        return { type: 'UnaryExpression', operator: 'nulla', argument, prefix: true, position };
    }

    if (ctx.matchKeyword('nonnulla')) {
        const position = ctx.tokens[ctx.current - 1]!.position;
        const argument = parseUnary(r);

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
    //      Check if followed by identifier or expression-starting keyword ON SAME LINE
    if (ctx.checkKeyword('nihil')) {
        const curr = ctx.peek();
        const next = ctx.peek(1);
        const sameLine = next && next.position.line === curr.position.line;
        const isUnaryOperand =
            sameLine &&
            (next?.type === 'IDENTIFIER' ||
                (next?.type === 'KEYWORD' &&
                    ['verum', 'falsum', 'nihil', 'ego', 'non', 'nulla', 'nonnulla', 'negativum', 'positivum', 'novum', 'cede'].includes(
                        next.value,
                    )));
        if (isUnaryOperand) {
            ctx.advance(); // consume 'nihil'
            const position = ctx.tokens[ctx.current - 1]!.position;
            const argument = parseUnary(r);

            return { type: 'UnaryExpression', operator: 'nihil', argument, prefix: true, position };
        }
    }

    // WHY: 'nonnihil x' checks if x is not null (always unary, no ambiguity)
    if (ctx.matchKeyword('nonnihil')) {
        const position = ctx.tokens[ctx.current - 1]!.position;
        const argument = parseUnary(r);

        return { type: 'UnaryExpression', operator: 'nonnihil', argument, prefix: true, position };
    }

    // WHY: 'verum x' checks if x === true (strict boolean true check)
    //      But 'verum' alone is the true literal (handled in parsePrimary)
    //      Must be on same line to avoid ambiguity with next statement
    if (ctx.checkKeyword('verum')) {
        const curr = ctx.peek();
        const next = ctx.peek(1);
        const sameLine = next && next.position.line === curr.position.line;
        const isUnaryOperand =
            sameLine &&
            (next?.type === 'IDENTIFIER' ||
                (next?.type === 'KEYWORD' &&
                    ['verum', 'falsum', 'nihil', 'ego', 'non', 'nulla', 'nonnulla', 'negativum', 'positivum', 'novum', 'cede'].includes(
                        next.value,
                    )));
        if (isUnaryOperand) {
            ctx.advance(); // consume 'verum'
            const position = ctx.tokens[ctx.current - 1]!.position;
            const argument = parseUnary(r);

            return { type: 'UnaryExpression', operator: 'verum', argument, prefix: true, position };
        }
    }

    // WHY: 'falsum x' checks if x === false (strict boolean false check)
    //      But 'falsum' alone is the false literal (handled in parsePrimary)
    //      Must be on same line to avoid ambiguity with next statement
    if (ctx.checkKeyword('falsum')) {
        const curr = ctx.peek();
        const next = ctx.peek(1);
        const sameLine = next && next.position.line === curr.position.line;
        const isUnaryOperand =
            sameLine &&
            (next?.type === 'IDENTIFIER' ||
                (next?.type === 'KEYWORD' &&
                    ['verum', 'falsum', 'nihil', 'ego', 'non', 'nulla', 'nonnulla', 'negativum', 'positivum', 'novum', 'cede'].includes(
                        next.value,
                    )));
        if (isUnaryOperand) {
            ctx.advance(); // consume 'falsum'
            const position = ctx.tokens[ctx.current - 1]!.position;
            const argument = parseUnary(r);

            return { type: 'UnaryExpression', operator: 'falsum', argument, prefix: true, position };
        }
    }

    if (ctx.matchKeyword('negativum')) {
        const position = ctx.tokens[ctx.current - 1]!.position;
        const argument = parseUnary(r);

        return {
            type: 'UnaryExpression',
            operator: 'negativum',
            argument,
            prefix: true,
            position,
        };
    }

    if (ctx.matchKeyword('positivum')) {
        const position = ctx.tokens[ctx.current - 1]!.position;
        const argument = parseUnary(r);

        return {
            type: 'UnaryExpression',
            operator: 'positivum',
            argument,
            prefix: true,
            position,
        };
    }

    if (ctx.matchKeyword('cede')) {
        const position = ctx.tokens[ctx.current - 1]!.position;
        const argument = parseUnary(r);

        return { type: 'CedeExpression', argument, position };
    }

    if (ctx.matchKeyword('novum')) {
        return parseNovumExpression(r);
    }

    if (ctx.matchKeyword('finge')) {
        return parseFingeExpression(r);
    }

    if (ctx.matchKeyword('praefixum')) {
        return parsePraefixumExpression(r);
    }

    if (ctx.matchKeyword('scriptum')) {
        return parseScriptumExpression(r);
    }

    if (ctx.matchKeyword('lege')) {
        return parseLegeExpression(r);
    }

    // Fall through to qua/call parsing via resolver
    // WHY: parseQua is the next level down in precedence, followed by parseCall
    return (r as ResolverWithQua).qua();
}

// =============================================================================
// PREFIX EXPRESSION PARSING
// =============================================================================

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
export function parsePraefixumExpression(r: Resolver): PraefixumExpression {
    const ctx = r.ctx();
    const position = ctx.tokens[ctx.current - 1]!.position; // Position of 'praefixum' we just consumed

    let body: Expression | BlockStatement;

    if (ctx.check('LBRACE')) {
        // Block form: praefixum { ... }
        body = r.block();
    }
    else if (ctx.match('LPAREN')) {
        // Expression form: praefixum(expr)
        body = r.expression();
        ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
    }
    else {
        // Error: expected { or ( - ctx.error() throws (never returns)
        return ctx.error(ParserErrorCode.ExpectedOpeningBraceOrParen, `got '${ctx.peek().value}'`);
    }

    return { type: 'PraefixumExpression', body, position };
}

// =============================================================================
// FORMAT STRING EXPRESSION PARSING
// =============================================================================

/**
 * Parse format string expression.
 *
 * GRAMMAR:
 *   scriptumExpr := 'scriptum' '(' STRING (',' expression)* ')'
 *
 * WHY: "scriptum" (that which has been written) is the perfect passive participle
 *      of scribere. While scribe outputs to console, scriptum returns a formatted string.
 *
 * WHY: The placeholder is converted to target-appropriate format specifiers.
 *
 * Examples:
 *   scriptum("Hello, ...", name)
 *   scriptum("... + ... = ...", a, b, a + b)
 */
export function parseScriptumExpression(r: Resolver): ScriptumExpression {
    const ctx = r.ctx();
    const position = ctx.tokens[ctx.current - 1]!.position; // Position of 'scriptum' we just consumed

    ctx.expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);

    // First argument must be the format string literal
    const formatToken = ctx.peek();
    if (formatToken.type !== 'STRING') {
        ctx.error(ParserErrorCode.ExpectedString, 'scriptum requires a format string literal as first argument');
    }
    ctx.advance();
    const format: Literal = {
        type: 'Literal',
        value: formatToken.value,
        raw: `"${formatToken.value}"`,
        position: formatToken.position,
    };

    // Parse remaining arguments
    const args: Expression[] = [];
    while (ctx.match('COMMA')) {
        args.push(r.expression());
    }

    ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

    return { type: 'ScriptumExpression', format, arguments: args, position };
}

// =============================================================================
// STDIN READ EXPRESSION PARSING
// =============================================================================

/**
 * Parse stdin read expression.
 *
 * GRAMMAR:
 *   legeExpr := 'lege' ('lineam')?
 *
 * Reads from stdin:
 *   lege        -> read all input until EOF
 *   lege lineam -> read one line
 */
export function parseLegeExpression(r: Resolver): LegeExpression {
    const ctx = r.ctx();
    const position = ctx.tokens[ctx.current - 1]!.position; // Position of 'lege' we just consumed

    // Check for 'lineam' modifier
    const mode = ctx.matchKeyword('lineam') ? 'line' : 'all';

    return { type: 'LegeExpression', mode, position };
}

// =============================================================================
// NEW EXPRESSION PARSING
// =============================================================================

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
export function parseNovumExpression(r: Resolver): NovumExpression {
    const ctx = r.ctx();
    const position = ctx.tokens[ctx.current - 1]!.position;
    const callee = ctx.parseIdentifier();

    let args: Expression[] = [];

    if (ctx.match('LPAREN')) {
        args = parseArgumentList(r);

        ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
    }

    let withExpression: Expression | undefined;

    // Check for property overrides: novum X { ... } or novum X de expr
    if (ctx.check('LBRACE')) {
        withExpression = (r as ResolverWithPrimary).primary();
    }
    else if (ctx.matchKeyword('de')) {
        withExpression = r.expression();
    }

    return { type: 'NovumExpression', callee, arguments: args, withExpression, position };
}

// =============================================================================
// DISCRETIO VARIANT CONSTRUCTION
// =============================================================================

/**
 * Parse discretio variant construction expression.
 *
 * GRAMMAR:
 *   fingeExpr := 'finge' IDENTIFIER ('{' fieldList '}')? ('qua' IDENTIFIER)?
 *
 * WHY: Latin 'finge' (form/shape) for constructing discretio variants.
 *      Variant name comes first, optional fields in braces, optional qua for
 *      explicit discretio type when not inferrable from context.
 *
 * Examples:
 *   finge Click { x: 10, y: 20 }           - payload variant
 *   finge Click { x: 10, y: 20 } qua Event - with explicit type
 *   finge Active                            - unit variant
 *   finge Active qua Status                 - unit variant with explicit type
 */
export function parseFingeExpression(r: Resolver): FingeExpression {
    const ctx = r.ctx();
    const position = ctx.tokens[ctx.current - 1]!.position;
    const variant = ctx.parseIdentifier();

    let fields: Expression | undefined;

    // Check for payload fields: finge Click { x: 10, y: 20 }
    if (ctx.check('LBRACE')) {
        const fieldsExpr = (r as ResolverWithPrimary).primary();

        if (fieldsExpr.type === 'ObjectExpression') {
            fields = fieldsExpr;
        }
    }

    let discretioType: Expression | undefined;

    // Check for explicit type: finge Click { } qua Event
    if (ctx.matchKeyword('qua')) {
        discretioType = ctx.parseIdentifier();
    }

    return {
        type: 'FingeExpression',
        variant,
        fields: fields as FingeExpression['fields'],
        discretioType: discretioType as FingeExpression['discretioType'],
        position,
    };
}

// =============================================================================
// ARGUMENT LIST PARSING
// =============================================================================

/**
 * Parse comma-separated argument list (without parens).
 *
 * WHY: Shared by novum and other call-like expressions.
 */
function parseArgumentList(r: Resolver): Expression[] {
    const ctx = r.ctx();
    const args: Expression[] = [];

    if (!ctx.check('RPAREN')) {
        do {
            args.push(r.expression());
        } while (ctx.match('COMMA'));
    }

    return args;
}

// =============================================================================
// EXTENDED RESOLVER TYPES
// =============================================================================

/**
 * Extended Resolver interface for qua expression parsing.
 *
 * WHY: The base Resolver doesn't include qua() since it's expression-internal.
 *      This type allows calling through to the implementation in index.ts.
 */
interface ResolverWithQua extends Resolver {
    qua(): Expression;
}

/**
 * Extended Resolver interface for primary expression parsing.
 *
 * WHY: parsePrimary is needed for object literals in novum/finge but isn't
 *      part of the base Resolver. The implementation provides this method.
 */
interface ResolverWithPrimary extends Resolver {
    primary(): Expression;
}
