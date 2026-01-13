/**
 * C++23 Code Generator - UnaryExpression
 *
 * TRANSFORMS:
 *   !x        -> !x
 *   -x        -> -x
 *   nulla x   -> (x.empty())
 *   nonnulla x -> (!x.empty())
 *   negativum x -> (x < 0)
 *   positivum x -> (x > 0)
 */

import type { UnaryExpression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genUnaryExpression(node: UnaryExpression, g: CppGenerator): string {
    const arg = g.genExpression(node.argument);

    // Handle Latin-specific operators
    switch (node.operator) {
        case 'nulla':
            return `(${arg}.empty())`;
        case 'nonnulla':
            return `(!${arg}.empty())`;
        case 'negativum':
            return `(${arg} < 0)`;
        case 'positivum':
            return `(${arg} > 0)`;
        case 'verum':
            return `(${arg} == true)`;
        case 'falsum':
            return `(${arg} == false)`;
    }

    return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
}
