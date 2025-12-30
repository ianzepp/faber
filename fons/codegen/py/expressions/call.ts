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
 */

import type { CallExpression, Expression, Identifier } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

// WHY: Unified registries for collection methods (stdlib refactor)
import { getListaMethod } from '../../lista';
import { getTabulaMethod } from '../../tabula';
import { getCopiaMethod } from '../../copia';

import { getMathesisFunction } from '../norma/mathesis';
import { getAleatorFunction } from '../norma/aleator';

/**
 * Python tempus (time) intrinsics.
 *
 * WHY: Time functions require the `time` module.
 */
const PY_TEMPUS: Record<string, (args: string[], g: PyGenerator) => string> = {
    nunc: (_, g) => {
        g.features.time = true;
        return `int(time.time() * 1000)`;
    },
    nunc_nano: (_, g) => {
        g.features.time = true;
        return `time.time_ns()`;
    },
    nunc_secunda: (_, g) => {
        g.features.time = true;
        return `int(time.time())`;
    },
    dormi: (args, g) => {
        g.features.time = true;
        return `time.sleep(${args[0]} / 1000)`;
    },
};

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

        // Check mathesis functions (ex "norma/mathesis" importa pavimentum, etc.)
        const mathesisFunc = getMathesisFunction(name);
        if (mathesisFunc) {
            if (mathesisFunc.requiresMath) {
                g.features.math = true;
            }
            if (typeof mathesisFunc.py === 'function') {
                return mathesisFunc.py(argsArray);
            }
            return mathesisFunc.py;
        }

        // Check aleator functions (ex "norma/aleator" importa fractus, etc.)
        const aleatorFunc = getAleatorFunction(name);
        if (aleatorFunc) {
            if (aleatorFunc.requiresRandom) {
                g.features.random = true;
            }
            if (aleatorFunc.requiresUuid) {
                g.features.uuid = true;
            }
            if (aleatorFunc.requiresSecrets) {
                g.features.secrets = true;
            }
            if (typeof aleatorFunc.py === 'function') {
                return aleatorFunc.py(argsArray);
            }
            return aleatorFunc.py;
        }

        // Check tempus functions (ex "norma/tempus" importa nunc, dormi, etc.)
        const tempusFunc = PY_TEMPUS[name];
        if (tempusFunc) {
            return tempusFunc(argsArray, g);
        }
    }

    // Check for collection methods (lista, tabula, copia)
    // WHY: Pass argsArray (not joined string) to method handlers
    //      so they can correctly handle multi-param lambdas with commas.
    if (node.callee.type === 'MemberExpression' && !node.callee.computed) {
        const methodName = (node.callee.property as Identifier).name;
        const obj = g.genExpression(node.callee.object);

        // Try lista methods
        const listaMethod = getListaMethod(methodName);
        if (listaMethod) {
            if (typeof listaMethod.py === 'function') {
                return listaMethod.py(obj, argsArray);
            }
            return `${obj}.${listaMethod.py}(${args})`;
        }

        // Try tabula methods
        const tabulaMethod = getTabulaMethod(methodName);
        if (tabulaMethod) {
            if (typeof tabulaMethod.py === 'function') {
                return tabulaMethod.py(obj, argsArray);
            }
            return `${obj}.${tabulaMethod.py}(${args})`;
        }

        // Try copia methods
        const copiaMethod = getCopiaMethod(methodName);
        if (copiaMethod) {
            if (typeof copiaMethod.py === 'function') {
                return copiaMethod.py(obj, argsArray);
            }
            return `${obj}.${copiaMethod.py}(${args})`;
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
