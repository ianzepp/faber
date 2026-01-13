/**
 * Python Code Generator - CustodiStatement
 *
 * TRANSFORMS:
 *   custodi x > 0 { ... } -> if x > 0: ...
 *   custodi x > 0, y > 0 { ... } -> if x > 0: ...; if y > 0: ...
 *
 * WHY: Guard statements in Latin become simple if statements in Python.
 *      Multiple guard clauses are emitted as consecutive if blocks.
 */

import type { CustodiStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genCustodiStatement(node: CustodiStatement, g: PyGenerator): string {
    const lines: string[] = [];

    for (const clause of node.clauses) {
        const test = g.genExpression(clause.test);
        lines.push(`${g.ind()}if ${test}:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(clause.consequent));
        g.depth--;
    }

    return lines.join('\n');
}
