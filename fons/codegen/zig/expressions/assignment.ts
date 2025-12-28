/**
 * Zig Code Generator - Assignment Expression
 *
 * TRANSFORMS:
 *   x = 5 -> x = 5
 *   x += 1 -> x += 1
 *   x /= 2 -> x = @divTrunc(x, 2)  (Zig requires explicit division)
 *   x %= 2 -> x = @mod(x, 2)
 */

import type { AssignmentExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genAssignmentExpression(node: AssignmentExpression, g: ZigGenerator): string {
    const left = node.left.type === 'Identifier' ? node.left.name : g.genBareExpression(node.left);
    const right = g.genBareExpression(node.right);

    // WHY: Zig requires @divTrunc for signed integer division
    if (node.operator === '/=') {
        return `${left} = @divTrunc(${left}, ${right})`;
    }

    // WHY: Zig requires @mod for signed integer modulo
    if (node.operator === '%=') {
        return `${left} = @mod(${left}, ${right})`;
    }

    return `${left} ${node.operator} ${right}`;
}
