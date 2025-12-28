/**
 * Rust Code Generator - PactumDeclaration
 *
 * TRANSFORMS:
 *   pactum Drawable { functio draw() -> nihil }
 *   -> trait Drawable { fn draw(); }
 */

import type { PactumDeclaration, PactumMethod } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genPactumDeclaration(node: PactumDeclaration, g: RsGenerator): string {
    const name = node.name.name;
    const typeParams = node.typeParameters ? `<${node.typeParameters.map(p => p.name).join(', ')}>` : '';
    const lines: string[] = [];

    lines.push(`${g.ind()}trait ${name}${typeParams} {`);
    g.depth++;

    for (const method of node.methods) {
        lines.push(genPactumMethod(method, g));
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

function genPactumMethod(node: PactumMethod, g: RsGenerator): string {
    const asyncMod = node.async ? 'async ' : '';
    const name = node.name.name;
    const params = ['&self', ...node.params.map(p => g.genParameter(p))].join(', ');
    let returnType = node.returnType ? ` -> ${g.genType(node.returnType)}` : '';

    return `${g.ind()}${asyncMod}fn ${name}(${params})${returnType};`;
}
