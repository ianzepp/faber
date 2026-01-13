/**
 * Faber Code Generator - DiscretioDeclaration (tagged union)
 */

import type { DiscretioDeclaration } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genDiscretioDeclaration(node: DiscretioDeclaration, g: FabGenerator): string {
    const lines: string[] = [];

    let header = `discretio ${node.name.name}`;
    if (node.typeParameters && node.typeParameters.length > 0) {
        header += `<${node.typeParameters.map(p => p.name).join(', ')}>`;
    }

    lines.push(`${g.ind()}${header} {`);

    g.depth++;
    for (const variant of node.variants) {
        if (variant.fields.length === 0) {
            lines.push(`${g.ind()}${variant.name.name}`);
        } else {
            const fields = variant.fields.map(f => `${g.genType(f.fieldType)} ${f.name.name}`).join(', ');
            lines.push(`${g.ind()}${variant.name.name} { ${fields} }`);
        }
    }
    g.depth--;

    lines.push(`${g.ind()}}`);
    return lines.join('\n');
}
