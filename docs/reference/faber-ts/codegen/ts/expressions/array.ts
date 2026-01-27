/**
 * TypeScript Code Generator - Array Expression
 *
 * TRANSFORMS:
 *   [] -> []
 *   [1, 2, 3] -> [1, 2, 3]
 *   [[1], [2]] -> [[1], [2]]
 *   [sparge a, sparge b] -> [...a, ...b]
 */

import type { ArrayExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genArrayExpression(node: ArrayExpression, g: TsGenerator): string {
    const elements = node.elements
        .map(el => {
            if (el.type === 'SpreadElement') {
                return `...${g.genExpression(el.argument)}`;
            }
            return g.genExpression(el);
        })
        .join(', ');

    return `[${elements}]`;
}
