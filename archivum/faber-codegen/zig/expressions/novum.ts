/**
 * Zig Code Generator - Novum Expression (new/constructor)
 *
 * TRANSFORMS:
 *   novum Foo -> Foo.init(.{})
 *   novum Foo(x, y) -> Foo.init(x, y)
 *   novum Foo { a: 1 } -> Foo.init(.{ .a = 1 })
 *   novum Foo de props -> Foo.init(props)
 *
 * TARGET: Zig doesn't have 'new' keyword. Idiomatic pattern is Type.init().
 *         Property overrides are passed as an anonymous struct.
 *
 * WHY: When no overrides are provided, we pass .{} (empty struct) so the
 *      init() function's @hasField checks all return false, using defaults.
 */

import type { NovumExpression, Expression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genNovumExpression(node: NovumExpression, g: ZigGenerator): string {
    const callee = node.callee.name;

    // Handle { ... } or de expr overrides
    if (node.withExpression) {
        const overrides = g.genExpression(node.withExpression);
        return `${callee}.init(${overrides})`;
    }

    // Regular constructor call with arguments
    if (node.arguments.length > 0) {
        const args = node.arguments
            .filter((arg): arg is Expression => arg.type !== 'SpreadElement')
            .map(a => g.genExpression(a))
            .join(', ');
        return `${callee}.init(${args})`;
    }

    // No arguments and no overrides: pass empty struct for @hasField pattern
    return `${callee}.init(.{})`;
}
