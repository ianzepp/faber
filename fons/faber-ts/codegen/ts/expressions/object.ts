/**
 * TypeScript Code Generator - Object Expression
 *
 * TRANSFORMS:
 *   { nomen: "Marcus" } -> { nomen: "Marcus" }
 *   { nomen: x, aetas: y } -> { nomen: x, aetas: y }
 *   { sparge defaults, x: 1 } -> { ...defaults, x: 1 }
 */

import type { ObjectExpression, Literal } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genLiteral } from './literal';

export function genObjectExpression(node: ObjectExpression, g: TsGenerator): string {
    if (node.properties.length === 0) {
        return '{}';
    }

    const props = node.properties.map(prop => {
        if (prop.type === 'SpreadElement') {
            return `...${g.genExpression(prop.argument)}`;
        }
        const key = prop.key.type === 'Identifier' ? prop.key.name : genLiteral(prop.key as Literal, g);
        const value = g.genExpression(prop.value);

        return `${key}: ${value}`;
    });

    return `{ ${props.join(', ')} }`;
}
