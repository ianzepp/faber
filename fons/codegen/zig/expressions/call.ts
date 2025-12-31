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

// WHY: Unified registries for collection methods (stdlib refactor)
import { getListaMethod } from '../../lista';
import { getTabulaMethod } from '../../tabula';
import { getCopiaMethod } from '../../copia';

// Collection method registries (will be unified in future phases)
import { getMathesisFunction } from '../norma/mathesis';
import { getTempusFunction } from '../norma/tempus';
import { getAleatorFunction } from '../norma/aleator';

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

    // TARGET: I/O intrinsics map to Zig's std.debug.print() variants
    if (node.callee.type === 'Identifier') {
        const name = node.callee.name;

        if (name === '_scribe' || name === '_vide' || name === '_mone') {
            const formatSpecs = node.arguments.map(arg => {
                if (arg.type === 'SpreadElement') {
                    return '{any}';
                }
                return getFormatSpecifier(arg);
            });

            // WHY: Different prefixes for different log levels
            // _scribe = standard output, _vide = debug, _mone = warning
            let prefix = '';
            if (name === '_vide') {
                prefix = '[DEBUG] ';
            } else if (name === '_mone') {
                prefix = '[WARN] ';
            }

            const format = prefix + formatSpecs.join(' ') + '\\n';
            return `std.debug.print("${format}", .{${args}})`;
        }

        // WHY: _lege reads a line from stdin. In Zig, this requires std.io.getStdIn()
        // and reading until newline. Returns empty string on EOF/error.
        if (name === '_lege') {
            // LIMITATION: Zig stdin reading is complex. This is a simplified version
            // that reads one line. For robust input, user should use std.io directly.
            return `(std.io.getStdIn().reader().readUntilDelimiterOrEof(buf, '\\n') catch "") orelse ""`;
        }

        // Check mathesis functions (ex "norma/mathesis" importa pavimentum, etc.)
        const mathesisFunc = getMathesisFunction(name);
        if (mathesisFunc) {
            if (typeof mathesisFunc.zig === 'function') {
                return mathesisFunc.zig(argsArray);
            }
            return mathesisFunc.zig;
        }

        // Check tempus functions (ex "norma/tempus" importa nunc, dormi, etc.)
        const tempusFunc = getTempusFunction(name);
        if (tempusFunc) {
            if (typeof tempusFunc.zig === 'function') {
                return tempusFunc.zig(argsArray);
            }
            return tempusFunc.zig;
        }

        // Check aleator functions (ex "norma/aleator" importa fractus, inter, etc.)
        // WHY: Aleator functions have fallback allocators, so don't require cura block
        const aleatorFunc = getAleatorFunction(name);
        if (aleatorFunc) {
            const curator = g.getCuratorOrUndefined();
            if (typeof aleatorFunc.zig === 'function') {
                return aleatorFunc.zig(argsArray, curator);
            }
            return aleatorFunc.zig;
        }
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
        // WHY: Only fetch curator when we have a known collection method that needs it
        if (collectionName === 'tabula') {
            const method = getTabulaMethod(methodName);
            if (method) {
                // WHY: Flag features so preamble includes Tabula and arena setup
                g.features.tabula = true;
                const curator = method.needsAlloc ? g.getCurator() : '';
                if (typeof method.zig === 'function') {
                    return method.zig(obj, argsArray, curator);
                }
                return `${obj}.${method.zig}(${args})`;
            }
        } else if (collectionName === 'copia') {
            const method = getCopiaMethod(methodName);
            if (method) {
                // WHY: Flag features so preamble includes Copia and arena setup
                g.features.copia = true;
                const curator = method.needsAlloc ? g.getCurator() : '';
                if (typeof method.zig === 'function') {
                    return method.zig(obj, argsArray, curator);
                }
                return `${obj}.${method.zig}(${args})`;
            }
        } else if (collectionName === 'lista') {
            const method = getListaMethod(methodName);
            if (method) {
                // WHY: Flag features so preamble includes Lista and arena setup
                g.features.lista = true;
                const curator = method.needsAlloc ? g.getCurator() : '';
                if (typeof method.zig === 'function') {
                    return method.zig(obj, argsArray, curator);
                }
                return `${obj}.${method.zig}(${args})`;
            }
        }

        // Fallback: no type info or unknown type - try lista (most common)
        const listaMethod = getListaMethod(methodName);
        if (listaMethod) {
            // WHY: Flag features so preamble includes Lista and arena setup in main()
            // Without this, code like `items.adde(alloc, 1)` would reference undefined alloc
            g.features.lista = true;
            // WHY: Only fetch curator if method actually needs allocator
            const curator = listaMethod.needsAlloc ? g.getCurator() : '';
            if (typeof listaMethod.zig === 'function') {
                return listaMethod.zig(obj, argsArray, curator);
            }
            return `${obj}.${listaMethod.zig}(${args})`;
        }
    }

    const callee = g.genExpression(node.callee);

    // WHY: Inject curator (allocator) for functions marked by semantic analyzer
    // Functions with 'curator' param type get allocator auto-injected at call sites
    let finalArgs = args;
    if (node.needsCurator) {
        const curator = g.getCurator();
        finalArgs = args ? `${args}, ${curator}` : curator;
    }

    // WHY: Optional call in Zig requires if-else pattern
    if (node.optional) {
        return `(if (${callee}) |_fn| _fn(${finalArgs}) else null)`;
    }

    // WHY: Non-null assertion unwraps optional function before calling
    if (node.nonNull) {
        return `${callee}.?(${finalArgs})`;
    }

    return `${callee}(${finalArgs})`;
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
