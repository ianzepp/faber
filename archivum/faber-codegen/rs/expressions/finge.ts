/**
 * Rust Code Generator - Finge Expression
 *
 * TRANSFORMS:
 *   finge Click { x: 10, y: 20 } qua Event -> Event::Click { x: 10, y: 20 }
 *   finge Active qua Status -> Status::Active
 *
 * WHY: Discretio variants in Rust use enum variant syntax with :: path separator.
 */

import type { FingeExpression, ObjectProperty } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genFingeExpression(node: FingeExpression, g: RsGenerator): string {
    const variant = node.variant.name;

    // If discretioType is provided, use DiscretioType::Variant syntax
    // Otherwise, just use the variant name (type will be inferred from context)
    const prefix = node.discretioType ? `${node.discretioType.name}::` : '';

    // Unit variant (no payload)
    if (!node.fields || node.fields.properties.length === 0) {
        return `${prefix}${variant}`;
    }

    // Payload variant - build struct fields
    const fields: string[] = [];

    for (const prop of node.fields.properties) {
        if (prop.type === 'ObjectProperty') {
            const key = prop.key.type === 'Identifier' ? prop.key.name : g.genExpression(prop.key);
            const value = g.genExpression(prop.value);

            fields.push(`${key}: ${value}`);
        }
    }

    return `${prefix}${variant} { ${fields.join(', ')} }`;
}
