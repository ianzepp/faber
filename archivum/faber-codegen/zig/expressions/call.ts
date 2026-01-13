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
 *
 * Collection methods are translated via the unified norma registry.
 */

import type { CallExpression, Expression, SpreadElement, Identifier } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

// WHY: Unified norma registry for all stdlib translations (from .fab files)
import { getNormaTranslation, applyNormaTemplate, applyNormaModuleCall, validateMorphology } from '../../norma-registry';
import { applyNamespaceTemplate, getNamespaceTranslation, isNamespaceCall } from '../../shared/norma-namespace';

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

    const buildNamespaceArgs = (params?: string[]): string[] => {
        if (!params || params.length === 0) {
            return [...argsArray];
        }

        const built: string[] = [];
        let argIndex = 0;
        for (const param of params) {
            if (param === 'alloc') {
                built.push(g.getCurator());
            } else {
                built.push(argsArray[argIndex++] ?? '');
            }
        }
        return built;
    };

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

        // Check norma module functions (mathesis, tempus, aleator)
        for (const module of ['mathesis', 'tempus', 'aleator']) {
            const call = applyNormaModuleCall('zig', module, name, [...argsArray]);
            if (call) {
                return call;
            }
        }
    }

    if (isNamespaceCall(node)) {
        const translation = getNamespaceTranslation(node.callee, 'zig');
        if (translation) {
            if (translation.method) {
                return `${translation.method}(${args})`;
            }
            if (translation.template) {
                const templateArgs = buildNamespaceArgs(translation.params);
                return applyNamespaceTemplate(translation.template, templateArgs);
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

        // Helper to apply norma translation with allocator handling
        const applyZigNorma = (coll: string, method: string): string | null => {
            const norma = getNormaTranslation('zig', coll, method);
            if (!norma) return null;

            // Flag features for preamble
            if (coll === 'lista') g.features.lista = true;
            else if (coll === 'tabula') g.features.tabula = true;
            else if (coll === 'copia') g.features.copia = true;

            if (norma.template && norma.params) {
                // WHY: Zig templates may include 'alloc' param - check if curator needed
                const needsAlloc = norma.params.includes('alloc');
                const curator = needsAlloc ? g.getCurator() : '';
                const templateArgs = needsAlloc ? [...argsArray, curator] : [...argsArray];
                return applyNormaTemplate(norma.template, [...norma.params], obj, templateArgs);
            }
            if (norma.method) {
                return `${obj}.${norma.method}(${args})`;
            }
            return null;
        };

        // Try norma registry for the resolved collection type
        if (collectionName) {
            // WHY: Validate morphology before translation. Catches undefined forms.
            const validation = validateMorphology(collectionName, methodName);
            if (!validation.valid) {
                return `// MORPHOLOGY: ${validation.error}\n${obj}.${methodName}(${args})`;
            }

            const result = applyZigNorma(collectionName, methodName);
            if (result) return result;
        }

        // WHY: No fallback guessing. If type isn't resolved to a known collection,
        //      preserve the Latin method name. Require type annotations for translation.
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
