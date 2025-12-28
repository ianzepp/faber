/**
 * TypeScript Code Generator - Arrow Function Expression
 *
 * TRANSFORMS:
 *   (x) => x + 1 -> (x) => x + 1
 *   (x) => { redde x + 1; } -> (x) => { return x + 1; }
 */

import type { ArrowFunctionExpression, Expression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genArrowFunction(node: ArrowFunctionExpression, g: TsGenerator): string {
    const params = node.params.map(p => g.genParameter(p)).join(', ');

    if (node.body.type === 'BlockStatement') {
        const body = genBlockStatement(node.body, g);

        return `(${params}) => ${body}`;
    }

    const body = g.genExpression(node.body as Expression);

    return `(${params}) => ${body}`;
}
