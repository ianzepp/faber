/**
 * C++23 Code Generator - SiStatement
 *
 * TRANSFORMS:
 *   si x > 0 { ... } -> if (x > 0) { ... }
 *   si x > 0 { ... } secus { ... } -> if (x > 0) { ... } else { ... }
 *   si x > 0 { ... } alioqui y > 0 { ... } -> if (x > 0) { ... } else if (y > 0) { ... }
 */

import type { SiStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genSiStatement(node: SiStatement, g: CppGenerator): string {
    let result = `${g.ind()}if (${g.genExpression(node.test)}) ${genBlockStatement(node.consequent, g)}`;

    if (node.alternate) {
        if (node.alternate.type === 'SiStatement') {
            result += ` else ${genSiStatement(node.alternate, g).trim()}`;
        } else {
            result += ` else ${genBlockStatement(node.alternate, g)}`;
        }
    }

    return result;
}
