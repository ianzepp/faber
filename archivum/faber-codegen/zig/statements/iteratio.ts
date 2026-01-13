/**
 * Zig Code Generator - IteratioStatement (for loop)
 *
 * TRANSFORMS:
 *   ex 0..10 pro i { }      -> for (0..10) |i| { }
 *   ex 0 usque 10 pro i { } -> for (0..11) |i| { }  (inclusive adds 1)
 *   ex 0..10 per 2 pro i {} -> while loop (Zig for doesn't support step)
 *   ex items pro item { }   -> for (items) |item| { }
 *
 * TARGET: Zig 0.11+ supports for (start..end) |i| syntax for ranges.
 *         Stepped ranges still require while loops.
 */

import type { IteratioStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genIteratioStatement(node: IteratioStatement, g: ZigGenerator): string {
    const varName = node.variable.name;
    const body = g.genBlockStatement(node.body);

    // Handle range expressions
    if (node.iterable.type === 'RangeExpression') {
        const range = node.iterable;
        const start = g.genExpression(range.start);
        const end = g.genExpression(range.end);

        // Stepped ranges need while loops (Zig for doesn't support step)
        // WHY: Wrap in block to scope the loop variable, avoiding redeclaration
        if (range.step) {
            const step = g.genExpression(range.step);
            const cmp = range.inclusive ? '<=' : '<';

            const lines: string[] = [];
            lines.push(`${g.ind()}{`);
            g.depth++;
            lines.push(`${g.ind()}var ${varName}: usize = ${start};`);
            lines.push(`${g.ind()}while (${varName} ${cmp} ${end}) : (${varName} += ${step}) ${body}`);
            g.depth--;
            lines.push(`${g.ind()}}`);
            return lines.join('\n');
        }

        // Use native for (start..end) syntax (Zig 0.11+)
        // Inclusive ranges need end + 1 since Zig ranges are exclusive
        const endExpr = range.inclusive ? `${end} + 1` : end;
        return `${g.ind()}for (${start}..${endExpr}) |${varName}| ${body}`;
    }

    const iterable = g.genExpression(node.iterable);

    // Zig uses for (slice) |item| syntax
    return `${g.ind()}for (${iterable}) |${varName}| ${body}`;
}
