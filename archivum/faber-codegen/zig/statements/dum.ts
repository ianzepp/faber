/**
 * Zig Code Generator - DumStatement (while)
 *
 * TRANSFORMS:
 *   dum x > 0 { x = x - 1 } -> while (x > 0) { x = x - 1; }
 *
 * TARGET: Zig uses while keyword like most languages.
 */

import type { DumStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genDumStatement(node: DumStatement, g: ZigGenerator): string {
    const test = g.genExpression(node.test);
    const body = g.genBlockStatement(node.body);

    return `${g.ind()}while (${test}) ${body}`;
}
