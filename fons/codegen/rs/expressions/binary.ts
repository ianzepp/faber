/**
 * Rust Code Generator - Binary Expression
 *
 * TRANSFORMS:
 *   a + b -> (a + b)
 *   a && b -> (a && b)
 *   a == b -> (a == b)
 */

import type { BinaryExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genBinaryExpression(node: BinaryExpression, g: RsGenerator): string {
    const left = g.genExpression(node.left);
    const right = g.genExpression(node.right);

    return `(${left} ${node.operator} ${right})`;
}
