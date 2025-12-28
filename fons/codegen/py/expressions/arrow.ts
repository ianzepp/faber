/**
 * Python Code Generator - Arrow Function Expression
 *
 * TRANSFORMS:
 *   (x) => x + 1                    -> lambda x: x + 1
 *   (x, y) => x + y                 -> lambda x, y: x + y
 *   (x) => { redde x + 1 }          -> lambda x: x + 1
 *   (x) => { ... complex ... }      -> lambda x: None  (fallback)
 *
 * WHY: Python lambdas are restricted to single expressions.
 *      Complex block bodies can't be represented in lambda syntax.
 *      For these cases, we emit None as a fallback - ideally they
 *      should be lifted to named functions (def).
 */

import type { ArrowFunctionExpression, Expression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genArrowFunction(node: ArrowFunctionExpression, g: PyGenerator): string {
    const params = node.params.map(p => p.name.name).join(', ');

    // Simple expression body -> lambda
    if (node.body.type !== 'BlockStatement') {
        const body = g.genExpression(node.body as Expression);
        return `lambda ${params}: ${body}`;
    }

    // Block body - extract return expression if simple
    const block = node.body;
    const firstStmt = block.body[0];
    if (block.body.length === 1 && firstStmt?.type === 'ReddeStatement') {
        if (firstStmt.argument) {
            const body = g.genExpression(firstStmt.argument);
            return `lambda ${params}: ${body}`;
        }
    }

    // Complex block body - Python lambdas can't have statements
    // Use None as fallback; these should ideally be lifted to named functions
    return `lambda ${params}: None`;
}
