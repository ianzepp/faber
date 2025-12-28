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

    // Check if any elements are spread
    const hasSpread = node.elements.some(el => el.type === 'SpreadElement');

    if (hasSpread) {
        // WHY: Zig doesn't have spread syntax. For comptime arrays, we use ++
        // This is a simplification - runtime would need allocator
        const parts = node.elements.map(el => {
            if (el.type === 'SpreadElement') {
                return g.genExpression(el.argument);
            }
            // Wrap non-spread elements in array literal
            return `.{ ${g.genExpression(el)} }`;
        });

        return parts.join(' ++ ');
    }

    const elements = node.elements.map(el => g.genExpression(el as Expression)).join(', ');

    return `.{ ${elements} }`;
}
