/**
 * TypeScript Code Generator - OrdoDeclaration
 *
 * TRANSFORMS:
 *   ordo color { rubrum, viridis } -> enum color { rubrum, viridis }
 *   ordo status { pendens = 0 } -> enum status { pendens = 0 }
 *   ordo dir { north = "n" } -> enum dir { north = "n" }
 *
 * WHY: TypeScript enums map directly from Latin 'ordo'.
 */

import type { OrdoDeclaration } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { getVisibilityFromAnnotations } from '../../types';

export function genOrdoDeclaration(node: OrdoDeclaration, g: TsGenerator): string {
    const name = node.name.name;

    // Module-level: export when public
    const visibility = getVisibilityFromAnnotations(node.annotations);
    const exportMod = visibility === 'public' ? 'export ' : '';

    const members = node.members.map(member => {
        const memberName = member.name.name;

        if (member.value !== undefined) {
            const value = typeof member.value.value === 'string' ? `"${member.value.value}"` : member.value.value;
            return `${memberName} = ${value}`;
        }

        return memberName;
    });

    return `${g.ind()}${exportMod}enum ${name} { ${members.join(', ')} }`;
}
