/**
 * Python Code Generator - FacBlockStatement
 *
 * Generates Python try/except statements from Latin fac/cape blocks (do-catch).
 *
 * TRANSFORMS:
 *   fac { x() } cape e { y() } -> try: x() except Exception as e: y()
 *   fac { x() }                -> x()  (no wrapping if no catch)
 *   fac { x() } dum cond       -> while True: x(); if not (cond): break
 *   fac { x() } cape e { y() } dum cond -> try: while True: x(); if not (cond): break except ...
 *
 * WHY: fac is a simpler variant of tempta - just an inline block with optional catch.
 *      Without a catch clause, the block contents are emitted directly.
 *      With dum, it becomes a do-while loop (Python lacks native do-while).
 */

import type { FacBlockStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genFacBlockStatement(node: FacBlockStatement, g: PyGenerator): string {
    const lines: string[] = [];

    if (node.test) {
        // Do-while loop: fac { body } dum condition
        // Python has no do-while, so use: while True: body; if not (cond): break
        const test = g.genExpression(node.test);

        if (node.catchClause) {
            lines.push(`${g.ind()}try:`);
            g.depth++;
        }

        lines.push(`${g.ind()}while True:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(node.body));
        lines.push(`${g.ind()}if not (${test}):`);
        g.depth++;
        lines.push(`${g.ind()}break`);
        g.depth--;
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

    // If there's a catch clause, wrap in try-except
    if (node.catchClause) {
        lines.push(`${g.ind()}try:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(node.body));
        g.depth--;
        lines.push(`${g.ind()}except Exception as ${node.catchClause.param.name}:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(node.catchClause.body));
        g.depth--;
    }
    else {
        // No catch - just emit the block contents
        lines.push(g.genBlockStatementContent(node.body));
    }

    return lines.join('\n');
}
