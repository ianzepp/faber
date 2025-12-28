/**
 * Rust Code Generator - Call Expression
 *
 * TRANSFORMS:
 *   foo(a, b) -> foo(a, b)
 *   foo?(a) -> foo.map(|f| f(a))
 */

import type { CallExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genCallExpression(node: CallExpression, g: RsGenerator): string {
    const args = node.arguments
        .map(arg => {
            if (arg.type === 'SpreadElement') {
                return `/* spread */ ${g.genExpression(arg.argument)}`;
            }
            return g.genExpression(arg);
        })
        .join(', ');

    const callee = g.genExpression(node.callee);

    if (node.optional) {
        return `${callee}.map(|f| f(${args}))`;
    }

    return `${callee}(${args})`;
}
