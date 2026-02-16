/**
 * TypeScript Code Generator - Qua Expression (type cast)
 *
 * TRANSFORMS:
 *   x qua textus -> (x as string)
 *   response.body qua objectum -> (response.body as object)
 *
 * WHY: TypeScript uses 'as' for type assertions. Parentheses ensure
 *      correct precedence when the cast appears in larger expressions.
 *
 * IMPORTANT: `qua` is a compile-time type assertion only. It does NOT construct
 *            or convert values at runtime. For construction, use:
 *              - { ... } novum Type    -> new Type({ ... })  (postfix construction)
 *              - novum Type { ... }    -> new Type({ ... })  (prefix construction)
 *              - [] innatum lista<T>   -> typed array
 *              - {} innatum tabula<K,V> -> new Map<K,V>()
 *              - [] innatum copia<T>   -> new Set<T>()
 */

import type { QuaExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genQuaExpression(node: QuaExpression, g: TsGenerator): string {
    const targetType = g.genType(node.targetType);
    const expr = g.genExpression(node.expression);
    return `(${expr} as ${targetType})`;
}
