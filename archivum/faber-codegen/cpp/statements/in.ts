/**
 * C++23 Code Generator - InStatement
 *
 * TRANSFORMS:
 *   in user { nomen = "Marcus" } -> { user.nomen = "Marcus"; }
 */

import type { InStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genInStatement(node: InStatement, g: CppGenerator): string {
    const context = g.genExpression(node.object);
    const lines: string[] = [];

    lines.push(`${g.ind()}{`);
    g.depth++;

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

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
