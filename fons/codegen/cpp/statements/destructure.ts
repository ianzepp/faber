/**
 * C++23 Code Generator - DestructureDeclaration
 *
 * TRANSFORMS:
 *   ex persona fixum nomen, aetas -> const auto& _tmp = persona; const auto& nomen = _tmp.nomen; ...
 *   ex persona fixum nomen ut n -> const auto& n = _tmp.nomen;
 *   ex promise figendum result -> const auto& _tmp = promise; const auto& result = _tmp.result;
 *
 * WHY: C++ doesn't have native object destructuring. We expand to
 *      temporary variable + field access for each specifier.
 */

import type { DestructureDeclaration } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genDestructureDeclaration(node: DestructureDeclaration, g: CppGenerator): string {
    const lines: string[] = [];
    const isConst = node.kind === 'fixum' || node.kind === 'figendum';
    const constPrefix = isConst ? 'const ' : '';

    // Generate source expression (ignore async - C++ handles differently)
    const sourceExpr = g.genExpression(node.source);

    // Create temp variable to hold the source
    lines.push(`${g.ind()}${constPrefix}auto& _tmp = ${sourceExpr};`);

    // Extract each property
    for (const spec of node.specifiers) {
        const importedName = spec.imported.name;
        const localName = spec.local.name;

        if (spec.rest) {
            // Rest pattern: not directly supported in C++, emit TODO
            lines.push(`${g.ind()}// TODO: rest pattern for ${localName}`);
        } else {
            // Regular property: extract from temp
            lines.push(`${g.ind()}${constPrefix}auto& ${localName} = _tmp.${importedName};`);
        }
    }

    return lines.join('\n');
}
