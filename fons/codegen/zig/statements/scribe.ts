/**
 * Zig Code Generator - ScribeStatement (print/debug/warn)
 *
 * TRANSFORMS:
 *   scribe "hello" -> std.debug.print("hello\n", .{});
 *   vide x         -> std.debug.print("[DEBUG] {any}\n", .{x});
 *   mone "oops"    -> std.debug.print("[WARN] oops\n", .{});
 *
 * TARGET: Zig's std.debug.print with level prefixes for debug/warn.
 */

import type { ScribeStatement, Expression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genScribeStatement(node: ScribeStatement, g: ZigGenerator): string {
    const prefix = node.level === 'debug' ? '[DEBUG] ' : node.level === 'warn' ? '[WARN] ' : '';

    if (node.arguments.length === 0) {
        return `${g.ind()}std.debug.print("${prefix}\\n", .{});`;
    }

    // Build format string and args list
    const formatParts: string[] = [];
    const args: string[] = [];

    for (const arg of node.arguments) {
        formatParts.push(getFormatSpecifier(arg));
        args.push(g.genExpression(arg));
    }

    const format = prefix + formatParts.join(' ') + '\\n';

    return `${g.ind()}std.debug.print("${format}", .{ ${args.join(', ')} });`;
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
