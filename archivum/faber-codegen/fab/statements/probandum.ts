/**
 * Faber Code Generator - ProbandumStatement (test suite)
 */

import type { ProbandumStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genProbandumStatement(node: ProbandumStatement, g: FabGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}probandum "${node.name}" {`);

    g.depth++;
    for (const item of node.body) {
        lines.push(g.genStatement(item));
    }
    g.depth--;

    lines.push(`${g.ind()}}`);
    return lines.join('\n');
}
