/**
 * Faber Code Generator - MemberExpression
 */

import type { MemberExpression, Identifier } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genMemberExpression(node: MemberExpression, g: FabGenerator): string {
    const obj = g.genExpression(node.object);
    const objType = node.object.resolvedType;

    if (objType?.kind === 'namespace') {
        if (node.computed) {
            const prop = g.genExpression(node.property);
            const op = node.optional ? '?[' : node.nonNull ? '![' : '[';
            return `${obj}${op}${prop}]`;
        }

        const propName = (node.property as Identifier).name;
        const dot = node.optional ? '?.' : node.nonNull ? '!.' : '.';
        return `${obj}${dot}${propName}`;
    }

    const prop = g.genExpression(node.property);

    if (node.computed) {
        const op = node.optional ? '?[' : node.nonNull ? '![' : '[';
        return `${obj}${op}${prop}]`;
    }

    const dot = node.optional ? '?.' : node.nonNull ? '!.' : '.';
    return `${obj}${dot}${prop}`;
}
