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
 *   functio f() fit T { redde x }   -> function f(): T { return drain(function* () { yield respond.ok(x); }); }
 *   functio f() fiunt T { cede x }  -> function* f(): Generator<T> { yield* flow((function* () { yield respond.item(x); yield respond.done(); })()); }
 *   functio f() fiet T { redde x }  -> async function f(): Promise<T> { return await drainAsync(async function* () { yield respond.ok(x); }); }
 *   functio f() fient T { cede x }  -> async function* f(): AsyncGenerator<T> { yield* flowAsync((async function* () { yield respond.item(x); yield respond.done(); })()); }
 *
 * WHY: `fit`/`fiet` trigger drain/drainAsync (single-value), `fiunt`/`fient` trigger flow/flowAsync (multi-value).
 *      All use Responsum protocol internally, but caller sees raw values.
 *      `->` arrow syntax uses direct return with zero overhead.
 */

import type { FunctioDeclaration, BlockStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import {
    getVisibilityFromAnnotations,
    isAbstractFromAnnotations,
    isAsyncFromAnnotations,
    isGeneratorFromAnnotations,
    isStaticFromAnnotations,
} from '../../types';

export function genFunctioDeclaration(node: FunctioDeclaration, g: TsGenerator): string {
    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p)).join(', ');

    // Generate type parameters: prae typus T -> <T>
    const typeParams = node.typeParams ? g.genTypeParams(node.typeParams) : '';

    // Module-level: export when public (inClass is false for top-level functions)
    const visibility = getVisibilityFromAnnotations(node.annotations);
    const exportMod = !g.inClass && visibility === 'public' ? 'export ' : '';

    // Derive async/generator from annotations OR node properties
    // WHY: @ futura and @ cursor annotations are equivalent to inline futura/cursor modifiers
    const isAsync = node.async || isAsyncFromAnnotations(node.annotations);
    const isGenerator = node.generator || isGeneratorFromAnnotations(node.annotations);

    // Handle abstract methods (no body)
    if (node.isAbstract || !node.body) {
        const returnType = node.returnType ? `: ${g.genType(node.returnType)}` : '';
        return `${g.ind()}abstract ${name}${typeParams}(${params})${returnType};`;
    }

    // WHY: Each verb triggers its corresponding helper:
    // fit -> drain() (sync single), fiunt -> flow() (sync multi)
    // fiet -> drainAsync() (async single), fient -> flowAsync() (async multi)
    const useFit = node.returnVerb === 'fit' && !node.isConstructor;
    const useFiunt = node.returnVerb === 'fiunt' && !node.isConstructor;
    const useFiet = node.returnVerb === 'fiet' && !node.isConstructor;
    const useFient = node.returnVerb === 'fient' && !node.isConstructor;

    // Wrap return type based on async/generator semantics
    let returnType = '';
    if (node.returnType) {
        let baseType = g.genType(node.returnType);
        if (isAsync && isGenerator) {
            baseType = `AsyncGenerator<${baseType}>`;
        } else if (isGenerator || useFiunt) {
            // WHY: fiunt functions return Generator<T> (flow() unwraps internally)
            baseType = `Generator<${baseType}>`;
        } else if (useFient) {
            // WHY: fient functions return AsyncGenerator<T> (flowAsync() unwraps internally)
            baseType = `AsyncGenerator<${baseType}>`;
        } else if (isAsync || useFiet) {
            // WHY: fiet functions return Promise<T> (drainAsync() unwraps internally)
            baseType = `Promise<${baseType}>`;
        }
        returnType = `: ${baseType}`;
    }

    // Track context for nested statement generation
    const prevInGenerator = g.inGenerator;
    const prevInFlumina = g.inFlumina;
    const prevInFiunt = g.inFiunt;
    const prevInFiet = g.inFiet;
    const prevInFient = g.inFient;

    g.inGenerator = isGenerator;

    if (useFit) {
        // WHY: Enable flumina mode so redde/iace emit yield respond.ok/error
        g.inFlumina = true;
        g.features.flumina = true;

        // Generate inner body statements
        g.depth++;
        const innerBody = node.body.body.map(stmt => g.genStatement(stmt)).join('\n');
        g.depth--;

        g.inFlumina = prevInFlumina;
        g.inGenerator = prevInGenerator;

        // WHY: Wrap body in asFit(function* () { ... }) for Responsum protocol
        const ind = g.ind();
        return `${ind}${exportMod}function ${name}${typeParams}(${params})${returnType} {
${ind}  return asFit(function* () {
${innerBody}
${ind}  });
${ind}}`;
    }

    if (useFiunt) {
        // WHY: Enable fiunt mode so cede emits yield respond.item(), iace emits yield respond.error()
        g.inFiunt = true;
        g.inGenerator = true; // cede should emit yield, not await
        g.features.flumina = true;

        // Generate inner body statements
        g.depth++;
        const innerBody = node.body.body.map(stmt => g.genStatement(stmt)).join('\n');
        g.depth--;

        g.inFiunt = prevInFiunt;
        g.inGenerator = prevInGenerator;

        // WHY: Wrap body in asFiunt((function* () { ... yield respond.done(); })()) for Responsum protocol
        // The implicit respond.done() signals stream completion
        const ind = g.ind();
        return `${ind}${exportMod}function* ${name}${typeParams}(${params})${returnType} {
${ind}  yield* asFiunt((function* () {
${innerBody}
${ind}    yield respond.done();
${ind}  })());
${ind}}`;
    }

    if (useFiet) {
        // WHY: Enable fiet mode so redde/iace emit yield respond.ok/error (async version)
        g.inFiet = true;
        g.inFlumina = true; // Reuse flumina logic for redde/iace
        g.features.flumina = true;

        // Generate inner body statements
        g.depth++;
        const innerBody = node.body.body.map(stmt => g.genStatement(stmt)).join('\n');
        g.depth--;

        g.inFiet = prevInFiet;
        g.inFlumina = prevInFlumina;
        g.inGenerator = prevInGenerator;

        // WHY: Wrap body in asFiet(async function* () { ... }) for async Responsum protocol
        const ind = g.ind();
        return `${ind}${exportMod}async function ${name}${typeParams}(${params})${returnType} {
${ind}  return await asFiet(async function* () {
${innerBody}
${ind}  });
${ind}}`;
    }

    if (useFient) {
        // WHY: Enable fient mode so cede emits yield respond.item(), iace emits yield respond.error()
        g.inFient = true;
        g.inGenerator = true; // For iace to emit yield respond.error
        g.features.flumina = true;

        // Generate inner body statements
        g.depth++;
        const innerBody = node.body.body.map(stmt => g.genStatement(stmt)).join('\n');
        g.depth--;

        g.inFient = prevInFient;
        g.inGenerator = prevInGenerator;

        // WHY: Wrap body in asFient((async function* () { ... yield respond.done(); })()) for async Responsum protocol
        const ind = g.ind();
        return `${ind}${exportMod}async function* ${name}${typeParams}(${params})${returnType} {
${ind}  yield* asFient((async function* () {
${innerBody}
${ind}    yield respond.done();
${ind}  })());
${ind}}`;
    }

    // Non-flumina path: arrow syntax, async, generator, or constructor
    const asyncMod = isAsync ? 'async ' : '';
    const star = isGenerator ? '*' : '';
    const body = genBlockStatement(node.body, g);

    g.inGenerator = prevInGenerator;
    g.inFlumina = prevInFlumina;
    g.inFiunt = prevInFiunt;
    g.inFiet = prevInFiet;
    g.inFient = prevInFient;

    return `${g.ind()}${exportMod}${asyncMod}function${star} ${name}${typeParams}(${params})${returnType} ${body}`;
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
    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p)).join(', ');

    // Derive async/generator from annotations OR node properties
    const isAsync = node.async || isAsyncFromAnnotations(node.annotations);
    const isGenerator = node.generator || isGeneratorFromAnnotations(node.annotations);

    // Wrap return type based on async/generator semantics
    let returnType = '';
    if (node.returnType) {
        let baseType = g.genType(node.returnType);
        if (isAsync && isGenerator) {
            baseType = `AsyncGenerator<${baseType}>`;
        } else if (isGenerator) {
            baseType = `Generator<${baseType}>`;
        } else if (isAsync) {
            baseType = `Promise<${baseType}>`;
        }
        returnType = `: ${baseType}`;
    }

    // Get visibility from annotations
    const visibility = getVisibilityFromAnnotations(node.annotations);
    const isAbstract = isAbstractFromAnnotations(node.annotations) || node.isAbstract;

    // Handle abstract methods (no body)
    if (isAbstract || !node.body) {
        const visibilityMod = visibility === 'protected' ? 'protected ' : '';
        return `${g.ind()}${visibilityMod}abstract ${name}(${params})${returnType};`;
    }

    const asyncMod = isAsync ? 'async ' : '';
    const star = isGenerator ? '*' : '';
    const visibilityMod = visibility === 'private' ? 'private ' : visibility === 'protected' ? 'protected ' : '';
    const isStatic = isStaticFromAnnotations(node.annotations);
    const staticMod = isStatic ? 'static ' : '';

    // Track generator context for cede -> yield vs await
    const prevInGenerator = g.inGenerator;
    g.inGenerator = isGenerator;
    const body = genBlockStatement(node.body, g);
    g.inGenerator = prevInGenerator;

    return `${g.ind()}${visibilityMod}${staticMod}${asyncMod}${star}${name}(${params})${returnType} ${body}`;
}
