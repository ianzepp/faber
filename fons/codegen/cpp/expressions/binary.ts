/**
 * C++23 Code Generator - BinaryExpression
 *
 * TRANSFORMS:
 *   a + b   -> (a + b)
 *   a === b -> (a == b)
 *   a !== b -> (a != b)
 *   a && b  -> (a && b)
 *   a || b  -> (a || b)
 *   a ?? b  -> (a != nullptr ? a : b)
 *
 * WHY: C++ has no ?? operator; use ternary with nullptr check.
 *      For std::optional, would use .value_or() instead.
 */

import type { BinaryExpression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

/**
 * Map operators to C++ equivalents.
 */
function mapOperator(op: string): string {
    switch (op) {
        case '===':
            return '==';
        case '!==':
            return '!=';
        case '&&':
            return '&&';
        case '||':
            return '||';
        default:
            return op;
    }
}

export function genBinaryExpression(node: BinaryExpression, g: CppGenerator): string {
    const left = g.genExpression(node.left);
    const right = g.genExpression(node.right);

    // WHY: C++ has no ?? operator; use ternary with nullptr check
    //      For std::optional, would use .value_or() instead
    if (node.operator === '??') {
        return `(${left} != nullptr ? ${left} : ${right})`;
    }

    const op = mapOperator(node.operator);

    return `(${left} ${op} ${right})`;
}
