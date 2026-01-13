/**
 * Faber Code Generator - NovumExpression
 */

import type { NovumExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genNovumExpression(node: NovumExpression, g: FabGenerator): string {
    const callee = node.callee.name;
    const args = node.arguments.map(arg => {
        if (arg.type === 'SpreadElement') {
            return `sparge ${g.genExpression(arg.argument)}`;
        }
        return g.genExpression(arg);
    });

    let result = `novum ${callee}`;

    if (args.length > 0) {
        result += `(${args.join(', ')})`;
    }

    if (node.withExpression) {
        result += ` de ${g.genExpression(node.withExpression)}`;
    }

    return result;
}
