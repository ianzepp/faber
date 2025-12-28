/**
 * Python Code Generator - AdfirmaStatement
 *
 * TRANSFORMS:
 *   adfirma x > 0 -> assert x > 0
 *   adfirma x > 0, "msg" -> assert x > 0, "msg"
 */

import type { AdfirmaStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genAdfirmaStatement(node: AdfirmaStatement, g: PyGenerator): string {
    const test = g.genExpression(node.test);

    if (node.message) {
        const message = g.genExpression(node.message);
        return `${g.ind()}assert ${test}, ${message}`;
    }

    return `${g.ind()}assert ${test}`;
}
