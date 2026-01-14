/**
 * TypeScript Code Generator - Qua Expression (type cast)
 *
 * TRANSFORMS:
 *   x qua textus -> (x as string)
 *   response.body qua objectum -> (response.body as object)
 *   { field: value } qua Genus -> new Genus({ field: value })
 *
 * WHY: TypeScript uses 'as' for type assertions. Parentheses ensure
 *      correct precedence when the cast appears in larger expressions.
 *
 * IMPORTANT: `qua` is a compile-time type assertion only. It does NOT construct
 *            or convert values at runtime. For actual construction of built-in
 *            collection types (lista, tabula, copia), use `innatum` instead:
 *              - [] innatum lista<T>   -> typed array
 *              - {} innatum tabula<K,V> -> new Map<K,V>()
 *              - [] innatum copia<T>   -> new Set<T>()
 *
 * EDGE: Object literals cast to genus types need `new` instantiation,
 *       not type assertion, because genus compiles to a class with methods.
 *       Plain `as` would create an object without the class prototype.
 */

import type { QuaExpression, TypeAnnotation } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

// WHY: TypeParameter can be TypeAnnotation | Literal. For type generation we need TypeAnnotation.
function getTypeAnnotation(param: TypeAnnotation | { type: string }): TypeAnnotation | null {
    if ('name' in param && param.type === 'TypeAnnotation') {
        return param as TypeAnnotation;
    }
    return null;
}

export function genQuaExpression(node: QuaExpression, g: TsGenerator): string {
    const targetType = g.genType(node.targetType);
    const targetTypeName = node.targetType.name;

    // WHY: Object literal + genus target = class instantiation, not type assertion.
    // The genus constructor accepts an overrides object, so we pass the literal directly.
    if (node.expression.type === 'ObjectExpression' && g.isGenus(targetTypeName)) {
        const props = g.genExpression(node.expression);
        return `new ${targetType}(${props})`;
    }

    const expr = g.genExpression(node.expression);
    return `(${expr} as ${targetType})`;
}
