/**
 * Zig Code Generator - ScribeStatement (print/debug/warn)
 *
 * TRANSFORMS:
 *   scribe "hello" -> stdout.print("{s}\n", .{"hello"});
 *   vide x         -> std.debug.print("[DEBUG] {any}\n", .{x});
 *   mone "oops"    -> stderr.print("[WARN] {s}\n", .{"oops"});
 *
 * TARGET:
 *   scribe -> stdout (normal output)
 *   mone   -> stderr (warnings/errors)
 *   vide   -> std.debug.print (debug, stripped in release)
 */

import type { ScribeStatement, Expression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genScribeStatement(node: ScribeStatement, g: ZigGenerator): string {
    // Build format string and args list
    const formatParts: string[] = [];
    const args: string[] = [];

    for (const arg of node.arguments) {
        formatParts.push(getFormatSpecifier(arg));
        args.push(g.genExpression(arg));
    }

    const argsStr = args.length > 0 ? ` ${args.join(', ')} ` : '';

    // vide -> std.debug.print (debug output, stripped in release builds)
    if (node.level === 'debug') {
        const format = '[DEBUG] ' + (formatParts.length > 0 ? formatParts.join(' ') : '') + '\\n';
        return `${g.ind()}std.debug.print("${format}", .{${argsStr}});`;
    }

    // mone -> stderr (warnings/errors)
    if (node.level === 'warn') {
        g.features.stderr = true;
        const format = '[WARN] ' + (formatParts.length > 0 ? formatParts.join(' ') : '') + '\\n';
        return `${g.ind()}eprint("${format}", .{${argsStr}});`;
    }

    // scribe -> stdout (normal output)
    g.features.stdout = true;
    const format = (formatParts.length > 0 ? formatParts.join(' ') : '') + '\\n';
    return `${g.ind()}print("${format}", .{${argsStr}});`;
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
