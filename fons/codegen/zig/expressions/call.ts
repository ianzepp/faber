/**
 * Zig Code Generator - Call Expression
 *
 * TRANSFORMS:
 *   scribe("hello") -> std.debug.print("{s}\n", .{"hello"})
 *   scribe(42) -> std.debug.print("{d}\n", .{42})
 *   foo(x, y) -> foo(x, y)
 *   foo(sparge args) -> (spread not directly supported)
 *
 * TARGET: scribe() is Latin's print function, maps to std.debug.print().
 *         Zig print uses format strings and anonymous tuple syntax (.{...}).
 *
 * LIMITATION: Zig doesn't support spread in function calls. Would require
 *             comptime tuple unpacking or @call with .args tuple.
 */

import type { CallExpression, Expression, SpreadElement, Identifier } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

// Collection method registries
import { getListaMethod } from '../norma/lista';
import { getTabulaMethod } from '../norma/tabula';
import { getCopiaMethod } from '../norma/copia';

export function genCallExpression(node: CallExpression, g: ZigGenerator): string {
    // Helper to generate argument, handling spread
    const genArg = (arg: Expression | SpreadElement): string => {
        if (arg.type === 'SpreadElement') {
            // WHY: Zig doesn't have spread in calls. This is a limitation.
            // Could potentially use @call(.auto, fn, args_tuple) but complex.
            return `@compileError("Call spread not supported in Zig target")`;
        }
        return g.genExpression(arg);
    };

    // WHY: Build args as array first, then join for regular calls.
    // Collection method handlers receive the array to preserve argument
    // boundaries (avoiding comma-in-lambda parsing issues).
    const argsArray = node.arguments.map(genArg);
    const args = argsArray.join(', ');

    // TARGET: _scribe intrinsic maps to Zig's std.debug.print()
    if (node.callee.type === 'Identifier' && node.callee.name === '_scribe') {
        const formatSpecs = node.arguments.map(arg => {
            if (arg.type === 'SpreadElement') {
                return '{any}';
            }
            return getFormatSpecifier(arg);
        });
        const format = formatSpecs.join(' ') + '\\n';

        return `std.debug.print("${format}", .{${args}})`;
    }

    // Check for collection methods (method calls on lista/tabula/copia)
    // WHY: Latin collection methods map to Zig stdlib ArrayList/HashMap operations
    // WHY: Pass argsArray (not joined string) to method handlers
    //      so they can correctly handle multi-param lambdas with commas.
    if (node.callee.type === 'MemberExpression' && !node.callee.computed) {
        const methodName = (node.callee.property as Identifier).name;
        const obj = g.genExpression(node.callee.object);

        // Use semantic type info to dispatch to correct collection registry
        const objType = node.callee.object.resolvedType;
        const collectionName = objType?.kind === 'generic' ? objType.name : null;

        // Dispatch based on resolved type
        // WHY: Pass current curator to method generators that need allocator
        const curator = g.getCurator();

        if (collectionName === 'tabula') {
            const method = getTabulaMethod(methodName);
            if (method) {
                if (typeof method.zig === 'function') {
                    return method.zig(obj, argsArray, curator);
                }
                return `${obj}.${method.zig}(${args})`;
            }
        } else if (collectionName === 'copia') {
            const method = getCopiaMethod(methodName);
            if (method) {
                if (typeof method.zig === 'function') {
                    return method.zig(obj, argsArray, curator);
                }
                return `${obj}.${method.zig}(${args})`;
            }
        } else if (collectionName === 'lista') {
            const method = getListaMethod(methodName);
            if (method) {
                if (typeof method.zig === 'function') {
                    return method.zig(obj, argsArray, curator);
                }
                return `${obj}.${method.zig}(${args})`;
            }
        }

        // Fallback: no type info or unknown type - try lista (most common)
        const listaMethod = getListaMethod(methodName);
        if (listaMethod) {
            if (typeof listaMethod.zig === 'function') {
                return listaMethod.zig(obj, argsArray, curator);
            }
            return `${obj}.${listaMethod.zig}(${args})`;
        }
    }

    const callee = g.genExpression(node.callee);

    // WHY: Optional call in Zig requires if-else pattern
    if (node.optional) {
        return `(if (${callee}) |_fn| _fn(${args}) else null)`;
    }

    // WHY: Non-null assertion unwraps optional function before calling
    if (node.nonNull) {
        return `${callee}.?(${args})`;
    }

    return `${callee}(${args})`;
}

/**
 * Get Zig format specifier for an expression based on its resolved type.
 *
 * TARGET: Zig uses {s} for strings, {d} for integers, {any} for unknown.
 */
function getFormatSpecifier(expr: Expression): string {
    // Use resolved type if available
    if (expr.resolvedType?.kind === 'primitive') {
        switch (expr.resolvedType.name) {
            case 'textus':
                return '{s}';
            case 'numerus':
                return '{d}';
            case 'bivalens':
                return '{}';
            default:
                return '{any}';
        }
    }

    // Fallback: infer from literal/identifier
    if (expr.type === 'Literal') {
        if (typeof expr.value === 'string') {
            return '{s}';
        }

        if (typeof expr.value === 'number') {
            return '{d}';
        }

        if (typeof expr.value === 'boolean') {
            return '{}';
        }
    }

    if (expr.type === 'Identifier') {
        if (expr.name === 'verum' || expr.name === 'falsum') {
            return '{}';
        }
    }

    if (expr.type === 'TemplateLiteral') {
        return '{s}';
    }

    return '{any}';
}
