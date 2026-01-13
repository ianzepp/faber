/**
 * Faber Code Generator - CustodiStatement (guard)
 */

import type { CustodiStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genCustodiStatement(node: CustodiStatement, g: FabGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}custodi {`);

    g.depth++;
    for (const clause of node.clauses) {
        lines.push(`${g.ind()}si ${g.genExpression(clause.test)} ${genBlockStatement(clause.consequent, g)}`);
    }
    g.depth--;

    lines.push(`${g.ind()}}`);
    return lines.join('\n');
}
