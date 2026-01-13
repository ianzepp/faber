/**
 * Zig Code Generator - OrdoDeclaration (enum)
 *
 * TRANSFORMS:
 *   ordo Color { Red, Green, Blue }
 *   -> const Color = enum { red, green, blue };
 *
 * TARGET: Zig has native enums with lowercase convention.
 */

import type { ZigGenerator } from '../generator';

// WHY: OrdoDeclaration may not be defined in AST yet, use generic type
interface OrdoDeclaration {
    type: 'OrdoDeclaration';
    name: { name: string };
    members: Array<{ name: { name: string }; value?: { value: number } }>;
}

export function genOrdoDeclaration(node: OrdoDeclaration, g: ZigGenerator): string {
    const name = node.name.name;
    const lines: string[] = [];

    lines.push(`${g.ind()}const ${name} = enum {`);
    g.depth++;

    for (const member of node.members) {
        const memberName = member.name.name.toLowerCase();
        if (member.value !== undefined) {
            lines.push(`${g.ind()}${memberName} = ${member.value.value},`);
        } else {
            lines.push(`${g.ind()}${memberName},`);
        }
    }

    g.depth--;
    lines.push(`${g.ind()}};`);

    return lines.join('\n');
}
