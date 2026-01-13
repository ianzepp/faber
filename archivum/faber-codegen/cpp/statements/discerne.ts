/**
 * C++23 Code Generator - DiscerneStatement
 *
 * TRANSFORMS:
 *   discerne event { casu Click ut c { ... } casu Click pro x, y { ... } casu Quit { ... } }
 *   -> TODO: std::visit pattern matching
 *
 * NOTE: C++ variant matching with std::visit is complex.
 *       For now, emit TODO placeholder.
 *
 * NOTE: Multi-discriminant matching not yet supported in C++ codegen.
 */

import type { DiscerneStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genDiscerneStatement(node: DiscerneStatement, g: CppGenerator): string {
    const lines: string[] = [];
    // Use first discriminant only (multi-discriminant not yet supported)
    const discriminant = g.genExpression(node.discriminants[0]!);

    lines.push(`${g.ind()}// TODO: discerne on ${discriminant} - implement std::visit for C++`);

    for (const caseNode of node.cases) {
        // Use first pattern only
        const pattern = caseNode.patterns[0];
        if (!pattern) continue;

        if (pattern.isWildcard) {
            lines.push(`${g.ind()}// casu _: { ... }`);
        } else if (pattern.alias) {
            lines.push(`${g.ind()}// casu ${pattern.variant.name} ut ${pattern.alias.name}: { ... }`);
        } else if (pattern.bindings.length > 0) {
            const bindings = pattern.bindings.map((b) => b.name).join(', ');
            lines.push(`${g.ind()}// casu ${pattern.variant.name} pro ${bindings}: { ... }`);
        } else {
            lines.push(`${g.ind()}// casu ${pattern.variant.name}: { ... }`);
        }
    }

    // Generate default case (ceterum) comment
    if (node.defaultCase) {
        lines.push(`${g.ind()}// ceterum: { ... }`);
    }

    return lines.join('\n');
}
