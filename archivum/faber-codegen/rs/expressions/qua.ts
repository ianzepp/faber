/**
 * Rust Code Generator - Qua Expression (Type Cast)
 *
 * TRANSFORMS:
 *   x qua numerus -> x as f64
 */

import type { QuaExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genQuaExpression(node: QuaExpression, g: RsGenerator): string {
    const expr = g.genExpression(node.expression);
    const targetType = g.genType(node.targetType);
    return `${expr} as ${targetType}`;
}
