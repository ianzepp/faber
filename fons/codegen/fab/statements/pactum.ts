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
        // Build annotation modifiers
        const annotationMods: string[] = [];

        if (method.async) {
            annotationMods.push('futura');
        }
        if (method.generator) {
            annotationMods.push('cursor');
        }

        // Emit annotation line if we have modifiers
        if (annotationMods.length > 0) {
            lines.push(`${g.ind()}@ ${annotationMods.join(' ')}`);
        }

        const mParts: string[] = [];

        mParts.push('functio');
        mParts.push(method.name.name);

        const params = method.params.map(p => g.genParameter(p)).join(', ');
        mParts[mParts.length - 1] += `(${params})`;

        // curata NAME stays inline (binds a name)
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
