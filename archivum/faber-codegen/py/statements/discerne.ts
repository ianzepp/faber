/**
 * Python Code Generator - DiscerneStatement
 *
 * Generate variant matching statement using match/case (Python 3.10+).
 *
 * TRANSFORMS:
 *   discerne event {
 *     casu Click ut c { ... }
 *     casu Click pro x, y { ... }
 *     casu Quit { ... }
 *   }
 *   ->
 *   match event:
 *       case {'tag': 'Click'} as c:
 *           ...
 *       case {'tag': 'Click', 'x': x, 'y': y}:
 *           ...
 *       case {'tag': 'Quit'}:
 *           ...
 *
 * WHY: Python match can destructure tagged unions using dict patterns.
 *
 * NOTE: Multi-discriminant matching not yet supported in Python codegen.
 *       For now, only single-discriminant patterns are emitted.
 */

import type { DiscerneStatement, Identifier } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genDiscerneStatement(node: DiscerneStatement, g: PyGenerator): string {
    const lines: string[] = [];
    // Use first discriminant only (multi-discriminant not yet supported)
    const discriminant = g.genExpression(node.discriminants[0]!);

    lines.push(`${g.ind()}match ${discriminant}:`);
    g.depth++;

    for (const caseNode of node.cases) {
        // Use first pattern only
        const pattern = caseNode.patterns[0];
        if (!pattern) continue;

        // Handle wildcard
        if (pattern.isWildcard) {
            lines.push(`${g.ind()}case _:`);
            g.depth++;
            lines.push(g.genBlockStatementContent(caseNode.consequent));
            g.depth--;
            continue;
        }

        const variantName = pattern.variant.name;

        if (pattern.alias) {
            // Alias binding: casu Click ut c { ... }
            lines.push(`${g.ind()}case {'tag': '${variantName}'} as ${pattern.alias.name}:`);
        } else if (pattern.bindings.length > 0) {
            // Positional bindings: casu Click pro x, y { ... }
            const bindingNames = pattern.bindings.map((b: Identifier) => b.name).join(', ');
            lines.push(`${g.ind()}case {'tag': '${variantName}', ${bindingNames}}:`);
        } else {
            lines.push(`${g.ind()}case {'tag': '${variantName}'}:`);
        }

        g.depth++;
        lines.push(g.genBlockStatementContent(caseNode.consequent));
        g.depth--;
    }

    // Generate default case (ceterum)
    if (node.defaultCase) {
        lines.push(`${g.ind()}case _:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(node.defaultCase));
        g.depth--;
    }

    g.depth--;

    return lines.join('\n');
}
