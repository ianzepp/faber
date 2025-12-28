/**
 * Rust Code Generator - DiscerneStatement
 *
 * TRANSFORMS:
 *   discerne event { quando Click { x }: ..., quando Quit: ... }
 *   -> match event { Click { x } => ..., Quit => ... }
 */

import type { DiscerneStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatementInline } from './functio';

export function genDiscerneStatement(node: DiscerneStatement, g: RsGenerator): string {
    const discriminant = g.genExpression(node.discriminant);
    const lines: string[] = [];

    lines.push(`${g.ind()}match ${discriminant} {`);
    g.depth++;

    for (const caseNode of node.cases) {
        const variantName = caseNode.variant.name;
        let pattern: string;

        if (caseNode.bindings.length > 0) {
            const bindings = caseNode.bindings.map(b => b.name).join(', ');
            pattern = `${variantName} { ${bindings} }`;
        } else {
            pattern = variantName;
        }

        const body = genBlockStatementInline(caseNode.consequent, g);
        lines.push(`${g.ind()}${pattern} => ${body},`);
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
