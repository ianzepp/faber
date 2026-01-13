/**
 * Zig Code Generator - Array Expression
 *
 * TRANSFORMS:
 *   [1, 2, 3] -> .{ 1, 2, 3 }
 *   [sparge a, sparge b] -> a ++ b (comptime only)
 *
 * TARGET: Zig uses .{ } for array/tuple literals.
 *         Spread requires ++ concatenation (comptime only).
 *
 * LIMITATION: Zig array spread only works at comptime. Runtime spread
 *             would require allocators and explicit memory management.
 */

import type { ArrayExpression, Expression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genArrayExpression(node: ArrayExpression, g: ZigGenerator): string {
    if (node.elements.length === 0) {
        return '.{}';
    }

    // WHY: Zig tuple literals use .{ } syntax. Spread elements emit their
    // argument directly - the ++ concatenation happens at the language level
    // when needed, not in our codegen.
    const elements = node.elements
        .map(el => {
            if (el.type === 'SpreadElement') {
                return g.genExpression(el.argument);
            }
            return g.genExpression(el as Expression);
        })
        .join(', ');

    return `.{ ${elements} }`;
}
