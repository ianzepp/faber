/**
 * C++23 Code Generator - LambdaExpression
 *
 * TRANSFORMS:
 *   pro x: x + 1             -> [&](auto x) { return x + 1; }
 *   pro x -> numerus: x + 1  -> [&](auto x) -> int64_t { return x + 1; }
 *
 * TARGET: C++ lambdas support trailing return types with ->
 */

import type { LambdaExpression, Expression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genLambdaExpression(node: LambdaExpression, g: CppGenerator): string {
    const params = node.params.map(p => `auto ${p.name}`).join(', ');

    // Add return type annotation if present
    const returnTypeAnno = node.returnType ? ` -> ${g.genType(node.returnType)}` : '';

    if (node.body.type === 'BlockStatement') {
        const body = genBlockStatement(node.body, g);

        return `[&](${params})${returnTypeAnno} ${body}`;
    }

    const body = g.genExpression(node.body as Expression);

    return `[&](${params})${returnTypeAnno} { return ${body}; }`;
}
