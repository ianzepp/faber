/**
 * TypeScript Code Generator - DestructureDeclaration
 *
 * TRANSFORMS:
 *   ex persona fixum nomen, aetas -> const { nomen, aetas } = persona
 *   ex persona fixum nomen ut n -> const { nomen: n } = persona
 *   ex persona fixum nomen, ceteri rest -> const { nomen, ...rest } = persona
 *   ex promise figendum result -> const { result } = await promise
 *
 * WHY: Brace-less syntax in Faber maps to brace destructuring in JS.
 *      'ut' aliases map to colon renaming in JS destructuring.
 */

import type { DestructureDeclaration } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genDestructureDeclaration(node: DestructureDeclaration, g: TsGenerator, semi: boolean): string {
    const isAsync = node.kind === 'figendum' || node.kind === 'variandum';
    const kind = node.kind === 'varia' || node.kind === 'variandum' ? 'let' : 'const';

    // Generate destructuring pattern from specifiers
    const props = node.specifiers.map(spec => {
        // Rest pattern: ceteri rest -> ...rest
        if (spec.rest) {
            return `...${spec.local.name}`;
        }
        // Alias: nomen ut n -> nomen: n
        if (spec.imported.name !== spec.local.name) {
            return `${spec.imported.name}: ${spec.local.name}`;
        }
        // Same name
        return spec.imported.name;
    });

    const pattern = `{ ${props.join(', ')} }`;
    const source = isAsync ? `await ${g.genExpression(node.source)}` : g.genExpression(node.source);

    return `${g.ind()}${kind} ${pattern} = ${source}${semi ? ';' : ''}`;
}
