/**
 * Zig Code Generator - InStatement (with/mutation block)
 *
 * TRANSFORMS:
 *   in user { nomen = "Marcus" } -> user.nomen = "Marcus";
 *
 * TARGET: Zig doesn't have with-blocks, we expand to member assignments.
 */

import type { InStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genInStatement(node: InStatement, g: ZigGenerator): string {
    const context = g.genExpression(node.object);
    const lines: string[] = [];

    for (const stmt of node.body.body) {
        if (stmt.type === 'ExpressionStatement' && stmt.expression.type === 'AssignmentExpression' && stmt.expression.left.type === 'Identifier') {
            const prop = stmt.expression.left.name;
            const value = g.genExpression(stmt.expression.right);
            const op = stmt.expression.operator;

            lines.push(`${g.ind()}${context}.${prop} ${op} ${value};`);
        } else {
            lines.push(g.genStatement(stmt));
        }
    }

    return lines.join('\n');
}
