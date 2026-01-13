/**
 * Faber Code Generator - ObjectExpression
 */

import type { ObjectExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genObjectExpression(node: ObjectExpression, g: FabGenerator): string {
    if (node.properties.length === 0) {
        return '{}';
    }

    const props = node.properties.map(prop => {
        if (prop.type === 'SpreadElement') {
            return `sparge ${g.genExpression(prop.argument)}`;
        }

        const key = prop.key.type === 'Literal' ? prop.key.raw : prop.key.name;
        return `${key}: ${g.genExpression(prop.value)}`;
    });

    return `{ ${props.join(', ')} }`;
}
