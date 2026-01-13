/**
 * Binary Expression Parsing
 *
 * Handles parsing of binary expressions including arithmetic, logical, comparison,
 * bitwise, range, assignment, and ternary operators.
 *
 * GRAMMAR: See `EBNF.md` "Expressions" section
 *
 * PRECEDENCE (lowest to highest):
 *   assignment < ternary < or < and < equality < comparison < bitwiseOr < bitwiseXor < bitwiseAnd < range < additive < multiplicative < unary
 *
 * @module parser/expressions/binary
 */

import type { Position } from '../../tokenizer/types';
import type { Resolver } from '../resolver';
import type {
    Expression,
    BinaryExpression,
    AssignmentExpression,
    ConditionalExpression,
    RangeExpression,
    EstExpression,
} from '../ast';
import { ParserErrorCode } from '../errors';

// =============================================================================
// FORWARD REFERENCE
// =============================================================================

/**
 * Forward reference to unary parsing.
 *
 * WHY: parseMultiplicative needs to call parseUnary, which will be in a separate
 *      module. For now, this is a placeholder that will be replaced with an import
 *      from './unary' once that module exists.
 */
let parseUnaryImpl: (r: Resolver) => Expression;

/**
 * Set the unary parser implementation.
 *
 * WHY: Allows the main parser to inject the unary parser after module loading
 *      to break circular dependencies.
 */
export function setUnaryParser(fn: (r: Resolver) => Expression): void {
    parseUnaryImpl = fn;
}

// =============================================================================
// EXPRESSION ENTRY POINT
// =============================================================================

/**
 * Parse expression.
 *
 * GRAMMAR:
 *   expression := assignment
 *
 * WHY: Top-level expression delegates to assignment (lowest precedence).
 */
export function parseExpression(r: Resolver): Expression {
    return parseAssignment(r);
}

// =============================================================================
// ASSIGNMENT
// =============================================================================

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
export function parseAssignment(r: Resolver): Expression {
    const ctx = r.ctx();
    const expr = parseTernary(r);

    if (ctx.match('EQUAL', 'PLUS_EQUAL', 'MINUS_EQUAL', 'STAR_EQUAL', 'SLASH_EQUAL', 'PERCENT_EQUAL', 'AMPERSAND_EQUAL', 'PIPE_EQUAL')) {
        const prevToken = ctx.tokens[ctx.current - 1]!;
        const operator = prevToken.value;
        const position = prevToken.position;
        const value = parseAssignment(r);

        if (expr.type === 'Identifier' || expr.type === 'MemberExpression') {
            return {
                type: 'AssignmentExpression',
                operator,
                left: expr,
                right: value,
                position,
            } as AssignmentExpression;
        }

        ctx.reportError(ParserErrorCode.InvalidAssignmentTarget);
    }

    return expr;
}

