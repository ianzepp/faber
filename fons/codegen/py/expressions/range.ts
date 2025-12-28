/**
 * Python Code Generator - Range Expression
 *
 * TRANSFORMS:
 *   0..10        -> list(range(0, 10))
 *   0 usque 10   -> list(range(0, 11))  (inclusive adds 1 to end)
 *   0..10 per 2  -> list(range(0, 10, 2))
 *
 * WHY: Python range() is exclusive. For inclusive ranges (usque),
 *      we add 1 to the end value.
 * WHY: Wrapped in list() because range() returns an iterator, not a list.
 *      When used in for-loops, the list() wrapper is unnecessary but harmless.
 */

import type { RangeExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genRangeExpression(node: RangeExpression, g: PyGenerator): string {
    const start = g.genExpression(node.start);
    const end = g.genExpression(node.end);
    const endExpr = node.inclusive ? `${end} + 1` : end;

    if (node.step) {
        const step = g.genExpression(node.step);
        return `list(range(${start}, ${endExpr}, ${step}))`;
    }

    return `list(range(${start}, ${endExpr}))`;
}
