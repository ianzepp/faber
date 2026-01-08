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
 *
 * Collection methods are translated via the unified norma registry.
 */

import type { CallExpression, Expression, Identifier } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

// WHY: Unified norma registry for all stdlib translations (from .fab files)
import { getNormaTranslation, applyNormaTemplate, applyNormaModuleCall, validateMorphology } from '../../norma-registry';

// WHY: C++ requires header tracking for imports. Map module/function to headers.
const CPP_HEADERS: Record<string, Record<string, string[]>> = {
    mathesis: {
        _default: ['<cmath>'],
    },
    tempus: {
        _default: ['<chrono>'],
        dormi: ['<chrono>', '<thread>'],
    },
    aleator: {
        _default: ['<random>'],
        uuid: ['<random>', '<string>'],
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

        // Check norma module functions (mathesis, tempus, aleator)
        for (const module of ['mathesis', 'tempus', 'aleator']) {
            const call = applyNormaModuleCall('cpp', module, name, [...argsArray]);
            if (call) {
                // Add required headers
                const moduleHeaders = CPP_HEADERS[module];
                if (moduleHeaders) {
                    const headers = moduleHeaders[name] || moduleHeaders._default || [];
                    for (const header of headers) {
                        g.includes.add(header);
                    }
                }
                return call;
            }
        }
    }

    // Check for collection methods (method calls on lista/tabula/copia)
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
                return `/* MORPHOLOGY: ${validation.error} */ ${obj}.${methodName}(${args})`;
            }

            const norma = getNormaTranslation('cpp', collectionName, methodName);
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
            const norma = getNormaTranslation('cpp', coll, methodName);
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
