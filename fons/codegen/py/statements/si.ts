/**
 * Python Code Generator - SiStatement (if/elif/else)
 *
 * TRANSFORMS:
 *   si (conditio) { ... } -> if conditio:
 *   si (conditio) { ... } secus { ... } -> if conditio: ... else: ...
 *   si (conditio) { ... } sin (conditio2) { ... } -> if conditio: ... elif conditio2: ...
 */

import type { SiStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genSiStatement(node: SiStatement, g: PyGenerator): string {
    const lines: string[] = [];

    // Handle catch clause by wrapping in try
    if (node.catchClause) {
        lines.push(`${g.ind()}try:`);
        g.depth++;
    }

    lines.push(`${g.ind()}if ${g.genExpression(node.test)}:`);
    g.depth++;
    lines.push(g.genBlockStatementContent(node.consequent));
    g.depth--;

    if (node.alternate) {
        if (node.alternate.type === 'SiStatement') {
            // elif chain
            const elifLines = genSiStatement(node.alternate, g).split('\n');
            // Replace 'if' with 'elif' on first line
            if (elifLines[0]) {
                elifLines[0] = elifLines[0].replace(/^(\s*)if /, '$1elif ');
            }
            lines.push(elifLines.join('\n'));
        } else {
            lines.push(`${g.ind()}else:`);
            g.depth++;
            lines.push(g.genBlockStatementContent(node.alternate));
            g.depth--;
        }
    }

    if (node.catchClause) {
        g.depth--;
        lines.push(`${g.ind()}except Exception as ${node.catchClause.param.name}:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(node.catchClause.body));
        g.depth--;
    }

    return lines.join('\n');
}
