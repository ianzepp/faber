/**
 * C++ Code Generator - Finge Expression
 *
 * TRANSFORMS:
 *   finge Click { x: 10, y: 20 } qua Event -> Click{.x = 10, .y = 20}
 *   finge Active qua Status -> Active{}
 *
 * WHY: Discretio variants in C++ use std::variant with struct types.
 *      Variant structs are instantiated directly with designated initializers.
 */

import type { FingeExpression, ObjectProperty } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genFingeExpression(node: FingeExpression, g: CppGenerator): string {
    const variant = node.variant.name;

    // Unit variant (no payload)
    if (!node.fields || node.fields.properties.length === 0) {
        return `${variant}{}`;
    }

    // Payload variant - build designated initializers
    const fields: string[] = [];

    for (const prop of node.fields.properties) {
        if (prop.type === 'ObjectProperty') {
            const key = prop.key.type === 'Identifier' ? prop.key.name : g.genExpression(prop.key);
            const value = g.genExpression(prop.value);

            fields.push(`.${key} = ${value}`);
        }
    }

    return `${variant}{${fields.join(', ')}}`;
}
