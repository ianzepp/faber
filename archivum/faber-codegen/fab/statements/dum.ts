/**
 * Faber Code Generator - DumStatement (while loop)
 */

import type { DumStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genDumStatement(node: DumStatement, g: FabGenerator): string {
    let result = `${g.ind()}dum ${g.genExpression(node.test)} ${genBlockStatement(node.body, g)}`;

    if (node.catchClause) {
        result += ` cape ${node.catchClause.param.name} ${genBlockStatement(node.catchClause.body, g)}`;
    }

    return result;
}
