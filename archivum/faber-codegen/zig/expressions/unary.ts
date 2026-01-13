/**
 * Zig Code Generator - Unary Expression
 *
 * TRANSFORMS:
 *   !x -> !x (prefix)
 *   x++ -> x++ (postfix)
 *   nulla x -> x.len == 0
 *   nonnulla x -> x.len > 0
 *
 * TARGET: Zig is statically typed, so we emit .len checks for slices.
 *         For optionals, would need != null pattern.
 */

import type { UnaryExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genUnaryExpression(node: UnaryExpression, g: ZigGenerator): string {
    const arg = g.genExpression(node.argument);

    // nulla: check if empty (for slices/arrays)
    if (node.operator === 'nulla') {
        return `(${arg}.len == 0)`;
    }

    // nonnulla: check if non-empty (for slices/arrays)
    if (node.operator === 'nonnulla') {
        return `(${arg}.len > 0)`;
    }

    // nihil: check if null (for optionals)
    if (node.operator === 'nihil') {
        return `(${arg} == null)`;
    }

    // nonnihil: check if not null (for optionals)
    if (node.operator === 'nonnihil') {
        return `(${arg} != null)`;
    }

    // negativum: check if less than zero
    if (node.operator === 'negativum') {
        return `(${arg} < 0)`;
    }

    // positivum: check if greater than zero
    if (node.operator === 'positivum') {
        return `(${arg} > 0)`;
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
