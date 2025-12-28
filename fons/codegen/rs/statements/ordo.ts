/**
 * Rust Code Generator - OrdoDeclaration
 *
 * TRANSFORMS:
 *   ordo Color { Red, Green, Blue }
 *   -> enum Color { Red, Green, Blue }
 *
 *   ordo Status { Active = 1, Inactive = 0 }
 *   -> enum Status { Active = 1, Inactive = 0 }
 */

import type { OrdoDeclaration } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genOrdoDeclaration(node: OrdoDeclaration, g: RsGenerator): string {
    const name = node.name.name;
    const lines: string[] = [];

    lines.push(`${g.ind()}enum ${name} {`);
    g.depth++;

    for (const member of node.members) {
        const memberName = member.name.name;
        if (member.value !== undefined) {
            const value = typeof member.value.value === 'string' ? `= "${member.value.value}"` : `= ${member.value.value}`;
            lines.push(`${g.ind()}${memberName} ${value},`);
        } else {
            lines.push(`${g.ind()}${memberName},`);
        }
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
