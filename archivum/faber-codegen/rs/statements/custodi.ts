/**
 * Rust Code Generator - CustodiStatement
 *
 * TRANSFORMS:
 *   custodi x > 0 { ... }
 *   -> if x > 0 { ... }
 *
 * WHY: Rust has no guard keyword; custodi is expressed as if statements.
 */

import type { CustodiStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genCustodiStatement(node: CustodiStatement, g: RsGenerator): string {
    const lines: string[] = [];

    for (const clause of node.clauses) {
        const test = g.genExpression(clause.test);
        const body = genBlockStatement(clause.consequent, g);
        lines.push(`${g.ind()}if ${test} ${body}`);
    }

    return lines.join('\n');
}
