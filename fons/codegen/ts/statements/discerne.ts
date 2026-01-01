/**
 * TypeScript Code Generator - DiscerneStatement
 *
 * TRANSFORMS:
 *   discerne event { si Click ut c { use(c.x) } }
 *   -> if (event.tag === 'Click') { const c = event; use(c.x); }
 *
 *   discerne event { si Click pro x, y { use(x, y) } si Quit { exit() } }
 *   -> if (event.tag === 'Click') { const { x, y } = event; use(x, y); }
 *      else if (event.tag === 'Quit') { exit(); }
 *
 * WHY: TypeScript discriminated unions use a 'tag' property for discrimination.
 */

import type { DiscerneStatement, Identifier } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genDiscerneStatement(node: DiscerneStatement, g: TsGenerator): string {
    const discriminant = g.genExpression(node.discriminant);
    let result = '';

    // Generate if/else chain
    for (let i = 0; i < node.cases.length; i++) {
        const caseNode = node.cases[i]!;
        const keyword = i === 0 ? 'if' : 'else if';

        // Variant matching: si VariantName (ut alias | pro bindings)? { ... }
        const variantName = caseNode.variant.name;
        result += `${g.ind()}${keyword} (${discriminant}.tag === '${variantName}') {\n`;
        g.depth++;

        // Alias binding: si Click ut c { ... }
        if (caseNode.alias) {
            result += `${g.ind()}const ${caseNode.alias.name} = ${discriminant};\n`;
        }
        // Destructure bindings: si Click pro x, y { ... }
        else if (caseNode.bindings.length > 0) {
            const bindingNames = caseNode.bindings.map((b: Identifier) => b.name).join(', ');
            result += `${g.ind()}const { ${bindingNames} } = ${discriminant};\n`;
        }

        for (const stmt of caseNode.consequent.body) {
            result += g.genStatement(stmt) + '\n';
        }

        g.depth--;
        result += `${g.ind()}}`;

        // Add newline if more cases follow
        if (i < node.cases.length - 1) {
            result += '\n';
        }
    }

    return result;
}
