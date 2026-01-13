/**
 * Faber Code Generator - PraefixumExpression (comptime)
 */

import type { PraefixumExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genPraefixumExpression(node: PraefixumExpression, g: FabGenerator): string {
    if (node.body.type === 'BlockStatement') {
        return `praefixum ${genBlockStatement(node.body, g)}`;
    }

    const expr = g.genExpression(node.body);
    return `praefixum(${expr})`;
}
