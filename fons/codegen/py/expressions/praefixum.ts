/**
 * Python Code Generator - PraefixumExpression
 *
 * TRANSFORMS:
 *   praefixum(expr) -> (expr)
 *   praefixum { redde expr } -> (expr)  (single return only)
 *   praefixum { ... redde x } -> __praefixum__('''...code...''')
 *
 * TARGET: Python lacks compile-time evaluation and true IIFEs.
 *         For simple expressions and single-return blocks, we emit
 *         the expression directly. Complex blocks use __praefixum__
 *         helper with exec().
 *
 * WHY: Rather than crashing the build, we degrade gracefully.
 *      The __praefixum__ helper executes code with restricted builtins
 *      (mimicking compile-time constraints: no I/O, limited stdlib).
 */

import type { PraefixumExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genPraefixumExpression(node: PraefixumExpression, g: PyGenerator): string {
    if (node.body.type === 'BlockStatement') {
        const block = node.body;
        const lastStmt = block.body[block.body.length - 1];

        // Single return statement: just emit the expression
        if (block.body.length === 1 && lastStmt?.type === 'ReddeStatement' && lastStmt.argument) {
            return `(${g.genExpression(lastStmt.argument)})`;
        }

        // Complex block: use __praefixum__ helper with exec()
        g.features.praefixum = true;

        // Generate block statements, transforming final return to __result__ assignment
        const statements: string[] = [];
        for (let i = 0; i < block.body.length; i++) {
            const stmt = block.body[i]!;
            if (i === block.body.length - 1 && stmt.type === 'ReddeStatement' && stmt.argument) {
                // Transform final return into __result__ assignment
                statements.push(`__result__ = ${g.genExpression(stmt.argument)}`);
            } else {
                // Generate statement without leading indent (we're inside a string)
                const saved = g.depth;
                g.depth = 0;
                statements.push(g.genStatement(stmt));
                g.depth = saved;
            }
        }

        const code = statements.join('\n');
        return `__praefixum__('''${code}''')`;
    }

    // Expression form: just parenthesize
    return `(${g.genExpression(node.body)})`;
}
