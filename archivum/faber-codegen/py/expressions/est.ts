/**
 * Python Code Generator - EstExpression
 *
 * TRANSFORMS:
 *   x est textus      -> isinstance(x, str)
 *   x est numerus     -> isinstance(x, int)
 *   x est persona     -> isinstance(x, persona)
 *   x est nihil       -> x is None
 *   x est textus?     -> isinstance(x, str) or x is None
 *   x est lista<T>    -> isinstance(x, list)
 *   x non est textus  -> not isinstance(x, str)
 *
 * WHY: Python uses isinstance() for both primitive and class type checks.
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
    const nullable = node.targetType.nullable;

    // Special case: nihil (None check)
    if (typeName === 'nihil') {
        return node.negated ? `${expr} is not None` : `${expr} is None`;
    }

    // Special case: array types (lista<T>) use list
    if (typeName === 'lista') {
        const check = `isinstance(${expr}, list)`;
        return node.negated ? `not ${check}` : check;
    }

    // Get Python type name
    const pyType = ISINSTANCE_PRIMITIVES[typeName] ?? typeName;
    const check = `isinstance(${expr}, ${pyType})`;

    // Handle nullable types: textus? -> isinstance(x, str) or x is None
    if (nullable && !node.negated) {
        return `${check} or ${expr} is None`;
    } else if (nullable && node.negated) {
        // x non est textus? -> not isinstance(x, str) and x is not None
        return `not ${check} and ${expr} is not None`;
    }

    return node.negated ? `not ${check}` : check;
}
