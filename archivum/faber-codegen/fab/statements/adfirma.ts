/**
 * Faber Code Generator - AdfirmaStatement (assert)
 */

import type { AdfirmaStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genAdfirmaStatement(node: AdfirmaStatement, g: FabGenerator): string {
    if (node.message) {
        return `${g.ind()}adfirma ${g.genExpression(node.test)}, ${g.genExpression(node.message)}`;
    }
    return `${g.ind()}adfirma ${g.genExpression(node.test)}`;
}
