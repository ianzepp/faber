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
 * Collection methods are translated via the unified norma registry.
 */

import type { CallExpression, Identifier } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

// WHY: Unified norma registry for all stdlib translations (from .fab files)
import { getNormaTranslation, applyNormaTemplate, applyNormaModuleCall } from '../../norma-registry';

/**
 * TypeScript intrinsic mappings.
 *
 * Maps Latin I/O intrinsic names to TypeScript/JavaScript equivalents.
 * These are always available without imports.
 */
const TS_INTRINSICS: Record<string, (args: string) => string> = {
    _scribe: args => `console.log(${args})`,
    _vide: args => `console.debug(${args})`,
    _mone: args => `console.warn(${args})`,
    _lege: () => `prompt() ?? ""`,
};

export function genCallExpression(node: CallExpression, g: TsGenerator): string {
    // GUARD: Optional computed call on tabula index: m?[k]()
    // WHY: m?[k]() is an optional call chain; when tabula maps to Map.get(),
    //      we need `(m?.get(k))?.()` not `m?.get(k)()`.
    if (node.callee.type === 'MemberExpression' && node.callee.computed && node.callee.optional) {
        const objType = node.callee.object.resolvedType;
        const collectionName = objType?.kind === 'generic' ? objType.name : null;

        if (collectionName === 'tabula') {
            const obj = g.genExpression(node.callee.object);
            const prop = g.genBareExpression(node.callee.property);

            const argsArray = node.arguments.map(arg => {
                if (arg.type === 'SpreadElement') {
                    return `...${g.genExpression(arg.argument)}`;
                }
                return g.genExpression(arg);
            });
            const args = argsArray.join(', ');

            return `(${obj}?.get(${prop}))?.(${args})`;
        }
    }

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

        // Check I/O intrinsics first (always available)
        const intrinsic = TS_INTRINSICS[name];
        if (intrinsic) {
            return intrinsic(args);
        }

        // Check norma module functions (mathesis, solum, aleator, tempus)
        // WHY: These are imported via `ex "norma/mathesis" importa pavimentum`
        for (const module of ['mathesis', 'tempus', 'aleator']) {
            const call = applyNormaModuleCall('ts', module, name, [...argsArray]);
            if (call) {
                return call;
            }
        }

        // Check solum separately for feature flagging
        const solumCall = applyNormaModuleCall('ts', 'solum', name, [...argsArray]);
        if (solumCall) {
            g.features.fs = true;
            if (['iunge', 'dir', 'basis', 'extensio', 'resolve'].includes(name)) {
                g.features.nodePath = true;
            }
            return solumCall;
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

        // Try norma registry for the resolved collection type
        if (collectionName) {
            const norma = getNormaTranslation('ts', collectionName, methodName);
            if (norma) {
                if (norma.method) {
                    return `${obj}.${norma.method}(${args})`;
                }
                if (norma.template && norma.params) {
                    return applyNormaTemplate(norma.template, [...norma.params], obj, [...argsArray]);
                }
            }
        }

        // Fallback: no type info - try all collection types
        for (const coll of ['lista', 'tabula', 'copia']) {
            const norma = getNormaTranslation('ts', coll, methodName);
            if (norma) {
                if (norma.method) {
                    return `${obj}.${norma.method}(${args})`;
                }
                if (norma.template && norma.params) {
                    return applyNormaTemplate(norma.template, [...norma.params], obj, [...argsArray]);
                }
            }
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
