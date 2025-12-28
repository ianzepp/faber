/**
 * TypeScript Code Generator - Est Expression (type check)
 *
 * TRANSFORMS:
 *   x est textus      -> (typeof x === "string")
 *   x est numerus     -> (typeof x === "number")
 *   x est persona     -> (x instanceof persona)
 *   x non est textus  -> (typeof x !== "string")
 *   x non est persona -> !(x instanceof persona)
 *
 * WHY: JavaScript distinguishes primitive type checks (typeof) from
 *      class/constructor checks (instanceof). Latin 'est' unifies these
 *      semantically; codegen chooses the right runtime mechanism.
 *
 * NOTE: For null checks, use `nihil x` or `nonnihil x` unary operators.
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
    const op = node.negated ? '!==' : '===';

    // Check if it's a primitive type (uses typeof)
    const jsType = TYPEOF_PRIMITIVES[typeName];
    if (jsType) {
        return `(typeof ${expr} ${op} "${jsType}")`;
    }

    // User-defined type or collection (uses instanceof)
    const targetType = g.genType(node.targetType);
    if (node.negated) {
        return `!(${expr} instanceof ${targetType})`;
    }
    return `(${expr} instanceof ${targetType})`;
}
