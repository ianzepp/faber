/**
 * Faber Code Generator - DiscerneStatement (pattern matching)
 *
 * Roundtrip: AST -> Faber source code.
 * Supports both single and multi-discriminant matching.
 */

import type { DiscerneStatement, VariantPattern } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genDiscerneStatement(node: DiscerneStatement, g: FabGenerator): string {
    const lines: string[] = [];

    // Generate discriminants (comma-separated)
    const discriminants = node.discriminants.map((d) => g.genExpression(d)).join(', ');
    lines.push(`${g.ind()}discerne ${discriminants} {`);

    g.depth++;
    for (const c of node.cases) {
        // Generate patterns (comma-separated)
        const patterns = c.patterns.map((p) => genPattern(p)).join(', ');
        lines.push(`${g.ind()}casu ${patterns} ${genBlockStatement(c.consequent, g)}`);
    }

    // Generate default case (ceterum)
    if (node.defaultCase) {
        lines.push(`${g.ind()}ceterum ${genBlockStatement(node.defaultCase, g)}`);
    }
    g.depth--;

    lines.push(`${g.ind()}}`);
    return lines.join('\n');
}

/**
 * Generate a single pattern for output.
 */
function genPattern(p: VariantPattern): string {
    if (p.isWildcard) {
        return '_';
    }

    let result = p.variant.name;

    if (p.alias) {
        result += ` ut ${p.alias.name}`;
    } else if (p.bindings.length > 0) {
        const bindings = p.bindings.map((b) => b.name).join(', ');
        result += ` pro ${bindings}`;
    }

    return result;
}
