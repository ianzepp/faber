/**
 * TypeScript Code Generator - AdfirmaStatement
 *
 * TRANSFORMS:
 *   adfirma x > 0 -> if (!(x > 0)) throw new Error("Assertion failed: x > 0");
 *   adfirma x > 0, "custom" -> if (!(x > 0)) throw new Error("custom");
 *
 * WHY: Always-on runtime checks that throw on failure.
 *      Auto-generates message from expression if not provided.
 */

import type { AdfirmaStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genAdfirmaStatement(node: AdfirmaStatement, g: TsGenerator): string {
    const test = g.genExpression(node.test);

    let message: string;

    if (node.message) {
        message = g.genExpression(node.message);
    } else {
        // Auto-generate message from the test expression
        message = `"Assertion failed: ${test.replace(/"/g, '\\"')}"`;
    }

    return `${g.ind()}if (!(${test})) { throw new Error(${message}); }`;
}
