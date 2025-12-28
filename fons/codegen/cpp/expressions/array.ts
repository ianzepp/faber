/**
 * C++23 Code Generator - ArrayExpression
 *
 * TRANSFORMS:
 *   [1, 2, 3] -> std::vector{1, 2, 3}
 *   []        -> std::vector<int>{}
 */

import type { ArrayExpression, Expression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

// WHY: Spread can be nested inside arrays (e.g., [[...inner], outer]).
// We need to recursively check all nested arrays for spread elements.
function containsSpread(node: ArrayExpression): boolean {
    for (const el of node.elements) {
        if (el.type === 'SpreadElement') return true;
        if (el.type === 'ArrayExpression' && containsSpread(el)) return true;
    }
    return false;
}

export function genArrayExpression(node: ArrayExpression, g: CppGenerator): string {
    g.includes.add('<vector>');

    if (node.elements.length === 0) {
        return 'std::vector<int>{}';
    }

    // WHY: Spread elements require runtime iteration in C++. When present
    // (even nested), we generate an empty vector as the base. The caller
    // is responsible for inserting spread elements via insert() calls.
    if (containsSpread(node)) {
        return 'std::vector{}';
    }

    const elements = node.elements.map(el => g.genExpression(el as Expression)).join(', ');

    return `std::vector{${elements}}`;
}
