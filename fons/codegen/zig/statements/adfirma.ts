/**
 * Zig Code Generator - AdfirmaStatement (assert)
 *
 * TRANSFORMS:
 *   adfirma x > 0 -> std.debug.assert(x > 0)
 *   adfirma x > 0, "msg" -> if (!(x > 0)) @panic("msg")
 *
 * TARGET: Zig has std.debug.assert() for assertions.
 *         For custom messages, we use @panic.
 */

import type { AdfirmaStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genAdfirmaStatement(node: AdfirmaStatement, g: ZigGenerator): string {
    const test = g.genExpression(node.test);

    if (node.message) {
        const message = g.genExpression(node.message);

        return `${g.ind()}if (!(${test})) @panic(${message});`;
    }

    return `${g.ind()}std.debug.assert(${test});`;
}
