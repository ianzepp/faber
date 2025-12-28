/**
 * Python Code Generator - DumStatement (while loop)
 *
 * TRANSFORMS:
 *   dum (conditio) { ... } -> while conditio: ...
 *   dum (conditio) { ... } cape e { ... } -> try: while conditio: ... except Exception as e: ...
 *
 * WHY: Python uses `while` for condition-based loops.
 *      Optional catch clause wraps in try-except.
 */

import type { DumStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genDumStatement(node: DumStatement, g: PyGenerator): string {
    const lines: string[] = [];

    if (node.catchClause) {
        lines.push(`${g.ind()}try:`);
        g.depth++;
    }

    lines.push(`${g.ind()}while ${g.genExpression(node.test)}:`);
    g.depth++;
    lines.push(g.genBlockStatementContent(node.body));
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
