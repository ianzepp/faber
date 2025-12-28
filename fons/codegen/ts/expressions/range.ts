/**
 * TypeScript Code Generator - Range Expression
 *
 * TRANSFORMS:
 *   0..5 -> Array.from({length: 5}, (_, i) => i)           // exclusive
 *   0 usque 5 -> Array.from({length: 6}, (_, i) => i)      // inclusive
 *   0..10 per 2 -> Array.from({length: 5}, (_, i) => i * 2)
 *
 * WHY: When used outside a for-loop, ranges become arrays.
 *      End is exclusive by default, inclusive with 'usque'.
 *      In for-loops, they compile to efficient traditional loops instead.
 */

import type { RangeExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genRangeExpression(node: RangeExpression, g: TsGenerator): string {
    const start = g.genExpression(node.start);
    const end = g.genExpression(node.end);
    const adj = node.inclusive ? ' + 1' : '';

    if (node.step) {
        const step = g.genExpression(node.step);

        // With step: more complex calculation
        // WHY: inclusive adds 1 to length calculation
        if (node.inclusive) {
            return `Array.from({length: Math.floor((${end} - ${start}) / ${step}) + 1}, (_, i) => ${start} + i * ${step})`;
        }

        return `Array.from({length: Math.ceil((${end} - ${start}) / ${step})}, (_, i) => ${start} + i * ${step})`;
    }

    // Simple range: adjust length for inclusive
    return `Array.from({length: ${end} - ${start}${adj}}, (_, i) => ${start} + i)`;
}
