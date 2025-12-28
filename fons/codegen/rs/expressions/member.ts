/**
 * Rust Code Generator - Member Expression
 *
 * TRANSFORMS:
 *   obj.prop -> obj.prop
 *   obj[idx] -> obj[idx]
 *   obj?.prop -> obj.as_ref().map(|x| x.prop)
 *   obj?[idx] -> obj.get(idx)
 */

import type { MemberExpression, Identifier } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genMemberExpression(node: MemberExpression, g: RsGenerator): string {
    const obj = g.genExpression(node.object);

    if (node.computed) {
        // WHY: Use genBareExpression to avoid unnecessary parens around index
        const prop = g.genBareExpression(node.property);
        if (node.optional) {
            return `${obj}.get(${prop})`;
        }
        return `${obj}[${prop}]`;
    }

    const prop = (node.property as Identifier).name;

    if (node.optional) {
        return `${obj}.as_ref().map(|x| x.${prop})`;
    }

    return `${obj}.${prop}`;
}
