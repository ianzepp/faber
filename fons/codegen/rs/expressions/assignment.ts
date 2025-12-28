/**
 * Rust Code Generator - Assignment Expression
 *
 * TRANSFORMS:
 *   x = 5 -> x = 5
 *   x += 1 -> x += 1
 */

import type { AssignmentExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genAssignmentExpression(node: AssignmentExpression, g: RsGenerator): string {
    const left = node.left.type === 'Identifier' ? node.left.name : g.genBareExpression(node.left);
    return `${left} ${node.operator} ${g.genBareExpression(node.right)}`;
}
