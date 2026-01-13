/**
 * Faber Code Generator - OrdoDeclaration (enum)
 */

import type { OrdoDeclaration } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genOrdoDeclaration(node: OrdoDeclaration, g: FabGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}ordo ${node.name.name} {`);

    g.depth++;
    for (const member of node.members) {
        if (member.value) {
            lines.push(`${g.ind()}${member.name.name} = ${member.value.raw}`);
        } else {
            lines.push(`${g.ind()}${member.name.name}`);
        }
    }
    g.depth--;

    lines.push(`${g.ind()}}`);
    return lines.join('\n');
}
