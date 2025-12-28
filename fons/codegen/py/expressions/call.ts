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
import { getListaMethod } from '../norma/lista';
import { getTabulaMethod } from '../norma/tabula';
import { getCopiaMethod } from '../norma/copia';

/**
 * Python intrinsic mappings.
 */
export const PY_INTRINSICS: Record<string, (args: string) => string> = {
    _scribe: args => `print(${args})`,
    _vide: args => `print(${args}, file=sys.stderr)`,
    _mone: args => `warnings.warn(${args})`,
    _lege: () => `input()`,
    _fortuitus: () => `random.random()`,
    _pavimentum: args => `math.floor(${args})`,
    _tectum: args => `math.ceil(${args})`,
    _radix: args => `math.sqrt(${args})`,
    _potentia: args => `math.pow(${args})`,
};

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

    // Check for intrinsics
    if (node.callee.type === 'Identifier') {
        const name = node.callee.name;
        const intrinsic = PY_INTRINSICS[name];
        if (intrinsic) {
            return intrinsic(args);
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

    const callee = g.genExpression(node.callee);

    // WHY: Python has no native optional chaining; expand to conditional
    if (node.optional) {
        return `(${callee}(${args}) if ${callee} is not None else None)`;
    }
    // WHY: Python has no non-null assertion; just call directly
    return `${callee}(${args})`;
}
