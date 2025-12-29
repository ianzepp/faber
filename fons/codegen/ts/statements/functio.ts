/**
 * TypeScript Code Generator - FunctioDeclaration
 *
 * TRANSFORMS:
 *   functio salve(nomen: textus): nihil -> function salve(nomen: string): null
 *   futura functio f(): numerus -> async function f(): Promise<number>
 *   cursor functio f(): numerus -> function* f(): Generator<number>
 *   futura cursor functio f(): numerus -> async function* f(): AsyncGenerator<number>
 *
 * FLUMINA (streams-first):
 *   functio f() fit T { redde x } -> function f(): T { return drain(function* () { yield respond.ok(x); }); }
 *
 * WHY: `fit` verb triggers flumina (stream protocol), `->` uses direct return.
 *      This allows opt-in streaming for functions that benefit from the protocol,
 *      while keeping zero overhead for simple functions using `->`.
 */

import type { FunctioDeclaration, BlockStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genFunctioDeclaration(node: FunctioDeclaration, g: TsGenerator): string {
    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p)).join(', ');

    // Generate type parameters: prae typus T -> <T>
    const typeParams = node.typeParams ? g.genTypeParams(node.typeParams) : '';

    // WHY: Only `fit` verb triggers flumina, not `->` arrow syntax
    // This allows developers to opt-in to streaming protocol when needed
    const useFlumina = node.returnVerb === 'fit' && !node.isConstructor;

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
        // WHY: fit functions keep their original return type (drain unwraps internally)
        returnType = `: ${baseType}`;
    }

    // Track context for nested statement generation
    const prevInGenerator = g.inGenerator;
    const prevInFlumina = g.inFlumina;

    g.inGenerator = node.generator;

    if (useFlumina) {
        // WHY: Enable flumina mode so redde/iace emit yield respond.ok/error
        g.inFlumina = true;
        g.features.flumina = true;

        // Generate inner body statements
        g.depth++;
        const innerBody = node.body.body.map(stmt => g.genStatement(stmt)).join('\n');
        g.depth--;

        g.inFlumina = prevInFlumina;
        g.inGenerator = prevInGenerator;

        // WHY: Wrap body in drain(function* () { ... }) for Responsum protocol
        const ind = g.ind();
        return `${ind}function ${name}${typeParams}(${params})${returnType} {
${ind}  return drain(function* () {
${innerBody}
${ind}  });
${ind}}`;
    }

    // Non-flumina path: arrow syntax, async, generator, or constructor
    const async = node.async ? 'async ' : '';
    const star = node.generator ? '*' : '';
    const body = genBlockStatement(node.body, g);

    g.inGenerator = prevInGenerator;
    g.inFlumina = prevInFlumina;

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
