/**
 * Rust Code Generator - ReddeStatement
 *
 * TRANSFORMS:
 *   redde 42 -> return 42;
 *   redde -> return;
 */

import type { ReddeStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genReddeStatement(node: ReddeStatement, g: RsGenerator): string {
    if (node.argument) {
        return `${g.ind()}return ${g.genExpression(node.argument)};`;
    }
    return `${g.ind()}return;`;
}
