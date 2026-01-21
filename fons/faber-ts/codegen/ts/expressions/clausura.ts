/**
 * TypeScript Code Generator - Clausura Expression
 *
 * TRANSFORMS:
 *   clausura x: x * 2                    -> (x) => x * 2
 *   clausura x, y: x + y                 -> (x, y) => x + y
 *   clausura: 42                         -> () => 42
 *   clausura numerus x: x * 2            -> (x: number) => x * 2
 *   clausura numerus x, numerus y: x + y -> (x: number, y: number) => x + y
 *   clausura x { redde x * 2 }           -> (x) => { return x * 2; }
 *   clausura { scribe "hi" }             -> () => { console.log("hi"); }
 *   clausura c { cede fetch() }          -> async (c) => { await fetch(); }
 *
 * WHY: Latin clausura (closure) creates arrow functions. Typed params get TS annotations.
 *      Async is inferred from presence of 'cede' in the body.
 */

import type { ClausuraExpression, Expression, Statement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

/**
 * Check if an expression or statement contains a cede (await) expression.
 */
function containsCede(node: Expression | Statement): boolean {
    if (node.type === 'CedeExpression') {
        return true;
    }
    if (node.type === 'BlockStatement') {
        return node.body.some(containsCede);
    }
    // Check common expression types that might contain cede
    if (node.type === 'BinaryExpression') {
        return containsCede(node.left) || containsCede(node.right);
    }
    if (node.type === 'UnaryExpression') {
        return containsCede(node.argument);
    }
    if (node.type === 'CallExpression') {
        return node.arguments.some(arg =>
            arg.type === 'SpreadElement' ? containsCede(arg.argument) : containsCede(arg)
        );
    }
    if (node.type === 'MemberExpression') {
        return containsCede(node.object);
    }
    if (node.type === 'ConditionalExpression') {
        return containsCede(node.test) || containsCede(node.consequent) || containsCede(node.alternate);
    }
    if (node.type === 'ArrayExpression') {
        return node.elements.some(e => e.type === 'SpreadElement' ? containsCede(e.argument) : containsCede(e));
    }
    if (node.type === 'ExpressionStatement') {
        return containsCede(node.expression);
    }
    if (node.type === 'VariaDeclaration') {
        return node.init ? containsCede(node.init) : false;
    }
    if (node.type === 'ReddeStatement') {
        return node.argument ? containsCede(node.argument) : false;
    }
    if (node.type === 'SiStatement') {
        return containsCede(node.test) ||
               node.consequent.body.some(containsCede) ||
               (node.alternate ? containsCede(node.alternate) : false);
    }
    return false;
}

export function genClausuraExpression(node: ClausuraExpression, g: TsGenerator): string {
    const params = node.params.map(p => g.genParameter(p)).join(', ');

    // Infer async from presence of cede in body
    const isAsync = containsCede(node.body);
    const asyncPrefix = isAsync ? 'async ' : '';

    // Add return type annotation if present
    const returnTypeAnno = node.returnType ? `: ${g.genType(node.returnType)}` : '';

    if (node.body.type === 'BlockStatement') {
        const body = genBlockStatement(node.body, g);
        return `${asyncPrefix}(${params})${returnTypeAnno} => ${body}`;
    }

    const body = g.genExpression(node.body);
    return `${asyncPrefix}(${params})${returnTypeAnno} => ${body}`;
}
