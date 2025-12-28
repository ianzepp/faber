/**
 * TypeScript Code Generator - Binary Expression
 *
 * TRANSFORMS:
 *   x + y -> (x + y)
 *   a && b -> (a && b)
 *
 * WHY: Parentheses ensure correct precedence in all contexts.
 */

import type { BinaryExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genBinaryExpression(node: BinaryExpression, g: TsGenerator): string {
    const left = g.genExpression(node.left);
    const right = g.genExpression(node.right);

    return `(${left} ${node.operator} ${right})`;
}
