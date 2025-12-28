/**
 * Python Code Generator - Binary Expression
 *
 * TRANSFORMS:
 *   a + b      -> (a + b)
 *   a && b     -> (a and b)
 *   a || b     -> (a or b)
 *   a === b    -> (a == b)
 *   a !== b    -> (a != b)
 *   a ?? b     -> (a if a is not None else b)
 *
 * WHY: Python doesn't have nullish coalescing (??), so we expand to
 *      conditional expression. This evaluates left twice, which is
 *      acceptable for simple expressions.
 */

import type { BinaryExpression } from '../../../parser/ast';
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

    const op = mapOperator(node.operator);

    return `(${left} ${op} ${right})`;
}
