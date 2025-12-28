/**
 * Python Code Generator - DestructureDeclaration
 *
 * TRANSFORMS:
 *   ex persona fixum nomen, aetas -> nomen, aetas = persona["nomen"], persona["aetas"]
 *   ex persona fixum nomen ut n -> n = persona["nomen"]
 *   ex persona fixum nomen, ceteri rest -> nomen = persona["nomen"]; rest = {k: v for ...}
 *   ex promise figendum result -> result = (await promise)["result"]
 *
 * WHY: Python lacks native object destructuring, so we emit multiple assignments.
 *      For rest patterns (ceteri), we use dict comprehension to collect remaining keys.
 */

import type { DestructureDeclaration } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genDestructureDeclaration(node: DestructureDeclaration, g: PyGenerator): string {
    const isAsync = node.kind === 'figendum' || node.kind === 'variandum';
    const awaitPrefix = isAsync ? 'await ' : '';
    const sourceExpr = g.genExpression(node.source);
    // WHY: Wrap in parens for async to ensure await applies to source, not index
    const source = isAsync ? `(${awaitPrefix}${sourceExpr})` : sourceExpr;

    const lines: string[] = [];

    // Separate regular specifiers from rest specifier
    const regularSpecs = node.specifiers.filter(s => !s.rest);
    const restSpec = node.specifiers.find(s => s.rest);

    // Generate regular property extractions
    for (const spec of regularSpecs) {
        const imported = spec.imported.name;
        const local = spec.local.name;
        lines.push(`${g.ind()}${local} = ${source}["${imported}"]`);
    }

    // Generate rest collection (remaining properties after extracting named ones)
    // WHY: Python doesn't have native rest destructuring, so we manually collect
    //      the remaining keys using dict comprehension
    if (restSpec) {
        const restName = restSpec.local.name;
        const excludeKeys = regularSpecs.map(s => `"${s.imported.name}"`).join(', ');
        lines.push(`${g.ind()}${restName} = {k: v for k, v in ${source}.items() if k not in [${excludeKeys}]}`);
    }

    return lines.join('\n');
}
