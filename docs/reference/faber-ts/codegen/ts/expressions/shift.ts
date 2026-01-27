/**
 * TypeScript Code Generator - Shift Expression (bit shifting)
 *
 * TRANSFORMS:
 *   x dextratum 3 -> (x >> 3)
 *   x sinistratum 3 -> (x << 3)
 */

import type { ShiftExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genShiftExpression(node: ShiftExpression, g: TsGenerator): string {
    const expr = g.genExpression(node.expression);
    const amount = g.genExpression(node.amount);
    const op = node.direction === 'dextratum' ? '>>' : '<<';

    return `(${expr} ${op} ${amount})`;
}
