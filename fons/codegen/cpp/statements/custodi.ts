/**
 * C++23 Code Generator - CustodiStatement
 *
 * TRANSFORMS:
 *   custodi x > 0 { redde } -> if (x > 0) { return; }
 */

import type { CustodiStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genCustodiStatement(node: CustodiStatement, g: CppGenerator): string {
    const lines: string[] = [];

    for (const clause of node.clauses) {
        const test = g.genExpression(clause.test);
        const body = genBlockStatement(clause.consequent, g);

        lines.push(`${g.ind()}if (${test}) ${body}`);
    }

    return lines.join('\n');
}
