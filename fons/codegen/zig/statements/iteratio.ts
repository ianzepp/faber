/**
 * Zig Code Generator - IteratioStatement (for loop)
 *
 * TRANSFORMS:
 *   ex 0..10 pro i { } -> var i: usize = 0; while (i < 10) : (i += 1) { }
 *   ex items pro item { } -> for (items) |item| { }
 *
 * TARGET: Zig uses for (slice) |item| for iteration over slices.
 *         For ranges, we use while loops since Zig doesn't have range syntax.
 */

import type { IteratioStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genIteratioStatement(node: IteratioStatement, g: ZigGenerator): string {
    const varName = node.variable.name;
    const body = g.genBlockStatement(node.body);

    // Handle range expressions with while loop
    if (node.iterable.type === 'RangeExpression') {
        const range = node.iterable;
        const start = g.genExpression(range.start);
        const end = g.genExpression(range.end);
        const cmp = range.inclusive ? '<=' : '<';

        if (range.step) {
            const step = g.genExpression(range.step);

            return `${g.ind()}var ${varName}: usize = ${start}; while (${varName} ${cmp} ${end}) : (${varName} += ${step}) ${body}`;
        }

        return `${g.ind()}var ${varName}: usize = ${start}; while (${varName} ${cmp} ${end}) : (${varName} += 1) ${body}`;
    }

    const iterable = g.genExpression(node.iterable);

    // Zig uses for (slice) |item| syntax
    return `${g.ind()}for (${iterable}) |${varName}| ${body}`;
}