// =============================================================================
// TERNARY
// =============================================================================

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
export function parseTernary(r: Resolver): Expression {
    const ctx = r.ctx();
    const test = parseOr(r);
    const position = test.position;

    // Check for symbolic ternary: condition ? consequent : alternate
    if (ctx.match('QUESTION')) {
        const consequent = parseExpression(r);

        if (!ctx.match('COLON')) {
            ctx.reportError(ParserErrorCode.ExpectedColon, `got '${ctx.peek().value}'`);
        }

        const alternate = parseTernary(r);

        return {
            type: 'ConditionalExpression',
            test,
            consequent,
            alternate,
            position,
        } as ConditionalExpression;
    }

    // Check for Latin ternary: condition sic consequent secus alternate
    if (ctx.matchKeyword('sic')) {
        const consequent = parseExpression(r);

        if (!ctx.matchKeyword('secus')) {
            ctx.reportError(ParserErrorCode.ExpectedKeywordSecus, `got '${ctx.peek().value}'`);
        }

        const alternate = parseTernary(r);

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

// =============================================================================
// LOGICAL OR / NULLISH
// =============================================================================

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
export function parseOr(r: Resolver): Expression {
    const ctx = r.ctx();
    let left = parseAnd(r);

    // Track which operator family we're using to prevent mixing
    let operatorKind: 'logical' | 'nullish' | null = null;

    while (true) {
        let isLogicalOr = false;
        let isNullish = false;

        if (ctx.match('OR') || ctx.matchKeyword('aut')) {
            isLogicalOr = true;
        }
        else if (ctx.matchKeyword('vel')) {
            isNullish = true;
        }
        else {
            break;
        }

        const currentKind = isLogicalOr ? 'logical' : 'nullish';

        // WHY: Like JavaScript, mixing ?? and || without parens is ambiguous
        if (operatorKind !== null && operatorKind !== currentKind) {
            ctx.reportError(
                ParserErrorCode.GenericError,
                `Cannot mix 'vel' (nullish) and 'aut'/'||' (logical) without parentheses`
            );
        }

        operatorKind = currentKind;
        const position = ctx.peek().position;
        const operator = isLogicalOr ? '||' : '??';
        const right = parseAnd(r);

        left = { type: 'BinaryExpression', operator, left, right, position };
    }

    return left;
}

// =============================================================================
// LOGICAL AND
// =============================================================================

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
export function parseAnd(r: Resolver): Expression {
    const ctx = r.ctx();
    let left = parseEquality(r);

    while (ctx.match('AND') || ctx.matchKeyword('et')) {
        const position = ctx.peek().position;
        const right = parseEquality(r);

        left = { type: 'BinaryExpression', operator: '&&', left, right, position };
    }

    return left;
}

// =============================================================================
// EQUALITY
// =============================================================================

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
export function parseEquality(r: Resolver): Expression {
    const ctx = r.ctx();
    let left = parseComparison(r);

    while (true) {
        let operator: string;
        let position: Position;

        if (ctx.match('EQUAL_EQUAL', 'BANG_EQUAL', 'TRIPLE_EQUAL', 'BANG_DOUBLE_EQUAL')) {
            const prevToken = ctx.tokens[ctx.current - 1]!;
            operator = prevToken.value;
            position = prevToken.position;
        }
        else if (ctx.checkKeyword('non') && ctx.peek(1)?.type === 'KEYWORD' && ctx.peek(1)?.value === 'est') {
            // 'non est' - negated type check
            position = ctx.peek().position;
            ctx.advance(); // consume 'non'
            ctx.advance(); // consume 'est'

            const targetType = r.typeAnnotation();
            left = {
                type: 'EstExpression',
                expression: left,
                targetType,
                negated: true,
                position,
            } as EstExpression;
            continue;
        }
        else if (ctx.matchKeyword('est')) {
            // 'est' - type check (instanceof/typeof)
            position = ctx.tokens[ctx.current - 1]!.position;

            const targetType = r.typeAnnotation();
            left = {
                type: 'EstExpression',
                expression: left,
                targetType,
                negated: false,
                position,
            } as EstExpression;
            continue;
        }
        else {
            break;
        }

        const right = parseComparison(r);
        left = { type: 'BinaryExpression', operator, left, right, position };
    }

    return left;
}

// =============================================================================
// COMPARISON
// =============================================================================

/**
 * Parse comparison expression.
 *
 * GRAMMAR:
 *   comparison := bitwiseOr (('<' | '>' | '<=' | '>=' | 'intra' | 'inter') bitwiseOr)*
 *
 * PRECEDENCE: Lower than bitwise OR, higher than equality.
 *
 * WHY: intra/inter at comparison level - same precedence as relational operators
 */
export function parseComparison(r: Resolver): Expression {
    const ctx = r.ctx();
    let left = parseBitwiseOr(r);

    while (ctx.match('LESS', 'LESS_EQUAL', 'GREATER', 'GREATER_EQUAL') || ctx.matchKeyword('intra') || ctx.matchKeyword('inter')) {
        const prevToken = ctx.tokens[ctx.current - 1]!;
        const operator = prevToken.value;
        const position = prevToken.position;
        const right = parseBitwiseOr(r);

        left = { type: 'BinaryExpression', operator, left, right, position };
    }

    return left;
}

// =============================================================================
// BITWISE OR
// =============================================================================

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
export function parseBitwiseOr(r: Resolver): Expression {
    const ctx = r.ctx();
    let left = parseBitwiseXor(r);

    while (ctx.match('PIPE')) {
        const prevToken = ctx.tokens[ctx.current - 1]!;
        const operator = prevToken.value;
        const position = prevToken.position;
        const right = parseBitwiseXor(r);

        left = { type: 'BinaryExpression', operator, left, right, position };
    }

    return left;
}

// =============================================================================
// BITWISE XOR
// =============================================================================

/**
 * Parse bitwise XOR expression.
 *
 * GRAMMAR:
 *   bitwiseXor := bitwiseAnd ('^' bitwiseAnd)*
 *
 * PRECEDENCE: Lower than bitwise AND, higher than bitwise OR.
 */
export function parseBitwiseXor(r: Resolver): Expression {
    const ctx = r.ctx();
    let left = parseBitwiseAnd(r);

    while (ctx.match('CARET')) {
        const prevToken = ctx.tokens[ctx.current - 1]!;
        const operator = prevToken.value;
        const position = prevToken.position;
        const right = parseBitwiseAnd(r);

        left = { type: 'BinaryExpression', operator, left, right, position };
    }

    return left;
}

// =============================================================================
// BITWISE AND
// =============================================================================

/**
 * Parse bitwise AND expression.
 *
 * GRAMMAR:
 *   bitwiseAnd := range ('&' range)*
 *
 * PRECEDENCE: Lower than range, higher than bitwise XOR.
 *
 * NOTE: Bit shift operators (<< and >>) were removed from the lexer to avoid
 *       ambiguity with nested generics. Shift operations now use the postfix
 *       keywords dextratum/sinistratum, parsed in parseQuaExpression.
 */
export function parseBitwiseAnd(r: Resolver): Expression {
    const ctx = r.ctx();
    let left = parseRange(r);

    while (ctx.match('AMPERSAND')) {
        const prevToken = ctx.tokens[ctx.current - 1]!;
        const operator = prevToken.value;
        const position = prevToken.position;
        const right = parseRange(r);

        left = { type: 'BinaryExpression', operator, left, right, position };
    }

    return left;
}

// =============================================================================
// RANGE
// =============================================================================

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
export function parseRange(r: Resolver): Expression {
    const ctx = r.ctx();
    const start = parseAdditive(r);

    // Check for range operators: .., ante (exclusive), usque (inclusive)
    let inclusive = false;

    if (ctx.match('DOT_DOT')) {
        inclusive = false;
    }
    else if (ctx.matchKeyword('ante')) {
        inclusive = false;
    }
    else if (ctx.matchKeyword('usque')) {
        inclusive = true;
    }
    else {
        return start;
    }

    const position = ctx.tokens[ctx.current - 1]!.position;
    const end = parseAdditive(r);

    let step: Expression | undefined;

    if (ctx.matchKeyword('per')) {
        step = parseAdditive(r);
    }

    return { type: 'RangeExpression', start, end, step, inclusive, position } as RangeExpression;
}

// =============================================================================
// ADDITIVE
// =============================================================================

/**
 * Parse additive expression.
 *
 * GRAMMAR:
 *   additive := multiplicative (('+' | '-') multiplicative)*
 *
 * PRECEDENCE: Lower than multiplicative, higher than range.
 */
export function parseAdditive(r: Resolver): Expression {
    const ctx = r.ctx();
    let left = parseMultiplicative(r);

    while (ctx.match('PLUS', 'MINUS')) {
        const prevToken = ctx.tokens[ctx.current - 1]!;
        const operator = prevToken.value;
        const position = prevToken.position;
        const right = parseMultiplicative(r);

        left = { type: 'BinaryExpression', operator, left, right, position };
    }

    return left;
}

// =============================================================================
// MULTIPLICATIVE
// =============================================================================

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
 *   - Floor division: a // b (no dedicated operator; tokenizes as '/''/')
 *   - Increment/decrement: x++, ++x, x--, --x
 */
export function parseMultiplicative(r: Resolver): Expression {
    const ctx = r.ctx();
    let left = parseUnaryImpl(r);

    while (ctx.match('STAR', 'SLASH', 'PERCENT')) {
        const prevToken = ctx.tokens[ctx.current - 1]!;
        const operator = prevToken.value;
        const position = prevToken.position;
        const right = parseUnaryImpl(r);

        left = { type: 'BinaryExpression', operator, left, right, position };
    }

    return left;
}
