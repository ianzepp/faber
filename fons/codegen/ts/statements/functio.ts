/**
 * TypeScript Code Generator - FunctioDeclaration
 *
 * TRANSFORMS:
 *   functio salve(nomen: textus): nihil -> function salve(nomen: string): null
 *   futura functio f(): numerus -> async function f(): Promise<number>
 *   cursor functio f(): numerus -> function* f(): Generator<number>
 *   futura cursor functio f(): numerus -> async function* f(): AsyncGenerator<number>
 */

import type { FunctioDeclaration, BlockStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genFunctioDeclaration(node: FunctioDeclaration, g: TsGenerator): string {
    const async = node.async ? 'async ' : '';
    const star = node.generator ? '*' : '';
    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p)).join(', ');

    // Generate type parameters: prae typus T -> <T>
    const typeParams = node.typeParams ? g.genTypeParams(node.typeParams) : '';

    // Wrap return type based on async/generator semantics
    let returnType = '';
    if (node.returnType) {
        let baseType = g.genType(node.returnType);
        if (node.async && node.generator) {
            baseType = `AsyncGenerator<${baseType}>`;
        } else if (node.generator) {
            baseType = `Generator<${baseType}>`;
        } else if (node.async) {
            baseType = `Promise<${baseType}>`;
        }
        returnType = `: ${baseType}`;
    }

    // Track generator context for cede -> yield vs await
    const prevInGenerator = g.inGenerator;
    g.inGenerator = node.generator;
    const body = genBlockStatement(node.body, g);
    g.inGenerator = prevInGenerator;

    return `${g.ind()}${async}function${star} ${name}${typeParams}(${params})${returnType} ${body}`;
}

/**
 * Generate block statement.
 */
export function genBlockStatement(node: BlockStatement, g: TsGenerator): string {
    if (node.body.length === 0) {
        return '{}';
    }

    g.depth++;
    const body = node.body.map(stmt => g.genStatement(stmt)).join('\n');

    g.depth--;

    return `{\n${body}\n${g.ind()}}`;
}

/**
 * Generate method declaration within a class.
 */
export function genMethodDeclaration(node: FunctioDeclaration, g: TsGenerator): string {
    const asyncMod = node.async ? 'async ' : '';
    const star = node.generator ? '*' : '';
    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p)).join(', ');

    // Wrap return type based on async/generator semantics
    let returnType = '';
    if (node.returnType) {
        let baseType = g.genType(node.returnType);
        if (node.async && node.generator) {
            baseType = `AsyncGenerator<${baseType}>`;
        } else if (node.generator) {
            baseType = `Generator<${baseType}>`;
        } else if (node.async) {
            baseType = `Promise<${baseType}>`;
        }
        returnType = `: ${baseType}`;
    }

    // Track generator context for cede -> yield vs await
    const prevInGenerator = g.inGenerator;
    g.inGenerator = node.generator;
    const body = genBlockStatement(node.body, g);
    g.inGenerator = prevInGenerator;

    return `${g.ind()}${asyncMod}${star}${name}(${params})${returnType} ${body}`;
}
