/**
 * C++23 Code Generator - EligeStatement
 *
 * TRANSFORMS:
 *   elige x { si 1 { ... } si 2 { ... } secus { ... } }
 *   -> switch (x) { case 1: { ... break; } case 2: { ... break; } default: { ... break; } }
 */

import type { EligeStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genEligeStatement(node: EligeStatement, g: CppGenerator): string {
    const discriminant = g.genExpression(node.discriminant);
    const lines: string[] = [];

    lines.push(`${g.ind()}switch (${discriminant}) {`);

    for (const caseNode of node.cases) {
        // Value matching: si expression { ... }
        const test = g.genExpression(caseNode.test);

        lines.push(`${g.ind()}case ${test}: {`);
        g.depth++;

        for (const stmt of caseNode.consequent.body) {
            lines.push(g.genStatement(stmt));
        }

        lines.push(`${g.ind()}break;`);
        g.depth--;
        lines.push(`${g.ind()}}`);
    }

    if (node.defaultCase) {
        lines.push(`${g.ind()}default: {`);
        g.depth++;

        for (const stmt of node.defaultCase.body) {
            lines.push(g.genStatement(stmt));
        }

        lines.push(`${g.ind()}break;`);
        g.depth--;
        lines.push(`${g.ind()}}`);
    }

    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
