/**
 * Rust Code Generator - Novum Expression
 *
 * TRANSFORMS:
 *   novum Persona { nomen: "Bob" } -> Persona { nomen: "Bob" }
 *   novum Persona() -> Persona::new()
 */

import type { NovumExpression, ObjectProperty } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genLiteral } from './literal';

export function genNovumExpression(node: NovumExpression, g: RsGenerator): string {
    const callee = node.callee.name;

    if (node.withExpression && node.withExpression.type === 'ObjectExpression') {
        const obj = node.withExpression;
        const fields = obj.properties
            .filter((p): p is ObjectProperty => p.type === 'ObjectProperty')
            .map(p => {
                const key = p.key.type === 'Identifier' ? p.key.name : genLiteral(p.key, g);
                const value = g.genExpression(p.value);
                return `${key}: ${value}`;
            });
        return `${callee} { ${fields.join(', ')} }`;
    }

    return `${callee}::new()`;
}
