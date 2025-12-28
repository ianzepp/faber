/**
 * Zig Code Generator - CustodiStatement (guard clauses)
 *
 * TRANSFORMS:
 *   custodi { si x == nihil { redde } }
 *   -> if (x == null) { return; }
 *
 * TARGET: Guards are just sequential if statements in Zig too.
 */

import type { CustodiStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genCustodiStatement(node: CustodiStatement, g: ZigGenerator): string {
    const lines: string[] = [];

    for (const clause of node.clauses) {
        const test = g.genExpression(clause.test);
        const body = g.genBlockStatement(clause.consequent);

        lines.push(`${g.ind()}if (${test}) ${body}`);
    }

    return lines.join('\n');
}
