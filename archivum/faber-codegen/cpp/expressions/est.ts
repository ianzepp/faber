/**
 * C++23 Code Generator - EstExpression
 *
 * TRANSFORMS:
 *   x est Foo        -> std::is_same_v<decltype(x), Foo>
 *   x non est Foo    -> !std::is_same_v<decltype(x), Foo>
 *
 * WHY: C++ doesn't have runtime type checking like Python's isinstance().
 * For compile-time type checking, std::is_same_v works for exact matches.
 * For polymorphic types, dynamic_cast would be needed but requires RTTI.
 */

import type { EstExpression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genEstExpression(node: EstExpression, g: CppGenerator): string {
    g.includes.add('<type_traits>');

    const expr = g.genExpression(node.expression);
    const targetType = g.genType(node.targetType);

    const check = `std::is_same_v<std::decay_t<decltype(${expr})>, ${targetType}>`;

    // WHY: std::decay_t removes references/cv-qualifiers for accurate comparison
    return node.negated ? `!${check}` : check;
}
