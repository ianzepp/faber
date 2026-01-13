/**
 * Rust Code Generator - Est Expression
 *
 * TRANSFORMS:
 *   x est Persona -> matches!(x, Persona)
 *   x non est Persona -> !matches!(x, Persona)
 */

import type { EstExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genEstExpression(node: EstExpression, g: RsGenerator): string {
    const expr = g.genExpression(node.expression);
    const typeName = node.targetType.name;

    // Rust type checks typically use pattern matching or trait methods
    if (node.negated) {
        return `!matches!(${expr}, ${typeName})`;
    }
    return `matches!(${expr}, ${typeName})`;
}
