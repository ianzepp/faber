/**
 * TypeScript Code Generator - CustodiStatement
 *
 * TRANSFORMS:
 *   custodi { si x == nihil { redde } si y < 0 { iace "error" } }
 *   -> if (x == null) { return; } if (y < 0) { throw "error"; }
 *
 * WHY: Guard clauses are just sequential if statements with early exits.
 */

import type { CustodiStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genCustodiStatement(node: CustodiStatement, g: TsGenerator): string {
    const lines: string[] = [];

    for (const clause of node.clauses) {
        const test = g.genExpression(clause.test);
        const body = genBlockStatement(clause.consequent, g);

        lines.push(`${g.ind()}if (${test}) ${body}`);
    }

    return lines.join('\n');
}
