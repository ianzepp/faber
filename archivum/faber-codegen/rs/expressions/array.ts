/**
 * Rust Code Generator - Array Expression
 *
 * TRANSFORMS:
 *   [1, 2, 3]   -> vec![1, 2, 3]
 *   [a, ...b]   -> vec![a, (spread) b]
 */

import type { ArrayExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genArrayExpression(node: ArrayExpression, g: RsGenerator): string {
    const elements = node.elements
        .map(el => {
            if (el.type === 'SpreadElement') {
                // Rust doesn't have spread in array literals
                return `/* spread */ ${g.genExpression(el.argument)}`;
            }
            return g.genExpression(el);
        })
        .join(', ');

    return `vec![${elements}]`;
}
