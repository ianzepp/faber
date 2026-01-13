/**
 * C++23 Code Generator - AdfirmaStatement
 *
 * TRANSFORMS:
 *   adfirma x > 0 -> assert(x > 0);
 *   adfirma x > 0, "x must be positive" -> assert((x > 0) && "x must be positive");
 */

import type { AdfirmaStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genAdfirmaStatement(node: AdfirmaStatement, g: CppGenerator): string {
    g.includes.add('<cassert>');

    const test = g.genExpression(node.test);

    if (node.message) {
        // C++ assert doesn't support messages easily, use comma operator trick
        const msg = g.genExpression(node.message);

        return `${g.ind()}assert((${test}) && ${msg});`;
    }

    return `${g.ind()}assert(${test});`;
}
