/**
 * Zig Code Generator - DestructureDeclaration
 *
 * TRANSFORMS:
 *   ex persona fixum nomen, aetas -> const _tmp = persona; const nomen = _tmp.nomen; const aetas = _tmp.aetas;
 *   ex persona fixum nomen ut n -> const _tmp = persona; const n = _tmp.nomen;
 *   ex persona fixum nomen, ceteri rest -> const _tmp = persona; const nomen = _tmp.nomen; // rest not directly supported
 *
 * TARGET: Zig doesn't have native object destructuring, so we expand to field access.
 *
 * NOTE: figendum/variandum async variants are not applicable in Zig since it doesn't
 *       have async/await like JS. We treat them the same as fixum/varia.
 */

import type { DestructureDeclaration } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genDestructureDeclaration(node: DestructureDeclaration, g: ZigGenerator): string {
    const kind = node.kind === 'varia' || node.kind === 'variandum' ? 'var' : 'const';
    const sourceExpr = g.genExpression(node.source);
    const lines: string[] = [];

    // Create a temp var to hold the object, then extract fields
    const tempVar = `_tmp`;
    lines.push(`${g.ind()}const ${tempVar} = ${sourceExpr};`);

    for (const spec of node.specifiers) {
        const localName = spec.local.name;
        const importedName = spec.imported.name;

        // WHY: rest patterns (ceteri) aren't directly supported in Zig
        // We'd need to iterate over remaining fields which isn't straightforward
        if (spec.rest) {
            lines.push(`${g.ind()}// ceteri (rest) not supported in Zig: ${localName}`);
            continue;
        }

        lines.push(`${g.ind()}${kind} ${localName} = ${tempVar}.${importedName};`);
    }

    return lines.join('\n');
}
