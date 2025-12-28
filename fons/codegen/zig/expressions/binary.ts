/**
 * Zig Code Generator - Binary Expression
 *
 * TRANSFORMS:
 *   x + y -> (x + y) for numbers
 *   "a" + "b" -> (a ++ b) for comptime strings
 *   a && b -> (a and b)
 *   a || b -> (a or b)
 *   s == "foo" -> std.mem.eql(u8, s, "foo")
 *   s != "foo" -> !std.mem.eql(u8, s, "foo")
 *
 * TARGET: Zig uses 'and'/'or' keywords not &&/|| operators.
 *         String concatenation requires ++ operator (comptime only).
 *         String comparison requires std.mem.eql, not ==.
 */

import type { BinaryExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genBinaryExpression(node: BinaryExpression, g: ZigGenerator): string {
    const left = g.genExpression(node.left);
    const right = g.genExpression(node.right);

    // Handle string concatenation with + operator
    if (node.operator === '+' && (g.isStringType(node.left) || g.isStringType(node.right))) {
        // For compile-time known strings, use ++ operator
        // Note: This only works for comptime-known strings in Zig
        // Runtime string concatenation would require an allocator
        return `(${left} ++ ${right})`;
    }

    // Handle string comparison with == or ===
    // WHY: Zig cannot compare []const u8 with ==, must use std.mem.eql
    if ((node.operator === '==' || node.operator === '===') && (g.isStringType(node.left) || g.isStringType(node.right))) {
        return `std.mem.eql(u8, ${left}, ${right})`;
    }

    // Handle string comparison with != or !==
    if ((node.operator === '!=' || node.operator === '!==') && (g.isStringType(node.left) || g.isStringType(node.right))) {
        return `!std.mem.eql(u8, ${left}, ${right})`;
    }

    // Handle integer division - Zig requires explicit truncating division for signed ints
    // WHY: Zig's / operator doesn't work with signed integers, must use @divTrunc
    if (node.operator === '/') {
        return `@divTrunc(${left}, ${right})`;
    }

    // Handle modulo - Zig requires @mod or @rem for signed integers
    if (node.operator === '%') {
        return `@mod(${left}, ${right})`;
    }

    const op = mapOperator(node.operator);

    return `(${left} ${op} ${right})`;
}

/**
 * Map JavaScript operators to Zig equivalents.
 *
 * TARGET: Zig uses keyword operators for boolean logic.
 */
function mapOperator(op: string): string {
    switch (op) {
        case '&&':
            return 'and';
        case '||':
            return 'or';
        case '??':
            // WHY: Zig's orelse works on optionals: a orelse b
            return 'orelse';
        case '===':
            return '==';
        case '!==':
            return '!=';
        default:
            return op;
    }
}
