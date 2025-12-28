/**
 * C++23 Code Generator - PraefixumExpression
 *
 * TRANSFORMS:
 *   praefixum(256 * 4)     -> (256 * 4)  (evaluated at compile time in constexpr context)
 *   praefixum { ... redde x } -> [&]{ ... return x; }()  (IIFE)
 *
 * WHY: C++23 constexpr is limited to certain operations. For simple expressions,
 *      we use constexpr. For blocks, we emit an IIFE (which may not be constexpr).
 */

import type { PraefixumExpression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genPraefixumExpression(node: PraefixumExpression, g: CppGenerator): string {
    if (node.body.type === 'BlockStatement') {
        // Block form: emit as IIFE
        // WHY: constexpr lambdas exist but have restrictions. For now, just use regular IIFE.
        const body = genBlockStatement(node.body, g);

        return `[&]${body}()`;
    }

    // Expression form: wrap in constexpr evaluation
    // WHY: In expression context, we can't declare a constexpr variable.
    //      Just parenthesize and trust the optimizer.
    return `(${g.genExpression(node.body)})`;
}
