/**
 * Zig Code Generator - Finge Expression
 *
 * TRANSFORMS:
 *   finge Click { x: 10, y: 20 } qua Event -> Event{ .click = .{ .x = 10, .y = 20 } }
 *   finge Active qua Status -> Status.active
 *
 * WHY: Discretio variants in Zig use tagged unions with lowercase field names.
 *      Payload variants use nested struct initialization.
 */

import type { FingeExpression, ObjectProperty } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genFingeExpression(node: FingeExpression, g: ZigGenerator): string {
    const variant = node.variant.name;
    // Zig uses lowercase for union field names
    const variantLower = variant.charAt(0).toLowerCase() + variant.slice(1);

    // If discretioType is provided, use it as prefix
    const prefix = node.discretioType ? `${node.discretioType.name}` : '';

    // Unit variant (no payload)
    if (!node.fields || node.fields.properties.length === 0) {
        if (prefix) {
            return `${prefix}.${variantLower}`;
        }

        return `.${variantLower}`;
    }

    // Payload variant - build struct initializer
    const fields: string[] = [];

    for (const prop of node.fields.properties) {
        if (prop.type === 'ObjectProperty') {
            const key = prop.key.type === 'Identifier' ? prop.key.name : g.genExpression(prop.key);
            const value = g.genExpression(prop.value);

            fields.push(`.${key} = ${value}`);
        }
    }

    const payload = `.{ ${fields.join(', ')} }`;

    if (prefix) {
        return `${prefix}{ .${variantLower} = ${payload} }`;
    }

    return `.{ .${variantLower} = ${payload} }`;
}
