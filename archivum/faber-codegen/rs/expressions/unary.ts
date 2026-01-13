/**
 * Rust Code Generator - Unary Expression
 *
 * TRANSFORMS:
 *   !x -> !x
 *   -x -> -x
 *   nulla x -> x.is_none()
 *   nonnulla x -> x.is_some()
 *   nihil x -> x.is_none()
 *   nonnihil x -> x.is_some()
 */

import type { UnaryExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genUnaryExpression(node: UnaryExpression, g: RsGenerator): string {
    const arg = g.genExpression(node.argument);

    // Rust-specific mappings
    if (node.operator === 'nulla') {
        return `${arg}.is_none()`;
    }
    if (node.operator === 'nonnulla') {
        return `${arg}.is_some()`;
    }
    if (node.operator === 'nihil') {
        return `${arg}.is_none()`;
    }
    if (node.operator === 'nonnihil') {
        return `${arg}.is_some()`;
    }

    // verum: strict boolean true check
    if (node.operator === 'verum') {
        return `(${arg} == true)`;
    }

    // falsum: strict boolean false check
    if (node.operator === 'falsum') {
        return `(${arg} == false)`;
    }

    return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
}
