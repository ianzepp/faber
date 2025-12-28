/**
 * TypeScript Code Generator - Assignment Expression
 *
 * TRANSFORMS:
 *   x = 5 -> x = 5
 *   x += 1 -> x += 1
 *   obj.prop = value -> obj.prop = value
 */

import type { AssignmentExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genAssignmentExpression(node: AssignmentExpression, g: TsGenerator): string {
    const left = node.left.type === 'Identifier' ? node.left.name : g.genBareExpression(node.left);

    return `${left} ${node.operator} ${g.genBareExpression(node.right)}`;
}
