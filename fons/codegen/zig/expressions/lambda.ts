/**
 * Zig Code Generator - Lambda Expression (pro syntax)
 *
 * TRANSFORMS:
 *   pro x -> numerus: x * 2 -> struct { fn call(x: anytype) i64 { return x * 2; } }.call
 *   pro x: x * 2 -> @compileError("Lambda requires return type for Zig")
 *
 * TARGET: Zig doesn't have lambdas/closures as first-class values.
 *         We emulate with anonymous struct containing a function.
 *         This ONLY works when a return type annotation is provided,
 *         because anytype can't be used as a return type in Zig.
 *
 * LIMITATION: Closures are not properly supported - captured variables
 *             would need to be passed explicitly via context struct.
 */

import type { LambdaExpression, Expression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genLambdaExpression(node: LambdaExpression, g: ZigGenerator): string {
    // GUARD: No return type - Zig can't infer lambda return types
    if (!node.returnType) {
        return `@compileError("Lambda requires return type annotation for Zig target: pro x -> Type: expr")`;
    }

    const params = node.params.map(p => `${p.name}: anytype`).join(', ');
    const returnType = g.genType(node.returnType);

    // GUARD: Block body - generate full function block
    if (node.body.type === 'BlockStatement') {
        const body = g.genBlockStatement(node.body);
        return `struct { fn call(${params}) ${returnType} ${body} }.call`;
    }

    // Expression body - wrap in return statement
    const body = g.genExpression(node.body as Expression);
    return `struct { fn call(${params}) ${returnType} { return ${body}; } }.call`;
}
