/**
 * Python Code Generator - Binary Expression
 *
 * TRANSFORMS:
 *   a + b           -> (a + b)
 *   a && b          -> (a and b)
 *   a || b          -> (a or b)
 *   a === b         -> (a == b)
 *   a !== b         -> (a != b)
 *   a ?? b          -> (a if a is not None else b)
 *   x intra 0..100  -> (x >= 0 and x < 100)
 *   x inter [1,2,3] -> x in [1,2,3]
 *
 * WHY: Python doesn't have nullish coalescing (??), so we expand to
 *      conditional expression. This evaluates left twice, which is
 *      acceptable for simple expressions.
 */

import type { BinaryExpression, RangeExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

/**
 * Map operators to Python equivalents.
 */
function mapOperator(op: string): string {
    switch (op) {
        case '&&':
            return 'and';
        case '||':
            return 'or';
        case '===':
            return '==';
        case '!==':
            return '!=';
        default:
            return op;
    }
}

export function genBinaryExpression(node: BinaryExpression, g: PyGenerator): string {
    const left = g.genExpression(node.left);
    const right = g.genExpression(node.right);

    // WHY: Python has no ?? operator; use conditional expression
    if (node.operator === '??') {
        return `(${left} if ${left} is not None else ${right})`;
    }

    // Range containment: x intra range
    // TRANSFORMS: x intra 0..100 -> (x >= 0 and x < 100)
    if (node.operator === 'intra') {
        if (node.right.type === 'RangeExpression') {
            const range = node.right as RangeExpression;
            const start = g.genExpression(range.start);
            const end = g.genExpression(range.end);
            const endOp = range.inclusive ? '<=' : '<';
            return `(${left} >= ${start} and ${left} ${endOp} ${end})`;
        }
        return `(${left} intra ${right})`;
    }

    // Set membership: x inter array
    // TRANSFORMS: x inter [1, 2, 3] -> x in [1, 2, 3]
    if (node.operator === 'inter') {
        return `${left} in ${right}`;
    }

    const op = mapOperator(node.operator);

    return `(${left} ${op} ${right})`;
}
