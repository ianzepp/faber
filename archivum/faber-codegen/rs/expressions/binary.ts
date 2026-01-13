/**
 * Rust Code Generator - Binary Expression
 *
 * TRANSFORMS:
 *   a + b           -> (a + b)
 *   a && b          -> (a && b)
 *   a == b          -> (a == b)
 *   x intra 0..100  -> (x >= 0 && x < 100)
 *   x inter [1,2,3] -> [1,2,3].contains(&x)
 */

import type { BinaryExpression, RangeExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genBinaryExpression(node: BinaryExpression, g: RsGenerator): string {
    const left = g.genExpression(node.left);
    const right = g.genExpression(node.right);

    // Range containment: x intra range
    // TRANSFORMS: x intra 0..100 -> (x >= 0 && x < 100)
    if (node.operator === 'intra') {
        if (node.right.type === 'RangeExpression') {
            const range = node.right as RangeExpression;
            const start = g.genExpression(range.start);
            const end = g.genExpression(range.end);
            const endOp = range.inclusive ? '<=' : '<';
            return `(${left} >= ${start} && ${left} ${endOp} ${end})`;
        }
        return `(${left} intra ${right})`;
    }

    // Set membership: x inter array
    // TRANSFORMS: x inter [1, 2, 3] -> [1, 2, 3].contains(&x)
    if (node.operator === 'inter') {
        return `${right}.contains(&${left})`;
    }

    return `(${left} ${node.operator} ${right})`;
}
