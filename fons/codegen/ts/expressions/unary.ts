/**
 * TypeScript Code Generator - Unary Expression
 *
 * TRANSFORMS:
 *   !x -> !x (prefix)
 *   x++ -> x++ (postfix)
 *   nulla x -> inline empty check
 *   nonnulla x -> inline non-empty check
 *   nihil x -> (x === null)
 *   nonnihil x -> (x !== null)
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

    // nihil: check if null
    if (node.operator === 'nihil') {
        return `(${arg} === null)`;
    }

    // nonnihil: check if not null
    if (node.operator === 'nonnihil') {
        return `(${arg} !== null)`;
    }

    // negativum: check if less than zero
    if (node.operator === 'negativum') {
        return `(${arg} < 0)`;
    }

    // positivum: check if greater than zero
    if (node.operator === 'positivum') {
        return `(${arg} > 0)`;
    }

    return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
}
