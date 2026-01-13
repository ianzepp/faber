/**
 * Faber Code Generator - IteratioStatement (for loop)
 */

import type { IteratioStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genIteratioStatement(node: IteratioStatement, g: FabGenerator): string {
    const keyword = node.kind === 'ex' ? 'ex' : 'de';
    const binding = node.async ? 'fiet' : 'pro';

    let result = `${g.ind()}${keyword} ${g.genExpression(node.iterable)} ${binding} ${node.variable.name} ${genBlockStatement(node.body, g)}`;

    if (node.catchClause) {
        result += ` cape ${node.catchClause.param.name} ${genBlockStatement(node.catchClause.body, g)}`;
    }

    return result;
}
