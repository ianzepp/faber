/**
 * C++23 Code Generator - DiscerneStatement
 *
 * TRANSFORMS:
 *   discerne event { casu Click ut c { ... } casu Click pro x, y { ... } casu Quit { ... } }
 *   -> TODO: std::visit pattern matching
 *
 * NOTE: C++ variant matching with std::visit is complex.
 *       For now, emit TODO placeholder.
 */

import type { DiscerneStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genDiscerneStatement(node: DiscerneStatement, g: CppGenerator): string {
    const lines: string[] = [];
    const discriminant = g.genExpression(node.discriminant);

    lines.push(`${g.ind()}// TODO: discerne on ${discriminant} - implement std::visit for C++`);

    for (const caseNode of node.cases) {
        if (caseNode.alias) {
            lines.push(`${g.ind()}// casu ${caseNode.variant.name} ut ${caseNode.alias.name}: { ... }`);
        } else if (caseNode.bindings.length > 0) {
            const bindings = caseNode.bindings.map(b => b.name).join(', ');
            lines.push(`${g.ind()}// casu ${caseNode.variant.name} pro ${bindings}: { ... }`);
        } else {
            lines.push(`${g.ind()}// casu ${caseNode.variant.name}: { ... }`);
        }
    }

    return lines.join('\n');
}
