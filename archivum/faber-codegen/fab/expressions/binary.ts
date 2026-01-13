/**
 * Faber Code Generator - BinaryExpression
 *
 * Maps JavaScript operators to Faber canonical forms for parseable output.
 */

import type { BinaryExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

/**
 * Map JavaScript operators to Faber canonical forms.
 */
function mapOperatorToFaber(operator: string): string {
    switch (operator) {
        case '&&': return 'et';
        case '||': return 'aut';
        case '??': return 'vel';
        default: return operator;
    }
}

export function genBinaryExpression(node: BinaryExpression, g: FabGenerator): string {
    const left = g.genExpression(node.left);
    const right = g.genExpression(node.right);
    const faberOperator = mapOperatorToFaber(node.operator);
    return `${left} ${faberOperator} ${right}`;
}
