/**
 * Faber Code Generator - EstExpression (type check)
 */

import type { EstExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genEstExpression(node: EstExpression, g: FabGenerator): string {
    const expr = g.genExpression(node.expression);
    const type = g.genType(node.targetType);
    const op = node.negated ? 'non est' : 'est';
    return `${expr} ${op} ${type}`;
}
