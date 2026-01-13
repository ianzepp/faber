/**
 * Faber Code Generator - ArrayExpression
 */

import type { ArrayExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genArrayExpression(node: ArrayExpression, g: FabGenerator): string {
    const elems = node.elements.map(elem => {
        if (elem.type === 'SpreadElement') {
            return `sparge ${g.genExpression(elem.argument)}`;
        }
        return g.genExpression(elem);
    });
    return `[${elems.join(', ')}]`;
}
