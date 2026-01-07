/**
 * Zig Code Generator - Conversion Expression (type conversion)
 *
 * TRANSFORMS:
 *   "42" numeratum -> std.fmt.parseInt(i64, "42", 10) catch unreachable
 *   "ff" numeratum<i32, Hex> -> std.fmt.parseInt(i32, "ff", 16) catch unreachable
 *   "42" numeratum vel 0 -> std.fmt.parseInt(i64, "42", 10) catch 0
 *   "3.14" fractatum -> std.fmt.parseFloat(f64, "3.14") catch unreachable
 *   42 textatum -> std.fmt.allocPrint(allocator, "{}", .{42})
 *   x bivalentum -> x != 0
 */

import type { ConversionExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

const RADIX_VALUES: Record<string, number> = {
    Dec: 10,
    Hex: 16,
    Oct: 8,
    Bin: 2,
};

export function genConversionExpression(node: ConversionExpression, g: ZigGenerator): string {
    const expr = g.genExpression(node.expression);
    const fallback = node.fallback ? g.genExpression(node.fallback) : null;

    switch (node.conversion) {
        case 'numeratum': {
            const targetType = node.targetType ? g.genType(node.targetType) : 'i64';
            const radix = node.radix ? RADIX_VALUES[node.radix] : 10;
            const catchClause = fallback ? `catch ${fallback}` : 'catch unreachable';
            return `std.fmt.parseInt(${targetType}, ${expr}, ${radix}) ${catchClause}`;
        }

        case 'fractatum': {
            const targetType = node.targetType ? g.genType(node.targetType) : 'f64';
            const catchClause = fallback ? `catch ${fallback}` : 'catch unreachable';
            return `std.fmt.parseFloat(${targetType}, ${expr}) ${catchClause}`;
        }

        case 'textatum': {
            // WHY: Zig requires allocator for string formatting
            const curator = g.getCurator();
            return `std.fmt.allocPrint(${curator}, "{}", .{${expr}})`;
        }

        case 'bivalentum':
            // WHY: Zig has no truthiness, need explicit comparison
            return `(${expr} != 0)`;
    }
}
