/**
 * C++23 Code Generator - DiscerneStatement
 *
 * TRANSFORMS:
 *   discerne event { si Click { ... } si Quit { ... } }
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
        lines.push(`${g.ind()}// si ${caseNode.variant.name}: { ... }`);
    }

    return lines.join('\n');
}
