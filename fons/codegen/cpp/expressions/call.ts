/**
 * C++23 Code Generator - CallExpression
 *
 * TRANSFORMS:
 *   fn()         -> fn()
 *   fn?()        -> (fn ? (*fn)() : std::nullopt)  (for function pointers)
 *   fn!()        -> (*fn)()  (assert not null)
 *   lista.adde(x)      -> lista.push_back(x)
 *   lista.filtrata(fn) -> (lista | views::filter(fn) | ranges::to<vector>())
 */

import type { CallExpression, Expression, Identifier } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { getListaMethod, getListaHeaders } from '../norma/lista';
import { getTabulaMethod, getTabulaHeaders } from '../norma/tabula';
import { getCopiaMethod, getCopiaHeaders } from '../norma/copia';

export function genCallExpression(node: CallExpression, g: CppGenerator): string {
    // WHY: Build both joined string (for simple cases) and array (for method handlers)
    // to preserve argument boundaries for multi-parameter lambdas containing commas.
    const argsArray = node.arguments.filter((arg): arg is Expression => arg.type !== 'SpreadElement').map(a => g.genExpression(a));
    const args = argsArray.join(', ');

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
        if (collectionName === 'tabula') {
            const method = getTabulaMethod(methodName);
            if (method) {
                for (const header of getTabulaHeaders(methodName)) {
                    g.includes.add(header);
                }
                if (typeof method.cpp === 'function') {
                    return method.cpp(obj, argsArray);
                }
                return `${obj}.${method.cpp}(${args})`;
            }
        } else if (collectionName === 'copia') {
            const method = getCopiaMethod(methodName);
            if (method) {
                for (const header of getCopiaHeaders(methodName)) {
                    g.includes.add(header);
                }
                if (typeof method.cpp === 'function') {
                    return method.cpp(obj, argsArray);
                }
                return `${obj}.${method.cpp}(${args})`;
            }
        } else if (collectionName === 'lista') {
            const method = getListaMethod(methodName);
            if (method) {
                for (const header of getListaHeaders(methodName)) {
                    g.includes.add(header);
                }
                if (typeof method.cpp === 'function') {
                    return method.cpp(obj, argsArray);
                }
                return `${obj}.${method.cpp}(${args})`;
            }
        }

        // Fallback: no type info - try lista (most common)
        const listaMethod = getListaMethod(methodName);
        if (listaMethod) {
            for (const header of getListaHeaders(methodName)) {
                g.includes.add(header);
            }
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
