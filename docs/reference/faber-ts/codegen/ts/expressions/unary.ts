/**
 * TypeScript Code Generator - Unary Expression
 *
 * TRANSFORMS:
 *   !x -> !x (prefix)
 *   x++ -> x++ (postfix)
 *   nulla x -> inline empty check
 *   nonnulla x -> inline non-empty check
 *   nihil x -> (x == null)    // catches null and undefined
 *   nonnihil x -> (x != null)  // catches null and undefined
 */

import type { UnaryExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genUnaryExpression(node: UnaryExpression, g: TsGenerator): string {
    const arg = g.genExpression(node.argument);

    // nulla: check if null/empty
    if (node.operator === 'nulla') {
        return `(${arg} == null || (Array.isArray(${arg}) || typeof ${arg} === 'string' ? ${arg}.length === 0 : typeof ${arg} === 'object' ? Object.keys(${arg}).length === 0 : !${arg}))`;
    }

    // nonnulla: check if non-null and has content
    if (node.operator === 'nonnulla') {
        return `(${arg} != null && (Array.isArray(${arg}) || typeof ${arg} === 'string' ? ${arg}.length > 0 : typeof ${arg} === 'object' ? Object.keys(${arg}).length > 0 : Boolean(${arg})))`;
    }

    // nihil: check if null (loose equality catches both null and undefined)
    if (node.operator === 'nihil') {
        return `(${arg} == null)`;
    }

    // nonnihil: check if not null (loose inequality catches both null and undefined)
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
        return `(${arg} === true)`;
    }

    // falsum: strict boolean false check
    if (node.operator === 'falsum') {
        return `(${arg} === false)`;
    }

    return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
}
