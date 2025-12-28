/**
 * Rust Code Generator - Arrow Function Expression
 *
 * TRANSFORMS:
 *   (x) => x + 1 -> |x| x + 1
 *   (x) => { redde x + 1 } -> |x| { return x + 1; }
 */

import type { ArrowFunctionExpression, Expression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genArrowFunction(node: ArrowFunctionExpression, g: RsGenerator): string {
    const params = node.params.map(p => p.name.name).join(', ');

    if (node.body.type === 'BlockStatement') {
        const body = genBlockStatement(node.body, g);
        return `|${params}| ${body}`;
    }

    const body = g.genExpression(node.body as Expression);
    return `|${params}| ${body}`;
}
