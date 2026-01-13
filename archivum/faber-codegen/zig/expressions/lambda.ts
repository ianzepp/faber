/**
 * Zig Code Generator - Lambda Expression (pro syntax)
 *
 * TRANSFORMS:
 *   pro x -> numerus: x * 2 -> struct { fn call(x: anytype) i64 { return x * 2; } }.call
 *   pro x: x * 2 -> struct { fn call(x: anytype) bool { return x * 2; } }.call (inferred)
 *
 * TARGET: Zig doesn't have lambdas/closures as first-class values.
 *         We emulate with anonymous struct containing a function.
 *         Return type comes from explicit annotation or semantic analysis inference.
 *
 * LIMITATION: Closures are not properly supported - captured variables
 *             would need to be passed explicitly via context struct.
 */

import type { LambdaExpression, Expression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genLambdaExpression(node: LambdaExpression, g: ZigGenerator): string {
    const params = node.params.map(p => `${p.name}: anytype`).join(', ');

    // Get return type from explicit annotation or inferred from semantic analysis
    let returnType: string;

    if (node.returnType) {
        // Explicit annotation: pro x -> bivalens: x > 1
        returnType = g.genType(node.returnType);
    } else if (node.resolvedType?.kind === 'function') {
        // Inferred from semantic analysis: pro x: x > 1
        returnType = g.semanticTypeToZig(node.resolvedType.returnType);
    } else {
        // Fallback: no type info available
        return `@compileError("Lambda return type could not be inferred for Zig target. Use explicit annotation: pro x -> Type: expr")`;
    }

    // GUARD: Block body - generate full function block
    if (node.body.type === 'BlockStatement') {
        const body = g.genBlockStatement(node.body);
        return `struct { fn call(${params}) ${returnType} ${body} }.call`;
    }

    // Expression body - wrap in return statement
    const body = g.genExpression(node.body as Expression);
    return `struct { fn call(${params}) ${returnType} { return ${body}; } }.call`;
}
