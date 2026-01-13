/**
 * Faber Code Generator - IaceStatement (throw/panic)
 */

import type { IaceStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genIaceStatement(node: IaceStatement, g: FabGenerator): string {
    const keyword = node.fatal ? 'mori' : 'iace';
    return `${g.ind()}${keyword} ${g.genExpression(node.argument)}`;
}
