/**
 * Python Code Generator - DiscerneStatement
 *
 * Generate variant matching statement using match/case (Python 3.10+).
 *
 * TRANSFORMS:
 *   discerne event {
 *     si Click pro x, y { ... }
 *     si Quit { ... }
 *   }
 *   ->
 *   match event:
 *       case {'tag': 'Click', x, y}:
 *           ...
 *       case {'tag': 'Quit'}:
 *           ...
 *
 * WHY: Python match can destructure tagged unions using dict patterns.
 */

import type { DiscerneStatement, Identifier } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genDiscerneStatement(node: DiscerneStatement, g: PyGenerator): string {
    const lines: string[] = [];
    const discriminant = g.genExpression(node.discriminant);

    lines.push(`${g.ind()}match ${discriminant}:`);
    g.depth++;

    for (const caseNode of node.cases) {
        // Variant matching: si VariantName pro bindings { ... }
        const variantName = caseNode.variant.name;
        if (caseNode.bindings.length > 0) {
            const bindingNames = caseNode.bindings.map((b: Identifier) => b.name).join(', ');
            lines.push(`${g.ind()}case {'tag': '${variantName}', ${bindingNames}}:`);
        } else {
            lines.push(`${g.ind()}case {'tag': '${variantName}'}:`);
        }
        g.depth++;
        lines.push(g.genBlockStatementContent(caseNode.consequent));
        g.depth--;
    }

    g.depth--;

    return lines.join('\n');
}
