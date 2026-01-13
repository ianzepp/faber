/**
 * Zig Code Generator - Scriptum Expression (format string)
 *
 * TRANSFORMS:
 *   scriptum("Hello, §!", name) -> std.fmt.allocPrint(alloc, "Hello, {any}!", .{name}) catch @panic("OOM")
 *   scriptum("§1 before §0", a, b) -> std.fmt.allocPrint(alloc, "{any} before {any}", .{b, a}) (indexed)
 *
 * TARGET: Zig's std.fmt.allocPrint for runtime string formatting.
 *
 * WHY: Zig's ++ operator is comptime-only. Runtime string concatenation
 *      requires an allocator. scriptum provides a clean syntax for this.
 *
 * WHY: Format string is passed through with § converted to {any}.
 *      User can also use Zig-specific format specifiers ({s} for strings,
 *      {d} for integers) which are passed through as-is.
 *
 * NOTE: Supports both positional (§) and indexed (§0, §1) placeholders.
 *       Since Zig's std.fmt doesn't support positional indices, we reorder
 *       the arguments in the tuple to match the order in the format string.
 */

import type { ScriptumExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genScriptumExpression(node: ScriptumExpression, g: ZigGenerator): string {
    const originalFormat = node.format.value as string;
    const curator = g.getCurator();
    const genArgs = node.arguments.map(arg => g.genExpression(arg));

    if (node.arguments.length === 0) {
        // No args - just convert § to {any} and return as constant
        const format = originalFormat.replace(/§/g, '{any}');
        return `"${format}"`;
    }

    // Parse format string to find placeholder order
    // WHY: Zig std.fmt doesn't support positional indices, so we must
    //      reorder arguments to match the order placeholders appear
    const placeholderOrder: number[] = [];
    let implicitIdx = 0;
    let format = '';

    for (let i = 0; i < originalFormat.length; i++) {
        if (originalFormat[i] === '§') {
            // Check for explicit index (§0, §1, etc.)
            let explicitIdxStr = '';
            let nextChar = originalFormat[i + 1];
            while (i + 1 < originalFormat.length && nextChar !== undefined && nextChar >= '0' && nextChar <= '9') {
                explicitIdxStr += originalFormat[++i];
                nextChar = originalFormat[i + 1];
            }

            const idx = explicitIdxStr ? parseInt(explicitIdxStr, 10) : implicitIdx++;
            placeholderOrder.push(idx);
            format += '{any}';
        }
        else {
            format += originalFormat[i];
        }
    }

    // Reorder arguments according to placeholder order
    const orderedArgs = placeholderOrder.map(idx => genArgs[idx] ?? 'undefined').join(', ');

    return `std.fmt.allocPrint(${curator}, "${format}", .{ ${orderedArgs} }) catch @panic("OOM")`;
}
