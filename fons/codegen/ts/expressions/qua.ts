/**
 * TypeScript Code Generator - Qua Expression (type cast)
 *
 * TRANSFORMS:
 *   x qua textus -> (x as string)
 *   response.body qua objectum -> (response.body as object)
 *
 * WHY: TypeScript uses 'as' for type assertions. Parentheses ensure
 *      correct precedence when the cast appears in larger expressions.
 */

import type { QuaExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genQuaExpression(node: QuaExpression, g: TsGenerator): string {
    const expr = g.genExpression(node.expression);
    const targetType = g.genType(node.targetType);

    return `(${expr} as ${targetType})`;
}
