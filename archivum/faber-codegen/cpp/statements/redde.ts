/**
 * C++23 Code Generator - ReddeStatement
 *
 * TRANSFORMS:
 *   redde x -> return x;
 *   redde -> return;
 */

import type { ReddeStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genReddeStatement(node: ReddeStatement, g: CppGenerator): string {
    if (node.argument) {
        return `${g.ind()}return ${g.genExpression(node.argument)};`;
    }

    return `${g.ind()}return;`;
}
