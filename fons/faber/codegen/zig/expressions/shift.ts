/**
 * Zig Code Generator - Shift Expression (bit shifting)
 *
 * TRANSFORMS:
 *   x dextratum 3 -> (x >> 3)
 *   x sinistratum 3 -> (x << 3)
 *
 * NOTE: Zig uses the same shift operators as C-family languages.
 */

import type { ShiftExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genShiftExpression(node: ShiftExpression, g: ZigGenerator): string {
    const expr = g.genExpression(node.expression);
    const amount = g.genExpression(node.amount);
    const op = node.direction === 'dextratum' ? '>>' : '<<';

    return `(${expr} ${op} ${amount})`;
}
