/**
 * Python Code Generator - Assignment Expression
 *
 * TRANSFORMS:
 *   x = 5       -> x = 5
 *   x += 1      -> x += 1
 *   obj.prop = v -> obj.prop = v
 */

import type { AssignmentExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genAssignmentExpression(node: AssignmentExpression, g: PyGenerator): string {
    const left = node.left.type === 'Identifier' ? node.left.name : g.genBareExpression(node.left);

    return `${left} ${node.operator} ${g.genBareExpression(node.right)}`;
}
