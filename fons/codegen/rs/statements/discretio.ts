/**
 * Rust Code Generator - DiscretioDeclaration
 *
 * TRANSFORMS:
 *   discretio Event { Click { numerus x }, Quit }
 *   -> enum Event { Click { x: f64 }, Quit }
 */

import type { DiscretioDeclaration, VariantDeclaration } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genDiscretioDeclaration(node: DiscretioDeclaration, g: RsGenerator): string {
    const name = node.name.name;
    const typeParams = node.typeParameters ? `<${node.typeParameters.map(p => p.name).join(', ')}>` : '';
    const lines: string[] = [];

    lines.push(`${g.ind()}enum ${name}${typeParams} {`);
    g.depth++;

    for (const variant of node.variants) {
        lines.push(genVariantDeclaration(variant, g));
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

function genVariantDeclaration(node: VariantDeclaration, g: RsGenerator): string {
    const name = node.name.name;

    if (node.fields.length === 0) {
        return `${g.ind()}${name},`;
    }

    const fields = node.fields.map(f => `${f.name.name}: ${g.genType(f.fieldType)}`).join(', ');

    return `${g.ind()}${name} { ${fields} },`;
}
