/**
 * Zig Code Generator - Object Expression
 *
 * TRANSFORMS:
 *   { nomen: "Marcus" } -> .{ .nomen = "Marcus" }
 *   { sparge defaults, x: 1 } -> (struct merge not directly supported)
 *
 * TARGET: Zig uses .{ .field = value } for struct literals.
 *
 * LIMITATION: Zig doesn't have object spread. We skip spread elements
 *             since they cannot be represented in Zig struct literals.
 *
 * EDGE: 'error' is a reserved keyword in Zig, rename to 'err'.
 */

import type { ObjectExpression, Literal } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genObjectExpression(node: ObjectExpression, g: ZigGenerator): string {
    if (node.properties.length === 0) {
        return '.{}';
    }

    // Filter out spread elements - they can't be represented in Zig
    const props = node.properties
        .filter(prop => prop.type !== 'SpreadElement')
        .map(prop => {
            let key = prop.key.type === 'Identifier' ? prop.key.name : String((prop.key as Literal).value);

            // EDGE: 'error' is reserved in Zig
            if (key === 'error') {
                key = 'err';
            }

            const value = g.genExpression(prop.value);

            return `.${key} = ${value}`;
        });

    return `.{ ${props.join(', ')} }`;
}
