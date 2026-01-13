/**
 * Python Code Generator - EligeStatement
 *
 * Generate switch statement using match/case (Python 3.10+).
 *
 * Value matching only; use discerne for variant matching.
 */

import type { EligeStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genEligeStatement(node: EligeStatement, g: PyGenerator): string {
    const lines: string[] = [];
    const discriminant = g.genExpression(node.discriminant);

    if (node.catchClause) {
        lines.push(`${g.ind()}try:`);
        g.depth++;
    }

    lines.push(`${g.ind()}match ${discriminant}:`);
    g.depth++;

    for (const caseNode of node.cases) {
        // Value matching: si expression { ... }
        const test = g.genExpression(caseNode.test);
        lines.push(`${g.ind()}case ${test}:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(caseNode.consequent));
        g.depth--;
    }

    if (node.defaultCase) {
        lines.push(`${g.ind()}case _:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(node.defaultCase));
        g.depth--;
    }

    g.depth--;

    if (node.catchClause) {
        g.depth--;
        lines.push(`${g.ind()}except Exception as ${node.catchClause.param.name}:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(node.catchClause.body));
        g.depth--;
    }

    return lines.join('\n');
}
