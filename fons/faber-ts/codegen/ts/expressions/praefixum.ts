/**
 * TypeScript Code Generator - Praefixum Expression (compile-time eval)
 *
 * TRANSFORMS:
 *   praefixum(expr) -> (expr)
 *   praefixum { ... } -> (() => { ... })()
 *
 * TARGET: TypeScript lacks compile-time evaluation. We emit an IIFE
 *         so the code compiles and runs, even though it won't have
 *         true compile-time semantics.
 *
 * WHY: Rather than crashing the build, we degrade gracefully.
 *      The code still works correctly at runtime - it just doesn't
 *      benefit from compile-time optimization.
 */

import type { PraefixumExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genPraefixumExpression(node: PraefixumExpression, g: TsGenerator): string {
    if (node.body.type === 'BlockStatement') {
        // Block form: wrap in IIFE
        const body = genBlockStatement(node.body, g);
        return `(() => ${body})()`;
    }

    // Expression form: just parenthesize
    return `(${g.genExpression(node.body)})`;
}
