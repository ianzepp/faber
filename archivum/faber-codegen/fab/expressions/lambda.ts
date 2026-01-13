/**
 * Faber Code Generator - LambdaExpression
 *
 * STYLE: Uses pro x: expr (canonical), not pro x redde expr
 */

import type { LambdaExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from '../statements/functio';

export function genLambdaExpression(node: LambdaExpression, g: FabGenerator): string {
    const params = node.params.map(p => p.name).join(', ');
    const returnType = node.returnType ? ` -> ${g.genType(node.returnType)}` : '';

    if (node.body.type === 'BlockStatement') {
        const body = genBlockStatement(node.body, g);
        return `pro ${params}${returnType} ${body}`;
    }

    // Expression form: use : (canonical)
    const body = g.genExpression(node.body);
    return `pro ${params}${returnType}: ${body}`;
}
