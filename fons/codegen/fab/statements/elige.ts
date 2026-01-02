/**
 * Faber Code Generator - EligeStatement (switch)
 */

import type { EligeStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genEligeStatement(node: EligeStatement, g: FabGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}elige ${g.genExpression(node.discriminant)} {`);

    g.depth++;
    for (const c of node.cases) {
        lines.push(`${g.ind()}casu ${g.genExpression(c.test)} ${genBlockStatement(c.consequent, g)}`);
    }

    if (node.defaultCase) {
        lines.push(`${g.ind()}ceterum ${genBlockStatement(node.defaultCase, g)}`);
    }
    g.depth--;

    lines.push(`${g.ind()}}`);

    if (node.catchClause) {
        lines[lines.length - 1] += ` cape ${node.catchClause.param.name} ${genBlockStatement(node.catchClause.body, g)}`;
    }

    return lines.join('\n');
}
