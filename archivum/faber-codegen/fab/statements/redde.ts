/**
 * Faber Code Generator - ReddeStatement (return)
 */

import type { ReddeStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genReddeStatement(node: ReddeStatement, g: FabGenerator): string {
    if (node.argument) {
        return `${g.ind()}redde ${g.genExpression(node.argument)}`;
    }
    return `${g.ind()}redde`;
}
