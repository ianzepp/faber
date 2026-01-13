/**
 * Faber Code Generator - QuaExpression (type cast)
 */

import type { QuaExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genQuaExpression(node: QuaExpression, g: FabGenerator): string {
    const expr = g.genExpression(node.expression);
    const type = g.genType(node.targetType);
    return `${expr} qua ${type}`;
}
