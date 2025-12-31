/**
 * Faber Code Generator - Finge Expression
 *
 * TRANSFORMS:
 *   finge Click { x: 10, y: 20 } qua Event -> finge Click { x: 10, y: 20 } qua Event
 *   finge Active qua Status -> finge Active qua Status
 *   finge Click { x: 10 } -> finge Click { x: 10 }
 *
 * WHY: Emits canonical Faber source for the finge expression.
 */

import type { FingeExpression, ObjectProperty } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genFingeExpression(node: FingeExpression, g: FabGenerator): string {
    const variant = node.variant.name;

    let result = `finge ${variant}`;

    // Add payload fields if present
    if (node.fields && node.fields.properties.length > 0) {
        const fields: string[] = [];

        for (const prop of node.fields.properties) {
            if (prop.type === 'ObjectProperty') {
                const key = prop.key.type === 'Identifier' ? prop.key.name : g.genExpression(prop.key);
                const value = g.genExpression(prop.value);

                fields.push(`${key}: ${value}`);
            }
        }

        result += ` { ${fields.join(', ')} }`;
    }

    // Add explicit discretio type if present
    if (node.discretioType) {
        result += ` qua ${node.discretioType.name}`;
    }

    return result;
}
