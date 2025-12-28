/**
 * C++23 Code Generator - ArrayExpression
 *
 * TRANSFORMS:
 *   [1, 2, 3] -> std::vector{1, 2, 3}
 *   []        -> std::vector<int>{}
 */

import type { ArrayExpression, Expression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genArrayExpression(node: ArrayExpression, g: CppGenerator): string {
    g.includes.add('<vector>');

    if (node.elements.length === 0) {
        return 'std::vector<int>{}';
    }

    const elements = node.elements
        .filter((el): el is Expression => el.type !== 'SpreadElement')
        .map(el => g.genExpression(el))
        .join(', ');

    return `std::vector{${elements}}`;
}
