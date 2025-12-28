/**
 * Python Code Generator - Object Expression
 *
 * TRANSFORMS:
 *   { nomen: "Marcus" }       -> {"nomen": "Marcus"}
 *   { sparge defaults, x: 1 } -> {**defaults, "x": 1}
 *
 * WHY: Python uses ** for unpacking dicts in dict literals.
 * WHY: Python dict keys must be quoted strings (for string keys).
 */

import type { ObjectExpression, Literal } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genObjectExpression(node: ObjectExpression, g: PyGenerator): string {
    if (node.properties.length === 0) {
        return '{}';
    }

    const props = node.properties.map(prop => {
        if (prop.type === 'SpreadElement') {
            return `**${g.genExpression(prop.argument)}`;
        }
        const key = prop.key.type === 'Identifier' ? `"${prop.key.name}"` : genLiteralKey(prop.key);
        const value = g.genExpression(prop.value);
        return `${key}: ${value}`;
    });

    return `{${props.join(', ')}}`;
}

/**
 * Generate literal key for object property.
 * Inlined to avoid circular dependencies with literal.ts
 */
function genLiteralKey(node: Literal): string {
    if (node.value === null) {
        return 'None';
    }

    if (typeof node.value === 'string') {
        return JSON.stringify(node.value);
    }

    if (typeof node.value === 'boolean') {
        return node.value ? 'True' : 'False';
    }

    if (typeof node.value === 'bigint') {
        return node.raw.replace(/n$/, '');
    }

    if (typeof node.value === 'number') {
        return node.raw;
    }

    return String(node.value);
}
