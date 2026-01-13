/**
 * Python Code Generator - InStatement
 *
 * TRANSFORMS:
 *   in user { nomen = "Marcus" } -> user.nomen = "Marcus"
 *
 * WHY: Python has no dedicated "with object" mutation block syntax.
 *      We expand the block, prefixing each property assignment with
 *      the context object.
 */

import type { InStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genInStatement(node: InStatement, g: PyGenerator): string {
    const context = g.genExpression(node.object);
    const lines: string[] = [];

    for (const stmt of node.body.body) {
        if (stmt.type === 'ExpressionStatement' && stmt.expression.type === 'AssignmentExpression' && stmt.expression.left.type === 'Identifier') {
            const prop = stmt.expression.left.name;
            const value = g.genExpression(stmt.expression.right);
            const op = stmt.expression.operator;
            lines.push(`${g.ind()}${context}.${prop} ${op} ${value}`);
        } else {
            lines.push(g.genStatement(stmt));
        }
    }

    return lines.join('\n');
}
