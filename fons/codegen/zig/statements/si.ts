/**
 * Zig Code Generator - SiStatement (if)
 *
 * TRANSFORMS:
 *   si x > 0 { a() } -> if (x > 0) { a(); }
 *   si x > 0 { a() } secus { b() } -> if (x > 0) { a(); } else { b(); }
 *   si x > 0 { a() } aut si y > 0 { b() } -> if (x > 0) { a(); } else if (y > 0) { b(); }
 *
 * TARGET: Zig uses if/else keywords like most languages.
 */

import type { SiStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genSiStatement(node: SiStatement, g: ZigGenerator): string {
    let result = '';

    // Zig doesn't have try/catch like JS, we'll use error handling differently
    // For now, ignore catchClause in Zig output
    result += `${g.ind()}if (${g.genExpression(node.test)}) ${g.genBlockStatement(node.consequent)}`;

    if (node.alternate) {
        if (node.alternate.type === 'SiStatement') {
            result += ` else ${genSiStatement(node.alternate, g).trim()}`;
        } else {
            result += ` else ${g.genBlockStatement(node.alternate)}`;
        }
    }

    return result;
}
