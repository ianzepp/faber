/**
 * TypeScript Code Generator - Innatum Expression (native type construction)
 *
 * TRANSFORMS:
 *   {} innatum tabula<K,V> -> new Map<K, V>()
 *   [] innatum lista<T> -> []
 *
 * WHY: tabula maps to Map in TypeScript, which requires explicit construction.
 *      lista maps to array, which can use literal syntax.
 *      Unlike 'qua' (type assertion), 'innatum' actually constructs the native type.
 */

import type { InnatumExpression, TypeAnnotation } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

// WHY: TypeParameter can be TypeAnnotation | Literal. For type generation we need TypeAnnotation.
function getTypeAnnotation(param: TypeAnnotation | { type: string }): TypeAnnotation | null {
    if ('name' in param && param.type === 'TypeAnnotation') {
        return param as TypeAnnotation;
    }
    return null;
}

export function genInnatumExpression(node: InnatumExpression, g: TsGenerator): string {
    const typeName = node.targetType.name;

    if (typeName === 'tabula') {
        // WHY: Map requires new construction with type parameters
        const typeParams = node.targetType.typeParameters;
        const keyTypeAnno = typeParams?.[0] ? getTypeAnnotation(typeParams[0]) : null;
        const valueTypeAnno = typeParams?.[1] ? getTypeAnnotation(typeParams[1]) : null;
        const keyType = keyTypeAnno ? g.genType(keyTypeAnno) : 'unknown';
        const valueType = valueTypeAnno ? g.genType(valueTypeAnno) : 'unknown';

        // Handle non-empty object literal: {a: 1, b: 2} innatum tabula
        if (node.expression.type === 'ObjectExpression' && node.expression.properties.length > 0) {
            const entries = node.expression.properties.map(prop => {
                if (prop.type === 'SpreadElement') {
                    return `...${g.genExpression(prop.argument)}`;
                }
                // WHY: key is Identifier | Literal, use name for identifiers, stringify for literals
                const key = prop.key.type === 'Identifier' ? JSON.stringify(prop.key.name) : JSON.stringify((prop.key as any).value);
                const value = g.genExpression(prop.value);
                return `[${key}, ${value}]`;
            }).join(', ');
            return `new Map<${keyType}, ${valueType}>([${entries}])`;
        }

        return `new Map<${keyType}, ${valueType}>()`;
    }

    if (typeName === 'lista') {
        // WHY: Array can use literal syntax, but we emit typed version for clarity
        const typeParams = node.targetType.typeParameters;
        const elemTypeAnno = typeParams?.[0] ? getTypeAnnotation(typeParams[0]) : null;
        if (elemTypeAnno) {
            const elemType = g.genType(elemTypeAnno);
            // Check if there are elements in the array
            if (node.expression.type === 'ArrayExpression' && node.expression.elements.length > 0) {
                return g.genExpression(node.expression);
            }
            return `([] as ${elemType}[])`;
        }
        return '[]';
    }

    if (typeName === 'copia') {
        // WHY: Set requires new construction with type parameter
        const typeParams = node.targetType.typeParameters;
        const elemTypeAnno = typeParams?.[0] ? getTypeAnnotation(typeParams[0]) : null;
        const elemType = elemTypeAnno ? g.genType(elemTypeAnno) : 'unknown';
        return `new Set<${elemType}>()`;
    }

    // Fallback: just generate the expression (shouldn't happen if semantic validation works)
    return g.genExpression(node.expression);
}
