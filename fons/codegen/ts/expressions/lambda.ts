/**
 * TypeScript Code Generator - Lambda Expression
 *
 * TRANSFORMS:
 *   pro x redde x * 2 -> (x) => x * 2
 *   pro x, y redde x + y -> (x, y) => x + y
 *   pro redde 42 -> () => 42
 *   pro x { redde x * 2 } -> (x) => { return x * 2; }
 *   pro { scribe "hi" } -> () => { console.log("hi"); }
 *
 * WHY: Latin pro (for) + redde (return) creates arrow functions.
 */

import type { LambdaExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genLambdaExpression(node: LambdaExpression, g: TsGenerator): string {
    const params = node.params.map(p => p.name).join(', ');
    const asyncPrefix = node.async ? 'async ' : '';

    // Add return type annotation if present
    const returnTypeAnno = node.returnType ? `: ${g.genType(node.returnType)}` : '';

    if (node.body.type === 'BlockStatement') {
        const body = genBlockStatement(node.body, g);
        return `${asyncPrefix}(${params})${returnTypeAnno} => ${body}`;
    }

    const body = g.genExpression(node.body);
    return `${asyncPrefix}(${params})${returnTypeAnno} => ${body}`;
}
