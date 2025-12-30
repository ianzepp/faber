/**
 * TypeScript Code Generator - Call Expression
 *
 * TRANSFORMS:
 *   scribe("hello") -> console.log("hello")
 *   _pavimentum(x) -> Math.floor(x)
 *   foo(x, y) -> foo(x, y)
 *   lista.adde(x) -> lista.push(x)
 *   lista.filtrata(fn) -> lista.filter(fn)
 *   f(sparge nums) -> f(...nums)
 *
 * Intrinsics are mapped to target-specific implementations.
 * Lista methods (Latin array methods) are translated to JS equivalents.
 */

import type { CallExpression, Identifier } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

// WHY: Unified registries for collection methods (stdlib refactor)
import { getListaMethod } from '../../lista';
import { getTabulaMethod } from '../../tabula';
import { getCopiaMethod } from '../../copia';

import { getMathesisFunction } from '../norma/mathesis';
import { getAleatorFunction } from '../norma/aleator';

/**
 * TypeScript intrinsic mappings.
 *
 * Maps Latin intrinsic names to TypeScript/JavaScript equivalents.
 *
 * Two categories:
 * - _prefixed: Internal intrinsics called from arca/norma/index.fab
 * - unprefixed: Direct stdlib functions (norma/tempus, etc.)
 */
const TS_INTRINSICS: Record<string, (args: string) => string> = {
    // I/O intrinsics (always available)
    _scribe: args => `console.log(${args})`,
    _vide: args => `console.debug(${args})`,
    _mone: args => `console.warn(${args})`,
    _lege: () => `prompt() ?? ""`,

    // norma/tempus - Time functions
    nunc: () => `Date.now()`,
    nunc_nano: () => `BigInt(Date.now()) * 1000000n`,
    nunc_secunda: () => `Math.floor(Date.now() / 1000)`,
    dormi: args => `new Promise(r => setTimeout(r, ${args}))`,
};

export function genCallExpression(node: CallExpression, g: TsGenerator): string {
    // WHY: Build args as array first, then join for regular calls.
    // Collection method handlers receive the array to preserve argument
    // boundaries (avoiding comma-in-lambda parsing issues).
    const argsArray = node.arguments.map(arg => {
        if (arg.type === 'SpreadElement') {
            return `...${g.genExpression(arg.argument)}`;
        }
        return g.genExpression(arg);
    });
    const args = argsArray.join(', ');

    // Check for intrinsics and stdlib functions (bare function calls)
    if (node.callee.type === 'Identifier') {
        const name = node.callee.name;

        // Check hardcoded intrinsics first
        const intrinsic = TS_INTRINSICS[name];
        if (intrinsic) {
            return intrinsic(args);
        }

        // Check mathesis functions (ex "norma/mathesis" importa pavimentum, etc.)
        const mathesisFunc = getMathesisFunction(name);
        if (mathesisFunc) {
            if (typeof mathesisFunc.ts === 'function') {
                return mathesisFunc.ts(argsArray);
            }
            return mathesisFunc.ts;
        }

        // Check aleator functions (ex "norma/aleator" importa fractus, etc.)
        const aleatorFunc = getAleatorFunction(name);
        if (aleatorFunc) {
            if (typeof aleatorFunc.ts === 'function') {
                return aleatorFunc.ts(argsArray);
            }
            return aleatorFunc.ts;
        }
    }

    // Check for collection methods (method calls on lista/tabula/copia)
    if (node.callee.type === 'MemberExpression' && !node.callee.computed) {
        const methodName = (node.callee.property as Identifier).name;
        const obj = g.genExpression(node.callee.object);

        // WHY: Use semantic type info to dispatch to correct collection registry.
        // This prevents method name collisions (e.g., accipe means different
        // things for lista vs tabula).
        const objType = node.callee.object.resolvedType;
        const collectionName = objType?.kind === 'generic' ? objType.name : null;

        // Dispatch based on resolved type
        // WHY: Pass argsArray (not joined string) to method handlers
        //      so they can correctly handle multi-param lambdas with commas.
        if (collectionName === 'tabula') {
            const method = getTabulaMethod(methodName);
            if (method) {
                if (typeof method.ts === 'function') {
                    return method.ts(obj, argsArray);
                }
                return `${obj}.${method.ts}(${args})`;
            }
        } else if (collectionName === 'copia') {
            const method = getCopiaMethod(methodName);
            if (method) {
                if (typeof method.ts === 'function') {
                    return method.ts(obj, argsArray);
                }
                return `${obj}.${method.ts}(${args})`;
            }
        } else if (collectionName === 'lista') {
            const method = getListaMethod(methodName);
            if (method) {
                if (typeof method.ts === 'function') {
                    return method.ts(obj, argsArray);
                }
                return `${obj}.${method.ts}(${args})`;
            }
        }

        // Fallback: no type info or unknown type - try lista (most common)
        const listaMethod = getListaMethod(methodName);
        if (listaMethod) {
            if (typeof listaMethod.ts === 'function') {
                return listaMethod.ts(obj, argsArray);
            }
            return `${obj}.${listaMethod.ts}(${args})`;
        }
    }

    const callee = g.genExpression(node.callee);

    // Handle optional call: callback?() -> callback?.()
    if (node.optional) {
        return `${callee}?.(${args})`;
    }

    // Handle non-null assertion call: handler!() -> handler!()
    if (node.nonNull) {
        return `${callee}!(${args})`;
    }

    return `${callee}(${args})`;
}
