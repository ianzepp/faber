/**
 * C++23 Code Generator - CallExpression
 *
 * TRANSFORMS:
 *   fn()         -> fn()
 *   fn?()        -> (fn ? (*fn)() : std::nullopt)  (for function pointers)
 *   fn!()        -> (*fn)()  (assert not null)
 *   lista.adde(x)      -> lista.push_back(x)
 *   lista.filtrata(fn) -> (lista | views::filter(fn) | ranges::to<vector>())
 *   _scribe(x)         -> std::println("{}", x)
 *   _vide(x)           -> std::cerr << "[DEBUG] " << x << std::endl
 *   _mone(x)           -> std::cerr << "[WARN] " << x << std::endl
 *   _lege()            -> std::getline(std::cin, ...)
 */

import type { CallExpression, Expression, Identifier } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

// WHY: Unified registries for collection methods (stdlib refactor)
import { getListaMethod } from '../../lista';
import { getTabulaMethod } from '../../tabula';
import { getCopiaMethod } from '../../copia';

import { getMathesisFunction, getMathesisHeaders } from '../norma/mathesis';
import { getAleatorFunction, getAleatorHeaders } from '../norma/aleator';

/**
 * C++23 tempus (time) intrinsic mappings.
 *
 * WHY: Time functions use <chrono> for time points and <thread> for sleep.
 */
const CPP_TEMPUS: Record<string, { cpp: string | ((args: string[]) => string); headers: string[] }> = {
    nunc: {
        cpp: 'std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::system_clock::now().time_since_epoch()).count()',
        headers: ['<chrono>'],
    },
    nunc_nano: {
        cpp: 'std::chrono::duration_cast<std::chrono::nanoseconds>(std::chrono::system_clock::now().time_since_epoch()).count()',
        headers: ['<chrono>'],
    },
    nunc_secunda: {
        cpp: 'std::chrono::duration_cast<std::chrono::seconds>(std::chrono::system_clock::now().time_since_epoch()).count()',
        headers: ['<chrono>'],
    },
    dormi: {
        cpp: (args: string[]) => `std::this_thread::sleep_for(std::chrono::milliseconds(${args[0]}))`,
        headers: ['<chrono>', '<thread>'],
    },
};

/**
 * C++23 I/O intrinsic mappings.
 *
 * WHY: Maps Latin I/O intrinsics to C++ equivalents.
 * - _scribe: Standard output via std::println (C++23) or std::cout
 * - _vide: Debug output to cerr with [DEBUG] prefix
 * - _mone: Warning output to cerr with [WARN] prefix
 * - _lege: Read line from stdin
 */
function genIntrinsic(name: string, argsArray: string[], g: CppGenerator): string | null {
    if (name === '_scribe') {
        g.includes.add('<print>');
        if (argsArray.length === 0) {
            return 'std::println("")';
        }
        // WHY: C++23 std::println uses {} format placeholders
        const placeholders = argsArray.map(() => '{}').join(' ');
        return `std::println("${placeholders}", ${argsArray.join(', ')})`;
    }

    if (name === '_vide') {
        g.includes.add('<iostream>');
        if (argsArray.length === 0) {
            return 'std::cerr << "[DEBUG]" << std::endl';
        }
        // WHY: Use stream insertion for multiple args
        const streamArgs = argsArray.map((a, i) => (i === 0 ? a : ` << " " << ${a}`)).join('');
        return `std::cerr << "[DEBUG] " << ${streamArgs} << std::endl`;
    }

    if (name === '_mone') {
        g.includes.add('<iostream>');
        if (argsArray.length === 0) {
            return 'std::cerr << "[WARN]" << std::endl';
        }
        const streamArgs = argsArray.map((a, i) => (i === 0 ? a : ` << " " << ${a}`)).join('');
        return `std::cerr << "[WARN] " << ${streamArgs} << std::endl`;
    }

    if (name === '_lege') {
        g.includes.add('<iostream>');
        g.includes.add('<string>');
        // WHY: C++ needs a variable to read into. We use a lambda to create scope.
        return '[&]{ std::string __line; std::getline(std::cin, __line); return __line; }()';
    }

    return null;
}

export function genCallExpression(node: CallExpression, g: CppGenerator): string {
    // WHY: Build both joined string (for simple cases) and array (for method handlers)
    // to preserve argument boundaries for multi-parameter lambdas containing commas.
    const argsArray = node.arguments.filter((arg): arg is Expression => arg.type !== 'SpreadElement').map(a => g.genExpression(a));
    const args = argsArray.join(', ');

    // Check for intrinsics (bare function calls)
    if (node.callee.type === 'Identifier') {
        const name = node.callee.name;

        const intrinsicResult = genIntrinsic(name, argsArray, g);
        if (intrinsicResult) {
            return intrinsicResult;
        }

        // Check mathesis functions (ex "norma/mathesis" importa pavimentum, etc.)
        const mathesisFunc = getMathesisFunction(name);
        if (mathesisFunc) {
            for (const header of getMathesisHeaders(name)) {
                g.includes.add(header);
            }
            if (typeof mathesisFunc.cpp === 'function') {
                return mathesisFunc.cpp(argsArray);
            }
            return mathesisFunc.cpp;
        }

        // Check tempus functions (ex "norma/tempus" importa nunc, dormi, etc.)
        const tempusFunc = CPP_TEMPUS[name];
        if (tempusFunc) {
            for (const header of tempusFunc.headers) {
                g.includes.add(header);
            }
            if (typeof tempusFunc.cpp === 'function') {
                return tempusFunc.cpp(argsArray);
            }
            return tempusFunc.cpp;
        }

        // Check aleator functions (ex "norma/aleator" importa fractus, inter, etc.)
        const aleatorFunc = getAleatorFunction(name);
        if (aleatorFunc) {
            for (const header of getAleatorHeaders(name)) {
                g.includes.add(header);
            }
            if (typeof aleatorFunc.cpp === 'function') {
                return aleatorFunc.cpp(argsArray);
            }
            return aleatorFunc.cpp;
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
        // TODO: Add header tracking to unified registries (currently C++ headers are not tracked)
        if (collectionName === 'tabula') {
            const method = getTabulaMethod(methodName);
            if (method) {
                if (typeof method.cpp === 'function') {
                    return method.cpp(obj, argsArray);
                }
                return `${obj}.${method.cpp}(${args})`;
            }
        } else if (collectionName === 'copia') {
            const method = getCopiaMethod(methodName);
            if (method) {
                if (typeof method.cpp === 'function') {
                    return method.cpp(obj, argsArray);
                }
                return `${obj}.${method.cpp}(${args})`;
            }
        } else if (collectionName === 'lista') {
            const method = getListaMethod(methodName);
            if (method) {
                if (typeof method.cpp === 'function') {
                    return method.cpp(obj, argsArray);
                }
                return `${obj}.${method.cpp}(${args})`;
            }
        }

        // Fallback: no type info - try lista (most common)
        const listaMethod = getListaMethod(methodName);
        if (listaMethod) {
            if (typeof listaMethod.cpp === 'function') {
                return listaMethod.cpp(obj, argsArray);
            }
            return `${obj}.${listaMethod.cpp}(${args})`;
        }
    }

    const callee = g.genExpression(node.callee);

    // WHY: For optional call, check if function pointer is valid
    if (node.optional) {
        g.includes.add('<optional>');
        return `(${callee} ? (*${callee})(${args}) : std::nullopt)`;
    }
    // WHY: For non-null assertion, dereference and call
    if (node.nonNull) {
        return `(*${callee})(${args})`;
    }
    return `${callee}(${args})`;
}
