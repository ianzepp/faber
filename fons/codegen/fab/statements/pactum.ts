/**
 * Faber Code Generator - PactumDeclaration
 *
 * TRANSFORMS:
 *   PactumDeclaration -> pactum name<T>? { methods }
 */

import type { PactumDeclaration } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genPactumDeclaration(node: PactumDeclaration, g: FabGenerator): string {
    const parts: string[] = ['pactum', node.name.name];

    // Type parameters
    if (node.typeParameters && node.typeParameters.length > 0) {
        parts[parts.length - 1] += `<${node.typeParameters.map(p => p.name).join(', ')}>`;
    }

    const lines: string[] = [];
    lines.push(`${g.ind()}${parts.join(' ')} {`);

    g.depth++;
    for (const method of node.methods) {
        const mParts: string[] = [];

        mParts.push('functio');
        mParts.push(method.name.name);

        const params = method.params.map(p => g.genParameter(p)).join(', ');
        mParts[mParts.length - 1] += `(${params})`;

        // Function modifiers (after params): futura, cursor, curata NAME
        if (method.async) {
            mParts.push('futura');
        }
        if (method.generator) {
            mParts.push('cursor');
        }
        if (method.curatorName) {
            mParts.push('curata');
            mParts.push(method.curatorName);
        }

        if (method.returnType) {
            mParts.push('->');
            mParts.push(g.genType(method.returnType));
        }

        lines.push(`${g.ind()}${mParts.join(' ')}`);
    }
    g.depth--;

    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
