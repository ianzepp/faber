/**
 * Rust Code Generator - Range Expression
 *
 * TRANSFORMS:
 *   0..10 -> (0..10)
 *   0...10 -> (0..=10)
 */

import type { RangeExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genRangeExpression(node: RangeExpression, g: RsGenerator): string {
    const start = g.genExpression(node.start);
    const end = g.genExpression(node.end);
    const rangeOp = node.inclusive ? '..=' : '..';

    return `(${start}${rangeOp}${end})`;
}
