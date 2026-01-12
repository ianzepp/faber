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
import {
    getNormaTranslation,
    applyNormaTemplate,
    applyNormaModuleCall,
    validateMorphology,
    getNormaReceiverCollectionsForMethod,
} from '../../norma-registry';
import { applyNamespaceTemplate, getNamespaceTranslation, isNamespaceCall } from '../../shared/norma-namespace';

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

    if (isNamespaceCall(node)) {
        const objType = node.callee.object.resolvedType;
        if (!objType || objType.kind !== 'namespace') {
            // Fall through to other handlers
        } else {
            const moduleName = objType.moduleName;
            const methodName = (node.callee.property as Identifier).name;
            const translation = getNamespaceTranslation(node.callee, 'ts');
            if (translation) {
                if (moduleName === 'solum') {
                    g.features.fs = true;
                    if (['iunge', 'dir', 'basis', 'extensio', 'resolve'].includes(methodName)) {
                        g.features.nodePath = true;
                    }
                }
                if (translation.method) {
                    return `${translation.method}(${args})`;
                }
                if (translation.template) {
                    return applyNamespaceTemplate(translation.template, [...argsArray]);
                }
            }
        }
    }

    // Check for collection methods (method calls on lista/tabula/copia)
    if (node.callee.type === 'MemberExpression' && !node.callee.computed) {
        const methodName = (node.callee.property as Identifier).name;
        const obj = g.genExpression(node.callee.object);

        // WHY: Use semantic type info to dispatch to correct collection registry.
        // This prevents method name collisions (e.g., accipe means different
        // things for lista vs tabula). Also handles primitives like textus.
        const objType = node.callee.object.resolvedType;
        const collectionName = objType?.kind === 'generic' ? objType.name : objType?.kind === 'primitive' ? objType.name : null;

        // WHY: Skip stdlib check entirely for user-defined types. User genus methods
        // should never match stdlib collections, even if method names coincide.
        if (objType?.kind === 'user') {
            // Pass through to normal method call emission below
            return `${obj}.${methodName}(${args})`;
        }

        // STRICT: No fallback guessing. If this method name is known to norma
        // for receiver types but we can't resolve the receiver type, emit an
        // actionable compiler error.
        if (!collectionName) {
            const candidates = getNormaReceiverCollectionsForMethod('ts', methodName);
            if (candidates.length > 0) {
                g.codegenError(
                    `Cannot translate stdlib method '${methodName}()' without resolved receiver type (candidates: ${candidates.join(
                        ', ',
                    )}). Add a type annotation, import the return type, or fix upstream type propagation.`,
                    node,
                );
            }
        }

        // Try norma registry for the resolved collection type
        if (collectionName) {
            // WHY: Validate morphology before translation. Catches undefined forms
            // like 'filtratura' when only 'imperativus, perfectum' are declared.
            const validation = validateMorphology(collectionName, methodName);
            if (!validation.valid) {
                // Emit warning comment but continue - method might work anyway
                return `/* MORPHOLOGY: ${validation.error} */ ${obj}.${methodName}(${args})`;
            }

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

        // WHY: No fallback guessing. If type isn't resolved to a known collection,
        //      preserve the Latin method name. Require type annotations for translation.
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
