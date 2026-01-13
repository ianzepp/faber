/**
 * C++23 Code Generator - RangeExpression
 *
 * TRANSFORMS:
 *   0..10       -> std::views::iota(0, 11)
 *   0 usque 10  -> std::views::iota(0, 11)  (inclusive)
 *
 * Note: Range expressions should be handled in for loops for efficiency.
 *       If used standalone, we need std::ranges::iota_view.
 */

import type { RangeExpression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genRangeExpression(node: RangeExpression, g: CppGenerator): string {
    // Range expressions should be handled in for loops
    // If used standalone, we'd need std::ranges::iota_view
    g.includes.add('<ranges>');

    const start = g.genExpression(node.start);
    const end = g.genExpression(node.end);

    return `std::views::iota(${start}, ${end} + 1)`;
}
