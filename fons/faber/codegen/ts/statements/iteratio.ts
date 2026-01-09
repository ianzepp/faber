/**
 * TypeScript Code Generator - IteratioStatement
 *
 * TRANSFORMS:
 *   ex 0..10 pro i { } -> for (let i = 0; i < 10; i++) { }
 *   ex 0..10 per 2 pro i { } -> for (let i = 0; i < 10; i += 2) { }
 *   ex items pro item { } -> for (const item of items) { }
 *   ex items fit item { } -> for (const item of items) { }
 *   ex stream fiet chunk { } -> for await (const chunk of stream) { }
 *   de tabula pro key { } -> for (const key of tabula.keys()) { }
 *
 * WHY: Range expressions compile to efficient traditional for loops
 *      instead of allocating arrays. The 'fiet' verb form generates
 *      'for await' for async iteration.
 *
 *      tabula (Map) requires special handling because JavaScript Maps
 *      don't support for-in iteration - use .keys() instead.
 */

import type { IteratioStatement, CollectionDSLTransform } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

/**
 * Check if an expression has tabula (Map) type.
 *
 * WHY: JavaScript Map doesn't support for-in iteration or bracket indexing.
 *      We need to detect tabula types to generate correct iteration code.
 */
function isTabulaType(node: IteratioStatement): boolean {
    const type = node.iterable.resolvedType;
    return type?.kind === 'generic' && type.name === 'tabula';
}

export function genIteratioStatement(node: IteratioStatement, g: TsGenerator): string {
    const varName = node.variable.name;
    const body = genBlockStatement(node.body, g);
    const awaitKeyword = node.async ? ' await' : '';

    // Check if iterable is a range expression for efficient loop generation
    if (node.iterable.type === 'RangeExpression') {
        const range = node.iterable;
        const start = g.genExpression(range.start);
        const end = g.genExpression(range.end);
        const cmp = range.inclusive ? '<=' : '<';

        let forHeader: string;

        if (range.step) {
            const step = g.genExpression(range.step);

            // With step: need to handle positive/negative direction
            // For simplicity, assume positive step uses </<= based on inclusive
            forHeader = `for${awaitKeyword} (let ${varName} = ${start}; ${varName} ${cmp} ${end}; ${varName} += ${step})`;
        } else {
            // Default step of 1
            forHeader = `for${awaitKeyword} (let ${varName} = ${start}; ${varName} ${cmp} ${end}; ${varName}++)`;
        }

        if (node.catchClause) {
            let result = `${g.ind()}try {\n`;

            g.depth++;
            result += `${g.ind()}${forHeader} ${body}`;
            g.depth--;
            result += `\n${g.ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`;

            return result;
        }

        return `${g.ind()}${forHeader} ${body}`;
    }

    // Standard for-of/for-in loop
    let iterable = g.genExpression(node.iterable);

    // WHY: JavaScript Maps don't support for-in iteration.
    //      When iterating keys with 'de tabula pro key', we must use .keys() method.
    //      for (const key in map) produces nothing; for (const key of map.keys()) works.
    const isTabula = isTabulaType(node);
    let keyword: string;
    if (node.kind === 'in') {
        if (isTabula) {
            // tabula: use for-of with .keys()
            iterable = `${iterable}.keys()`;
            keyword = 'of';
        } else {
            // Regular object: use for-in
            keyword = 'in';
        }
    } else {
        keyword = 'of';
    }

    // Apply DSL transforms as method chain
    if (node.transforms && node.transforms.length > 0) {
        iterable = applyDSLTransforms(iterable, node.transforms, g);
    }

    if (node.catchClause) {
        let result = `${g.ind()}try {\n`;

        g.depth++;
        result += `${g.ind()}for${awaitKeyword} (const ${varName} ${keyword} ${iterable}) ${body}`;
        g.depth--;
        result += `\n${g.ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`;

        return result;
    }

    return `${g.ind()}for${awaitKeyword} (const ${varName} ${keyword} ${iterable}) ${body}`;
}

/**
 * Apply DSL transforms as method calls.
 *
 * TRANSFORMS:
 *   prima 5      -> .slice(0, 5)
 *   ultima 3     -> .slice(-3)
 *   summa        -> .reduce((a, b) => a + b, 0)
 *
 * WHY: DSL verbs desugar to norma method implementations.
 */
export function applyDSLTransforms(source: string, transforms: CollectionDSLTransform[], g: TsGenerator): string {
    let result = source;

    for (const transform of transforms) {
        switch (transform.verb) {
            case 'prima':
                // prima N -> .slice(0, N)
                if (transform.argument) {
                    const n = g.genExpression(transform.argument);
                    result = `${result}.slice(0, ${n})`;
                }
                break;
            case 'ultima':
                // ultima N -> .slice(-N)
                if (transform.argument) {
                    const n = g.genExpression(transform.argument);
                    result = `${result}.slice(-${n})`;
                }
                break;
            case 'summa':
                // summa -> .reduce((a, b) => a + b, 0)
                result = `${result}.reduce((a, b) => a + b, 0)`;
                break;
        }
    }

    return result;
}
