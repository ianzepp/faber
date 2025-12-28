/**
 * Python Code Generator - Array Expression
 *
 * TRANSFORMS:
 *   [1, 2, 3]           -> [1, 2, 3]
 *   [sparge a, sparge b] -> [*a, *b]
 *
 * WHY: Python uses * for unpacking iterables in list literals.
 */

import type { ArrayExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genArrayExpression(node: ArrayExpression, g: PyGenerator): string {
    const elements = node.elements
        .map(el => {
            if (el.type === 'SpreadElement') {
                return `*${g.genExpression(el.argument)}`;
            }
            return g.genExpression(el);
        })
        .join(', ');
    return `[${elements}]`;
}
