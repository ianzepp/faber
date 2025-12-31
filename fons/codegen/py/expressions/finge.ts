/**
 * Python Code Generator - Finge Expression
 *
 * TRANSFORMS:
 *   finge Click { x: 10, y: 20 } qua Event -> Event_Click(x=10, y=20)
 *   finge Active qua Status -> Status_Active()
 *
 * WHY: Discretio variants in Python use dataclasses with DiscretioType_Variant naming.
 */

import type { FingeExpression, ObjectProperty } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genFingeExpression(node: FingeExpression, g: PyGenerator): string {
    const variant = node.variant.name;

    // If discretioType is provided, use DiscretioType_Variant naming
    // Otherwise, just use the variant name (type will be inferred from context)
    const className = node.discretioType ? `${node.discretioType.name}_${variant}` : variant;

    // Build keyword arguments for payload fields
    const args: string[] = [];

    if (node.fields) {
        for (const prop of node.fields.properties) {
            if (prop.type === 'ObjectProperty') {
                const key = prop.key.type === 'Identifier' ? prop.key.name : g.genExpression(prop.key);
                const value = g.genExpression(prop.value);

                args.push(`${key}=${value}`);
            }
        }
    }

    return `${className}(${args.join(', ')})`;
}
