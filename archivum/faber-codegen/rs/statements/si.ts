/**
 * Rust Code Generator - SiStatement
 *
 * TRANSFORMS:
 *   si x > 0 { } -> if x > 0 { }
 *   si x > 0 { } secus { } -> if x > 0 { } else { }
 *   si x > 0 { } aut si y > 0 { } -> if x > 0 { } else if y > 0 { }
 */

import type { SiStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genSiStatement(node: SiStatement, g: RsGenerator): string {
    let result = `${g.ind()}if ${g.genExpression(node.test)} ${genBlockStatement(node.consequent, g)}`;

    if (node.alternate) {
        if (node.alternate.type === 'SiStatement') {
            result += ` else ${genSiStatement(node.alternate, g).trim()}`;
        } else {
            result += ` else ${genBlockStatement(node.alternate, g)}`;
        }
    }

    return result;
}
