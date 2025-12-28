/**
 * Python Code Generator - EstExpression
 *
 * TRANSFORMS:
 *   x est textus      -> isinstance(x, str)
 *   x est numerus     -> isinstance(x, int)
 *   x est persona     -> isinstance(x, persona)
 *   x est nihil       -> (x is None)
 *   x non est textus  -> not isinstance(x, str)
 *
 * WHY: Python uses isinstance() for both primitive and class type checks.
 *
 * NOTE: For null checks, use `nihil x` or `nonnihil x` unary operators.
 */

import type { EstExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

/**
 * Primitive types that use isinstance with Python built-in types.
 */
const ISINSTANCE_PRIMITIVES: Record<string, string> = {
    textus: 'str',
    numerus: 'int',
    fractus: 'float',
    bivalens: 'bool',
    magnus: 'int', // Python int handles bigint natively
};

export function genEstExpression(node: EstExpression, g: PyGenerator): string {
    const expr = g.genExpression(node.expression);
    const typeName = node.targetType.name;

    // Get Python type name
    const pyType = ISINSTANCE_PRIMITIVES[typeName] ?? typeName;
    const check = `isinstance(${expr}, ${pyType})`;

    return node.negated ? `(not ${check})` : check;
}
