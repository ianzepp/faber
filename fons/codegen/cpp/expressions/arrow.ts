/**
 * C++23 Code Generator - ArrowFunctionExpression
 *
 * TRANSFORMS:
 *   (x) => x + 1     -> [&](auto x) { return x + 1; }
 *   (x, y) => x + y  -> [&](auto x, auto y) { return x + y; }
 *   () => { ... }    -> [&]() { ... }
 */

import type { ArrowFunctionExpression, Expression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genArrowFunction(node: ArrowFunctionExpression, g: CppGenerator): string {
    const params = node.params
        .map(p => {
            const name = p.name.name;

            if (p.typeAnnotation) {
                return `${g.genType(p.typeAnnotation)} ${name}`;
            }

            return `auto ${name}`;
        })
        .join(', ');

    if (node.body.type === 'BlockStatement') {
        const body = genBlockStatement(node.body, g);

        return `[&](${params}) ${body}`;
    }

    const body = g.genExpression(node.body as Expression);

    return `[&](${params}) { return ${body}; }`;
}
