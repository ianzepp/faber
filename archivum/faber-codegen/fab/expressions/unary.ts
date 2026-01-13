/**
 * Faber Code Generator - UnaryExpression
 *
 * STYLE: Uses non (canonical), not !
 */

import type { UnaryExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genUnaryExpression(node: UnaryExpression, g: FabGenerator): string {
    const arg = g.genExpression(node.argument);

    // Canonical: use non instead of !
    const op = node.operator === '!' ? 'non' : node.operator;

    if (node.prefix) {
        return `${op} ${arg}`;
    }
    return `${arg}${op}`;
}
