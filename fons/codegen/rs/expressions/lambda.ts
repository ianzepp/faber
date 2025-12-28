/**
 * Rust Code Generator - Lambda Expression
 *
 * TRANSFORMS:
 *   \x -> x + 1 -> |x| x + 1
 *   \x, y -> x + y -> |x, y| x + y
 *   futura \x -> fetchData(x) -> async |x| fetchData(x)
 */

import type { LambdaExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genLambdaExpression(node: LambdaExpression, g: RsGenerator): string {
    const params = node.params.map(p => p.name).join(', ');
    const asyncPrefix = node.async ? 'async ' : '';

    if (node.body.type === 'BlockStatement') {
        const body = genBlockStatement(node.body, g);
        return `${asyncPrefix}|${params}| ${body}`;
    }

    const body = g.genExpression(node.body);
    return `${asyncPrefix}|${params}| ${body}`;
}
