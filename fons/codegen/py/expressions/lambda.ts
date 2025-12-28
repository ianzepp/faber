/**
 * Python Code Generator - Lambda Expression
 *
 * TRANSFORMS:
 *   pro x redde x + 1               -> lambda x: x + 1
 *   pro x, y redde x + y            -> lambda x, y: x + y
 *   pro x { redde x + 1 }           -> lambda x: x + 1
 *   pro x { ... complex ... }       -> lambda x: None  (fallback)
 *
 * TARGET: Python lambdas don't support type annotations, so returnType
 *         is ignored. Use def with type hints for typed functions.
 *
 * WHY: Python lambdas are restricted to single expressions.
 *      Complex block bodies can't be represented in lambda syntax.
 *      For these cases, we emit None as a fallback - ideally they
 *      should be lifted to named functions (def).
 */

import type { Expression, LambdaExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genLambdaExpression(node: LambdaExpression, g: PyGenerator): string {
    // Note: node.returnType is ignored in Python - lambdas can't have type hints
    const params = node.params.map(p => p.name).join(', ');

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
    return `lambda ${params}: None`;
}
