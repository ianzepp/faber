/**
 * C++23 Code Generator - ObjectExpression
 *
 * TRANSFORMS:
 *   { x: 1, y: 2 } -> {.x = 1, .y = 2}  (C++20 designated initializers)
 *   {}             -> {}
 */

import type { ObjectExpression, Literal } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genObjectExpression(node: ObjectExpression, g: CppGenerator): string {
    if (node.properties.length === 0) {
        return '{}';
    }

    // For simple structs, use designated initializers (C++20)
    const props = node.properties
        .filter(prop => prop.type !== 'SpreadElement')
        .map(prop => {
            const p = prop as any;
            const key = p.key.type === 'Identifier' ? p.key.name : String((p.key as Literal).value);
            const value = g.genExpression(p.value);

            return `.${key} = ${value}`;
        });

    return `{${props.join(', ')}}`;
}
