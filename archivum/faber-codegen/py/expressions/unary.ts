/**
 * Python Code Generator - Unary Expression
 *
 * TRANSFORMS:
 *   nulla x     -> (not x or len(x) == 0 if hasattr(x, '__len__') else not x)
 *   nonnulla x  -> (x and (len(x) > 0 if hasattr(x, '__len__') else bool(x)))
 *   nihil x     -> (x is None)
 *   nonnihil x  -> (x is not None)
 *   negativum x -> (x < 0)
 *   positivum x -> (x > 0)
 *   !x          -> not x
 *   -x          -> -x
 */

import type { UnaryExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genUnaryExpression(node: UnaryExpression, g: PyGenerator): string {
    const arg = g.genExpression(node.argument);

    // nulla: check if empty
    if (node.operator === 'nulla') {
        return `(not ${arg} or len(${arg}) == 0 if hasattr(${arg}, '__len__') else not ${arg})`;
    }

    // nonnulla: check if non-empty
    if (node.operator === 'nonnulla') {
        return `(${arg} and (len(${arg}) > 0 if hasattr(${arg}, '__len__') else bool(${arg})))`;
    }

    // nihil: check if None
    if (node.operator === 'nihil') {
        return `(${arg} is None)`;
    }

    // nonnihil: check if not None
    if (node.operator === 'nonnihil') {
        return `(${arg} is not None)`;
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
        return `(${arg} is True)`;
    }

    // falsum: strict boolean false check
    if (node.operator === 'falsum') {
        return `(${arg} is False)`;
    }

    // Map ! to not
    if (node.operator === '!') {
        return `not ${arg}`;
    }

    return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
}
