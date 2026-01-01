/**
 * TypeScript Code Generator - PactumDeclaration
 *
 * Generates TypeScript interface from pactum declaration.
 *
 * TRANSFORMS:
 *   pactum Comparable<T> { functio compare(T other) -> numerus }
 *   ->
 *   interface Comparable<T> { compare(other: T): number; }
 */

import type { PactumDeclaration, PactumMethod } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { getVisibilityFromAnnotations } from '../../types';

export function genPactumDeclaration(node: PactumDeclaration, g: TsGenerator, semi: boolean): string {
    const name = node.name.name;
    const typeParams = node.typeParameters ? `<${node.typeParameters.map(p => p.name).join(', ')}>` : '';

    // Module-level: export when public
    const visibility = getVisibilityFromAnnotations(node.annotations);
    const exportMod = visibility === 'public' ? 'export ' : '';

    const lines: string[] = [];

    lines.push(`${g.ind()}${exportMod}interface ${name}${typeParams} {`);
    g.depth++;

    for (const method of node.methods) {
        lines.push(genPactumMethod(method, g, semi));
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

function genPactumMethod(node: PactumMethod, g: TsGenerator, semi: boolean): string {
    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p)).join(', ');
    let returnType = node.returnType ? g.genType(node.returnType) : 'void';

    // Wrap return type based on async/generator semantics
    if (node.async && node.generator) {
        returnType = `AsyncGenerator<${returnType}>`;
    } else if (node.generator) {
        returnType = `Generator<${returnType}>`;
    } else if (node.async) {
        returnType = `Promise<${returnType}>`;
    }

    return `${g.ind()}${name}(${params}): ${returnType}${semi ? ';' : ''}`;
}
