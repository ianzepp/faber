/**
 * TypeScript Code Generator - IteratioStatement
 *
 * TRANSFORMS:
 *   ex 0..10 fixum i { } -> for (const i = 0; i < 10; i++) { }
 *   ex 0..10 per 2 varia i { } -> for (let i = 0; i < 10; i += 2) { }
 *   ex items fixum item { } -> for (const item of items) { }
 *   ex items varia item { } -> for (let item of items) { }
 *   de tabula fixum key { } -> for (const key of tabula.keys()) { }
 *
 * WHY: Range expressions compile to efficient traditional for loops
 *      instead of allocating arrays. The 'fixum' keyword generates
 *      const bindings, 'varia' generates let bindings.
 *
 *      tabula (Map) requires special handling because JavaScript Maps
 *      don't support for-in iteration - use .keys() instead.
 */

import type { IteratioStatement, CollectionDSLTransform, ArrayPattern } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

/**
 * Generate the variable binding pattern for a for-of loop.
 *
 * WHY: Supports both simple identifiers and array destructuring patterns.
 *      ex items pro item { }      -> for (const item of items)
 *      ex map fixum [k, v] { }    -> for (const [k, v] of map)
 */
function genLoopVariable(variable: IteratioStatement['variable']): string {
    if (variable.type === 'ArrayPattern') {
        const elements = variable.elements.map(el => {
            if (el.skip) return '_';
            if (el.rest) return `...${el.name.name}`;
            return el.name.name;
        });
        return `[${elements.join(', ')}]`;
    }
    return variable.name;
}

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
    const varName = genLoopVariable(node.variable);
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

    // Use let for mutable bindings, const for immutable
    const bindingKeyword = node.mutable ? 'let' : 'const';

    if (node.catchClause) {
        let result = `${g.ind()}try {\n`;

        g.depth++;
        result += `${g.ind()}for${awaitKeyword} (${bindingKeyword} ${varName} ${keyword} ${iterable}) ${body}`;
        g.depth--;
        result += `\n${g.ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`;

        return result;
    }

    return `${g.ind()}for${awaitKeyword} (${bindingKeyword} ${varName} ${keyword} ${iterable}) ${body}`;
}

/**
 * Apply DSL transforms as method calls.
 *
 * TRANSFORMS:
 *   prima 5                     -> .slice(0, 5)
 *   ultima 3                    -> .slice(-3)
 *   summa                       -> .reduce((a, b) => a + b, 0)
 *   summa pretium               -> .reduce((a, b) => a + b.pretium, 0)
 *   ordina per nomen            -> .toSorted((a, b) => a.nomen < b.nomen ? -1 : a.nomen > b.nomen ? 1 : 0)
 *   ordina per nomen descendens -> .toSorted((a, b) => b.nomen < a.nomen ? -1 : b.nomen > a.nomen ? 1 : 0)
 *   collige nomen               -> .map(_x => _x.nomen)
 *   grupa per categoria         -> Object.groupBy(arr, _x => _x.categoria)
 *   maximum                     -> Math.max(...arr)
 *   minimum                     -> Math.min(...arr)
 *   medium                      -> (arr.reduce((a, b) => a + b, 0) / arr.length)
 *   numera                      -> .length
 *
 * WHY: DSL verbs desugar to target-language collection operations.
 *      Uses toSorted() for immutable sort (ES2023+).
 *      Uses Object.groupBy() for grouping (ES2024+).
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
                // summa prop -> .reduce((a, b) => a + b.prop, 0)
                if (transform.property) {
                    const prop = g.genExpression(transform.property);
                    result = `${result}.reduce((a, b) => a + b.${prop}, 0)`;
                } else {
                    result = `${result}.reduce((a, b) => a + b, 0)`;
                }
                break;

            case 'ordina': {
                // ordina per prop -> .toSorted((a, b) => compare)
                // Uses toSorted() for immutable sort
                if (transform.property) {
                    const prop = g.genExpression(transform.property);
                    const desc = transform.direction === 'descendens';
                    const [first, second] = desc ? ['b', 'a'] : ['a', 'b'];
                    result = `${result}.toSorted((a, b) => ${first}.${prop} < ${second}.${prop} ? -1 : ${first}.${prop} > ${second}.${prop} ? 1 : 0)`;
                }
                break;
            }

            case 'collige':
                // collige prop -> .map(_x => _x.prop)
                if (transform.property) {
                    const prop = g.genExpression(transform.property);
                    result = `${result}.map(_x => _x.${prop})`;
                }
                break;

            case 'grupa':
                // grupa per prop -> Object.groupBy(arr, _x => _x.prop)
                if (transform.property) {
                    const prop = g.genExpression(transform.property);
                    result = `Object.groupBy(${result}, _x => _x.${prop})`;
                }
                break;

            case 'maximum':
                // maximum -> Math.max(...arr)
                result = `Math.max(...${result})`;
                break;

            case 'minimum':
                // minimum -> Math.min(...arr)
                result = `Math.min(...${result})`;
                break;

            case 'medium':
                // medium -> (arr.reduce((a, b) => a + b, 0) / arr.length)
                // Need to wrap in IIFE to avoid re-evaluating source
                result = `((_arr) => _arr.reduce((a, b) => a + b, 0) / _arr.length)(${result})`;
                break;

            case 'numera':
                // numera -> .length
                result = `${result}.length`;
                break;
        }
    }

    return result;
}
