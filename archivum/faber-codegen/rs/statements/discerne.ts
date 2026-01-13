/**
 * Rust Code Generator - DiscerneStatement
 *
 * TRANSFORMS:
 *   discerne event { casu Click ut c { ... } casu Click pro x, y { ... } casu Quit { ... } }
 *   -> match event { c @ Click { .. } => ..., Click { x, y } => ..., Quit => ... }
 *
 * NOTE: Multi-discriminant matching not yet supported in Rust codegen.
 *       For now, only single-discriminant patterns are emitted.
 */

import type { DiscerneStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatementInline } from './functio';

export function genDiscerneStatement(node: DiscerneStatement, g: RsGenerator): string {
    // Use first discriminant only (multi-discriminant not yet supported)
    const discriminant = g.genExpression(node.discriminants[0]!);
    const lines: string[] = [];

    lines.push(`${g.ind()}match ${discriminant} {`);
    g.depth++;

    for (const caseNode of node.cases) {
        // Use first pattern only
        const pattern = caseNode.patterns[0];
        if (!pattern) continue;

        let patternStr: string;

        // Handle wildcard
        if (pattern.isWildcard) {
            patternStr = '_';
        } else if (pattern.alias) {
            // Alias binding: casu Click ut c { ... } -> c @ Click { .. }
            patternStr = `${pattern.alias.name} @ ${pattern.variant.name} { .. }`;
        } else if (pattern.bindings.length > 0) {
            // Positional bindings: casu Click pro x, y { ... } -> Click { x, y }
            const bindings = pattern.bindings.map((b) => b.name).join(', ');
            patternStr = `${pattern.variant.name} { ${bindings} }`;
        } else {
            patternStr = pattern.variant.name;
        }

        const body = genBlockStatementInline(caseNode.consequent, g);
        lines.push(`${g.ind()}${patternStr} => ${body},`);
    }

    // Generate default case (ceterum) as _ =>
    if (node.defaultCase) {
        const body = genBlockStatementInline(node.defaultCase, g);
        lines.push(`${g.ind()}_ => ${body},`);
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
