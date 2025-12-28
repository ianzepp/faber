/**
 * Rust Code Generator - InStatement
 *
 * TRANSFORMS:
 *   in obj { x = 1, y = 2 }
 *   -> obj.x = 1;
 *      obj.y = 2;
 *
 * WHY: Rust doesn't have a "with" construct; we expand to individual assignments.
 */

import type { InStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genInStatement(node: InStatement, g: RsGenerator): string {
    const context = g.genExpression(node.object);
    const lines: string[] = [];

    for (const stmt of node.body.body) {
        if (stmt.type === 'ExpressionStatement' && stmt.expression.type === 'AssignmentExpression' && stmt.expression.left.type === 'Identifier') {
            const prop = stmt.expression.left.name;
            const value = g.genExpression(stmt.expression.right);
            lines.push(`${g.ind()}${context}.${prop} = ${value};`);
        } else {
            lines.push(g.genStatement(stmt));
        }
    }

    return lines.join('\n');
}
