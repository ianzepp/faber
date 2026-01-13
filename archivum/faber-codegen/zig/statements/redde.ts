/**
 * Zig Code Generator - ReddeStatement (return)
 *
 * TRANSFORMS:
 *   redde x -> return x;
 *   redde   -> return;
 */

import type { ReddeStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genReddeStatement(node: ReddeStatement, g: ZigGenerator): string {
    if (node.argument) {
        return `${g.ind()}return ${g.genExpression(node.argument)};`;
    }

    return `${g.ind()}return;`;
}
