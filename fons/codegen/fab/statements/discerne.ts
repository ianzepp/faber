/**
 * Faber Code Generator - DiscerneStatement (pattern matching)
 */

import type { DiscerneStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genDiscerneStatement(node: DiscerneStatement, g: FabGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}discerne ${g.genExpression(node.discriminant)} {`);

    g.depth++;
    for (const c of node.cases) {
        if (c.alias) {
            // Alias binding: casu Click ut c { ... }
            lines.push(`${g.ind()}casu ${c.variant.name} ut ${c.alias.name} ${genBlockStatement(c.consequent, g)}`);
        } else if (c.bindings.length > 0) {
            // Positional bindings: casu Click pro x, y { ... }
            const bindings = c.bindings.map(b => b.name).join(', ');
            lines.push(`${g.ind()}casu ${c.variant.name} pro ${bindings} ${genBlockStatement(c.consequent, g)}`);
        } else {
            lines.push(`${g.ind()}casu ${c.variant.name} ${genBlockStatement(c.consequent, g)}`);
        }
    }
    g.depth--;

    lines.push(`${g.ind()}}`);
    return lines.join('\n');
}
