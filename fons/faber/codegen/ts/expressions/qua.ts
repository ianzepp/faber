/**
 * TypeScript Code Generator - Qua Expression (type cast)
 *
 * TRANSFORMS:
 *   x qua textus -> (x as string)
 *   response.body qua objectum -> (response.body as object)
 *   { field: value } qua Genus -> new Genus({ field: value })
 *   {} qua copia<T> -> new Set<T>()
 *
 * WHY: TypeScript uses 'as' for type assertions. Parentheses ensure
 *      correct precedence when the cast appears in larger expressions.
 *
 * EDGE: Object literals cast to genus types need `new` instantiation,
 *       not type assertion, because genus compiles to a class with methods.
 *       Plain `as` would create an object without the class prototype.
 *
 * EDGE: Empty object literals cast to copia (Set) need `new Set()` construction,
 *       not type assertion. `{} as Set<T>` creates a plain object without Set prototype.
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

    // WHY: Empty object literal + copia target = Set construction, not type assertion.
    // `{} as Set<T>` creates a plain object without Set methods like .has(), .add(), etc.
    if (node.expression.type === 'ObjectExpression' && targetTypeName === 'copia') {
        const typeParams = node.targetType.typeParameters;
        const elemTypeAnno = typeParams?.[0] ? getTypeAnnotation(typeParams[0]) : null;
        const elemType = elemTypeAnno ? g.genType(elemTypeAnno) : 'unknown';
        return `new Set<${elemType}>()`;
    }

    const expr = g.genExpression(node.expression);
    return `(${expr} as ${targetType})`;
}
