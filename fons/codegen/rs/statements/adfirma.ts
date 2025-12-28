/**
 * Rust Code Generator - AdfirmaStatement
 *
 * TRANSFORMS:
 *   adfirma x > 0 -> assert!(x > 0);
 *   adfirma x > 0, "message" -> assert!(x > 0, "message");
 */

import type { AdfirmaStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genAdfirmaStatement(node: AdfirmaStatement, g: RsGenerator): string {
    const test = g.genExpression(node.test);

    if (node.message) {
        const msg = g.genExpression(node.message);
        return `${g.ind()}assert!(${test}, ${msg});`;
    }

    return `${g.ind()}assert!(${test});`;
}
