/**
 * Python Code Generator - OrdoDeclaration (enum)
 *
 * TRANSFORMS:
 *   ordo Color { rubrum, viridis, caeruleum }
 *   ->
 *   class Color(Enum):
 *       rubrum = auto()
 *       viridis = auto()
 *       caeruleum = auto()
 *
 *   ordo Status { pendens = 0, actum = 1 }
 *   ->
 *   class Status(Enum):
 *       pendens = 0
 *       actum = 1
 *
 * WHY: Python uses class-based Enum from the enum module.
 *      Members without explicit values use auto() for automatic numbering.
 */

import type { OrdoDeclaration } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genOrdoDeclaration(node: OrdoDeclaration, g: PyGenerator): string {
    // Track that we need the Enum import in preamble
    g.features.enum = true;

    const name = node.name.name;
    const lines: string[] = [];

    lines.push(`${g.ind()}class ${name}(Enum):`);
    g.depth++;

    for (const member of node.members) {
        const memberName = member.name.name;

        if (member.value !== undefined) {
            const value = typeof member.value.value === 'string' ? `"${member.value.value}"` : member.value.value;
            lines.push(`${g.ind()}${memberName} = ${value}`);
        } else {
            lines.push(`${g.ind()}${memberName} = auto()`);
        }
    }

    g.depth--;
    return lines.join('\n');
}
