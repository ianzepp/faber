/**
 * C++23 Code Generator - NovumExpression
 *
 * TRANSFORMS:
 *   novum Foo()        -> Foo{}
 *   novum Foo { x: 1 } -> Foo{.x = 1}
 *   novum Foo(1, 2)    -> Foo(1, 2)
 */

import type { NovumExpression, Expression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genNovumExpression(node: NovumExpression, g: CppGenerator): string {
    const callee = node.callee.name;

    // Handle { ... } or de expr overrides - use aggregate initialization
    if (node.withExpression) {
        const overrides = g.genExpression(node.withExpression);

        return `${callee}${overrides}`;
    }

    // No arguments - stack allocation with default init
    if (node.arguments.length === 0) {
        return `${callee}{}`;
    }

    // With arguments - use parentheses
    const args = node.arguments
        .filter((arg): arg is Expression => arg.type !== 'SpreadElement')
        .map(arg => g.genExpression(arg))
        .join(', ');

    return `${callee}(${args})`;
}
