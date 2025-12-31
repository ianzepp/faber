/**
 * TypeScript Code Generator - Finge Expression
 *
 * TRANSFORMS:
 *   finge Click { x: 10, y: 20 } qua Event -> { tag: 'Click', x: 10, y: 20 }
 *   finge Active qua Status -> { tag: 'Active' }
 *
 * WHY: Discretio variants in TypeScript use discriminated unions with a 'tag' property.
 */

import type { FingeExpression, ObjectProperty } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genFingeExpression(node: FingeExpression, g: TsGenerator): string {
    const variant = node.variant.name;

    // Build the object with tag property
    const parts: string[] = [`tag: '${variant}'`];

    // Add payload fields if present
    if (node.fields) {
        for (const prop of node.fields.properties) {
            if (prop.type === 'ObjectProperty') {
                const key = prop.key.type === 'Identifier' ? prop.key.name : g.genExpression(prop.key);
                const value = g.genExpression(prop.value);

                parts.push(`${key}: ${value}`);
            }
        }
    }

    return `{ ${parts.join(', ')} }`;
}
