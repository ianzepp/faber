/**
 * Faber Code Generator - AssignmentExpression
 */

import type { AssignmentExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genAssignmentExpression(node: AssignmentExpression, g: FabGenerator): string {
    const left = g.genExpression(node.left);
    const right = g.genExpression(node.right);
    return `${left} ${node.operator} ${right}`;
}
