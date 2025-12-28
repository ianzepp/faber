/**
 * Rust Code Generator - Object Expression
 *
 * TRANSFORMS:
 *   { x: 1, y: 2 } -> { x: 1, y: 2 }
 *   { ...other } -> { ..other }
 */

import type { ObjectExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genLiteral } from './literal';

export function genObjectExpression(node: ObjectExpression, g: RsGenerator): string {
    if (node.properties.length === 0) {
        return '{}';
    }

    const props = node.properties.map(prop => {
        if (prop.type === 'SpreadElement') {
            return `..${g.genExpression(prop.argument)}`;
        }
        const key = prop.key.type === 'Identifier' ? prop.key.name : genLiteral(prop.key, g);
        const value = g.genExpression(prop.value);
        return `${key}: ${value}`;
    });

    return `{ ${props.join(', ')} }`;
}
