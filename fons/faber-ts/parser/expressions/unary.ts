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
    PraefixumExpression,
    BlockStatement,
} from '../ast';
import { ParserErrorCode } from '../errors';
import {
    parseQuaExpression,
    parseNovumExpression,
    parseFingeExpression,
    parseScriptumExpression,
    parseLegeExpression,
} from './primary';

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

    // Fall through to qua/call parsing
    // WHY: parseQua is the next level down in precedence, followed by parseCall
    return parseQuaExpression(r);
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
