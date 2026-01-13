/**
 * Python Code Generator - ReddeStatement
 *
 * TRANSFORMS:
 *   redde x -> return x
 *   redde   -> return
 */

import type { ReddeStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genReddeStatement(node: ReddeStatement, g: PyGenerator): string {
    if (node.argument) {
        return `${g.ind()}return ${g.genExpression(node.argument)}`;
    }
    return `${g.ind()}return`;
}
