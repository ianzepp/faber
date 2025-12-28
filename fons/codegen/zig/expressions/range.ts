/**
 * Zig Code Generator - Range Expression
 *
 * TRANSFORMS:
 *   0..10 -> (range not directly expressible in Zig)
 *
 * TARGET: Zig doesn't have range literals. This is only used when a range
 *         appears outside a for loop. We generate a comment indicating
 *         the limitation.
 */

import type { RangeExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genRangeExpression(node: RangeExpression, g: ZigGenerator): string {
    const start = g.genExpression(node.start);
    const end = g.genExpression(node.end);

    // Zig doesn't have standalone range expressions
    // This would need to be converted to a slice or iterator
    return `@compileError("Range expressions must be used in for loops")`;
}
