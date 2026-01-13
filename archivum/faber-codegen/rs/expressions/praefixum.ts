/**
 * Rust Code Generator - Praefixum Expression
 *
 * TRANSFORMS:
 *   praefixum { 2 + 2 } -> const { 2 + 2 }
 */

import type { PraefixumExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genPraefixumExpression(node: PraefixumExpression, g: RsGenerator): string {
    // Rust const evaluation
    if (node.body.type === 'BlockStatement') {
        return `const ${genBlockStatement(node.body, g)}`;
    }
    return `const { ${g.genExpression(node.body)} }`;
}
