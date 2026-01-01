/**
 * Rust Code Generator - DiscerneStatement
 *
 * TRANSFORMS:
 *   discerne event { si Click ut c { ... } si Click pro x, y { ... } si Quit { ... } }
 *   -> match event { c @ Click { .. } => ..., Click { x, y } => ..., Quit => ... }
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

        if (caseNode.alias) {
            // Alias binding: si Click ut c { ... } -> c @ Click { .. }
            pattern = `${caseNode.alias.name} @ ${variantName} { .. }`;
        } else if (caseNode.bindings.length > 0) {
            // Positional bindings: si Click pro x, y { ... } -> Click { x, y }
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
