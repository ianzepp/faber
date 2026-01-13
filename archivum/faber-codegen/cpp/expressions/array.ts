/**
 * C++23 Code Generator - ArrayExpression
 *
 * TRANSFORMS:
 *   [1, 2, 3] -> std::vector{1, 2, 3}
 *   []        -> {}
 *
 * WHY: Empty arrays use {} (brace initialization) to let C++ infer type from context.
 *      This works for default parameters, assignments, and return statements.
 */

import type { ArrayExpression, Expression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

// WHY: Spread can be nested inside arrays (e.g., [[...inner], outer]).
// We need to recursively check all nested arrays for spread elements.
function containsSpread(node: ArrayExpression): boolean {
    for (const el of node.elements) {
        if (el.type === 'SpreadElement') {
            return true;
        }
        if (el.type === 'ArrayExpression' && containsSpread(el)) {
            return true;
        }
    }
    return false;
}

export function genArrayExpression(node: ArrayExpression, g: CppGenerator): string {
    g.includes.add('<vector>');

    // WHY: Empty array uses {} for type inference from context
    if (node.elements.length === 0) {
        return '{}';
    }

    // TODO: Spread elements require runtime iteration in C++.
    // Currently broken - just outputs {} and drops the spread.
    // Proper fix: generate vector with insert() calls for spread elements.
    if (containsSpread(node)) {
        return '{}';
    }

    const elements = node.elements.map(el => g.genExpression(el as Expression)).join(', ');

    return `std::vector{${elements}}`;
}
