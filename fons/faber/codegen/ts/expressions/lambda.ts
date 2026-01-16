/**
 * TypeScript Code Generator - Lambda Expression
 *
 * TRANSFORMS:
 *   pro x: x * 2                    -> (x) => x * 2
 *   pro x, y: x + y                 -> (x, y) => x + y
 *   pro: 42                         -> () => 42
 *   pro numerus x: x * 2            -> (x: number) => x * 2
 *   pro numerus x, numerus y: x + y -> (x: number, y: number) => x + y
 *   pro x { redde x * 2 }           -> (x) => { return x * 2; }
 *   pro { scribe "hi" }             -> () => { console.log("hi"); }
 *
 * WHY: Latin pro (for) creates arrow functions. Typed params get TS annotations.
 */

import type { LambdaExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genLambdaExpression(node: LambdaExpression, g: TsGenerator): string {
    const params = node.params.map(p => g.genParameter(p)).join(', ');
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
