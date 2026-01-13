/**
 * Faber Code Generator - RangeExpression
 */

import type { RangeExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genRangeExpression(node: RangeExpression, g: FabGenerator): string {
    const start = g.genExpression(node.start);
    const end = g.genExpression(node.end);
    const op = node.inclusive ? 'usque' : '..';

    let result = `${start} ${op} ${end}`;

    if (node.step) {
        result += ` per ${g.genExpression(node.step)}`;
    }

    return result;
}
