/**
 * TypeScript Code Generator - Postfix Novum Expression
 *
 * TRANSFORMS:
 *   { x: 1 } novum Type -> new Type({ x: 1 })
 *
 * WHY: Postfix construction syntax parallels 'qua' casting syntax but
 *      explicitly constructs a class instance instead of type assertion.
 */

import type { PostfixNovumExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genPostfixNovumExpression(node: PostfixNovumExpression, g: TsGenerator): string {
    const targetType = g.genType(node.targetType);
    const expr = g.genExpression(node.expression);
    return `new ${targetType}(${expr})`;
}
