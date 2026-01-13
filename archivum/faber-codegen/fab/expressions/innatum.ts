/**
 * Faber Code Generator - InnatumExpression (native type construction)
 */

import type { InnatumExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genInnatumExpression(node: InnatumExpression, g: FabGenerator): string {
    const expr = g.genExpression(node.expression);
    const type = g.genType(node.targetType);
    return `${expr} innatum ${type}`;
}
