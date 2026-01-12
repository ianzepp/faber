/**
 * Python Code Generator - CallExpression
 *
 * TRANSFORMS:
 *   fn()    -> fn()
 *   fn?()   -> (fn() if fn is not None else None)
 *   fn!()   -> fn()  (Python has no assertion, just call)
 *   f(sparge nums) -> f(*nums)
 *
 * WHY: Python uses * for unpacking iterables in function calls.
 * Collection methods are translated via the unified norma registry.
 */

import type { CallExpression, Identifier } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

// WHY: Unified norma registry for all stdlib translations (from .fab files)
import { getNormaTranslation, applyNormaTemplate, applyNormaModuleCall, validateMorphology } from '../../norma-registry';
import { applyNamespaceTemplate, getNamespaceTranslation, isNamespaceCall } from '../../shared/norma-namespace';

/**
 * Python I/O intrinsic handler.
 *
 * WHY: I/O intrinsics need to set feature flags for imports.
 * - _scribe: print() - no imports needed
 * - _vide: print(file=sys.stderr) - needs sys import
 * - _mone: warnings.warn() - needs warnings import
 * - _lege: input() - no imports needed
 */
function genIntrinsic(name: string, args: string, g: PyGenerator): string | null {
    if (name === '_scribe') {
        return `print(${args})`;
    }

    if (name === '_vide') {
        g.features.sys = true;
        return `print(${args}, file=sys.stderr)`;
    }

    if (name === '_mone') {
        g.features.warnings = true;
        return `warnings.warn(${args})`;
    }

    if (name === '_lege') {
        return 'input()';
    }

    return null;
}

export function genCallExpression(node: CallExpression, g: PyGenerator): string {
    // WHY: Build args as array first, then join for regular calls.
    // Collection method handlers receive the array to preserve argument
    // boundaries (avoiding comma-in-lambda parsing issues).
    const argsArray = node.arguments.map(arg => {
        if (arg.type === 'SpreadElement') {
            return `*${g.genExpression(arg.argument)}`;
        }
        return g.genExpression(arg);
    });
    const args = argsArray.join(', ');

    // Check for intrinsics and stdlib functions (bare function calls)
    if (node.callee.type === 'Identifier') {
        const name = node.callee.name;

        // Check I/O intrinsics first
        const intrinsicResult = genIntrinsic(name, args, g);
        if (intrinsicResult) {
            return intrinsicResult;
        }

        // Check norma module functions (mathesis, tempus, aleator)
        // WHY: Feature flags for Python imports are set based on module
        for (const module of ['mathesis', 'tempus', 'aleator']) {
            const call = applyNormaModuleCall('py', module, name, [...argsArray]);
            if (call) {
                // Set feature flags for Python imports
                if (module === 'mathesis') {
                    g.features.math = true;
                } else if (module === 'tempus') {
                    g.features.time = true;
                } else if (module === 'aleator') {
                    g.features.random = true;
                    if (name === 'uuid') g.features.uuid = true;
                    if (name === 'octeti') g.features.secrets = true;
                }
                return call;
            }
        }
    }

    if (isNamespaceCall(node)) {
        const moduleName = node.callee.object.resolvedType.moduleName;
        const methodName = (node.callee.property as Identifier).name;
        const translation = getNamespaceTranslation(node.callee, 'py');
        if (translation) {
            if (moduleName === 'mathesis') {
                g.features.math = true;
            } else if (moduleName === 'tempus') {
                g.features.time = true;
            } else if (moduleName === 'aleator') {
                g.features.random = true;
                if (methodName === 'uuid') g.features.uuid = true;
                if (methodName === 'octeti') g.features.secrets = true;
            } else if (moduleName === 'json') {
                g.features.json = true;
            }

            if (translation.method) {
                return `${translation.method}(${args})`;
            }
            if (translation.template) {
                return applyNamespaceTemplate(translation.template, [...argsArray]);
            }
        }
    }

    // Check for collection methods (lista, tabula, copia)
    if (node.callee.type === 'MemberExpression' && !node.callee.computed) {
        const methodName = (node.callee.property as Identifier).name;
        const obj = g.genExpression(node.callee.object);

        // WHY: Use semantic type info to dispatch to correct collection registry.
        const objType = node.callee.object.resolvedType;
        const collectionName = objType?.kind === 'generic' ? objType.name : null;

        // WHY: Skip stdlib check entirely for user-defined types. User genus methods
        // should never match stdlib collections, even if method names coincide.
        if (objType?.kind === 'user') {
            // Pass through to normal method call emission below
            return `${obj}.${methodName}(${args})`;
        }

        // Try norma registry for the resolved collection type
        if (collectionName) {
            // WHY: Validate morphology before translation. Catches undefined forms.
            const validation = validateMorphology(collectionName, methodName);
            if (!validation.valid) {
                return `# MORPHOLOGY: ${validation.error}\n${obj}.${methodName}(${args})`;
            }

            const norma = getNormaTranslation('py', collectionName, methodName);
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
            const norma = getNormaTranslation('py', coll, methodName);
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

    // WHY: Optional chaining on the callee (e.g., obj?[key]() or obj?.method())
    //      requires wrapping the entire call in a conditional, not just the member access.
    //      We detect optional member expressions and generate them without the wrapper,
    //      then wrap the whole call expression instead.
    if (node.callee.type === 'MemberExpression' && node.callee.optional) {
        const obj = g.genExpression(node.callee.object);
        const prop = node.callee.computed ? `[${g.genBareExpression(node.callee.property)}]` : `.${(node.callee.property as Identifier).name}`;

        return `(${obj}${prop}(${args}) if ${obj} is not None else None)`;
    }

    const callee = g.genExpression(node.callee);

    // WHY: Python has no native optional chaining; expand to conditional
    if (node.optional) {
        return `(${callee}(${args}) if ${callee} is not None else None)`;
    }
    // WHY: Python has no non-null assertion; just call directly
    return `${callee}(${args})`;
}
