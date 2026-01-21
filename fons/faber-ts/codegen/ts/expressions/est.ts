/**
 * TypeScript Code Generator - Est Expression (type check)
 *
 * TRANSFORMS:
 *   x est textus      -> typeof x === "string"
 *   x est numerus     -> typeof x === "number"
 *   x est persona     -> x instanceof persona
 *   x est nihil       -> x === null
 *   x est textus?     -> typeof x === "string" || x === null
 *   x est lista<T>    -> Array.isArray(x)
 *   x non est textus  -> typeof x !== "string"
 *   x non est persona -> !(x instanceof persona)
 *
 * WHY: JavaScript distinguishes primitive type checks (typeof) from
 *      class/constructor checks (instanceof). Latin 'est' unifies these
 *      semantically; codegen chooses the right runtime mechanism.
 */

import type { EstExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

/**
 * Primitive types that use typeof for runtime checks.
 *
 * WHY: These Latin type names map to JavaScript typeof strings.
 *      Other types (user-defined, collections) use instanceof.
 */
const TYPEOF_PRIMITIVES: Record<string, string> = {
    textus: 'string',
    numerus: 'number',
    fractus: 'number',
    bivalens: 'boolean',
    functio: 'function',
    signum: 'symbol',
    magnus: 'bigint',
    incertum: 'undefined',
    objectum: 'object',
};

export function genEstExpression(node: EstExpression, g: TsGenerator): string {
    const expr = g.genExpression(node.expression);
    const typeName = node.targetType.name;
    const nullable = node.targetType.nullable;

    // Special case: nihil (null check)
    if (typeName === 'nihil') {
        return node.negated ? `${expr} !== null` : `${expr} === null`;
    }

    // Special case: array types (lista<T>) use Array.isArray
    if (typeName === 'lista') {
        if (node.negated) {
            return `!Array.isArray(${expr})`;
        }
        return `Array.isArray(${expr})`;
    }

    // Check if it's a primitive type (uses typeof)
    const jsType = TYPEOF_PRIMITIVES[typeName];
    if (jsType) {
        const op = node.negated ? '!==' : '===';
        const typeCheck = `typeof ${expr} ${op} "${jsType}"`;

        // Handle nullable types: textus? -> typeof x === "string" || x === null
        if (nullable && !node.negated) {
            return `${typeCheck} || ${expr} === null`;
        } else if (nullable && node.negated) {
            // x non est textus? -> typeof x !== "string" && x !== null
            return `${typeCheck} && ${expr} !== null`;
        }

        return typeCheck;
    }

    // User-defined type or collection (uses instanceof)
    const targetType = g.genType(node.targetType);
    if (node.negated) {
        return `!(${expr} instanceof ${targetType})`;
    }
    return `${expr} instanceof ${targetType}`;
}
