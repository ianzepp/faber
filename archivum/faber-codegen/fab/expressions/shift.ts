/**
 * Faber Code Generator - Shift Expression (bit shifting)
 *
 * TRANSFORMS:
 *   x dextratum 3 -> x dextratum 3
 *   x sinistratum 3 -> x sinistratum 3
 *
 * NOTE: Faber-to-Faber roundtripping preserves the Latin keywords.
 */

import type { ShiftExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genShiftExpression(node: ShiftExpression, g: FabGenerator): string {
    const expr = g.genExpression(node.expression);
    const amount = g.genExpression(node.amount);

    return `${expr} ${node.direction} ${amount}`;
}
