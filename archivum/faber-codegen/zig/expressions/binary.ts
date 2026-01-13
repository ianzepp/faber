/**
 * Zig Code Generator - Binary Expression
 *
 * TRANSFORMS:
 *   x + y           -> (x + y) for numbers
 *   "a" + "b"       -> (a ++ b) for comptime strings
 *   a && b          -> (a and b)
 *   a || b          -> (a or b)
 *   s == "foo"      -> std.mem.eql(u8, s, "foo")
 *   s != "foo"      -> !std.mem.eql(u8, s, "foo")
 *   x intra 0..100  -> (x >= 0 and x < 100)
 *   x inter [1,2,3] -> std.mem.indexOfScalar(i64, &.{1,2,3}, x) != null
 *
 * TARGET: Zig uses 'and'/'or' keywords not &&/|| operators.
 *         String concatenation requires ++ operator (comptime only).
 *         String comparison requires std.mem.eql, not ==.
 */

import type { BinaryExpression, RangeExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genBinaryExpression(node: BinaryExpression, g: ZigGenerator): string {
    const left = g.genExpression(node.left);
    const right = g.genExpression(node.right);

    // Handle string concatenation with + operator
    // WHY: Zig's ++ operator is comptime-only. Runtime string concat needs an allocator.
    //      Instead of silently emitting broken code, throw an error guiding users to scriptum().
    if (node.operator === '+' && (g.isStringType(node.left) || g.isStringType(node.right))) {
        throw new Error(
            `String concatenation with '+' is not supported for Zig target. ` +
                `Use scriptum("format {}", arg) instead. Example: scriptum("Hello, {}", name)`,
        );
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

    // Range containment: x intra range
    // TRANSFORMS: x intra 0..100 -> (x >= 0 and x < 100)
    if (node.operator === 'intra') {
        if (node.right.type === 'RangeExpression') {
            const range = node.right as RangeExpression;
            const start = g.genExpression(range.start);
            const end = g.genExpression(range.end);
            const endOp = range.inclusive ? '<=' : '<';
            return `(${left} >= ${start} and ${left} ${endOp} ${end})`;
        }
        return `(${left} intra ${right})`;
    }

    // Set membership: x inter array
    // TRANSFORMS: x inter [1, 2, 3] -> std.mem.indexOfScalar(i64, &.{1,2,3}, x) != null
    // WHY: Zig doesn't have a direct contains function, use indexOfScalar
    if (node.operator === 'inter') {
        return `(std.mem.indexOfScalar(i64, ${right}, ${left}) != null)`;
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
