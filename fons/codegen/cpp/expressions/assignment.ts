/**
 * C++23 Code Generator - AssignmentExpression
 *
 * TRANSFORMS:
 *   x = 5   -> x = 5
 *   x += 1  -> x += 1
 *   x -= 1  -> x -= 1
 */

import type { AssignmentExpression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genAssignmentExpression(node: AssignmentExpression, g: CppGenerator): string {
    const left = node.left.type === 'Identifier' ? node.left.name : g.genExpression(node.left);
    const right = g.genExpression(node.right);

    return `${left} ${node.operator} ${right}`;
}
