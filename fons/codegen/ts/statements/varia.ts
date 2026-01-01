/**
 * TypeScript Code Generator - VariaDeclaration
 *
 * TRANSFORMS:
 *   varia x: numerus = 5 -> let x: number = 5
 *   fixum y: textus = "hello" -> const y: string = "hello"
 *   fixum [a, b, c] = coords -> const [a, b, c] = coords
 *   figendum data = fetchData() -> const data = await fetchData()
 *   variandum result = fetch() -> let result = await fetch()
 *
 * WHY: Async bindings (figendum/variandum) imply await without explicit cede.
 *      The gerundive form ("that which will be fixed/varied") signals async intent.
 *
 * NOTE: Object destructuring now uses DestructureDeclaration with ex-prefix:
 *       ex persona fixum nomen, aetas -> const { nomen, aetas } = persona
 */

import type { VariaDeclaration } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { getVisibilityFromAnnotations } from '../../types';

export function genVariaDeclaration(node: VariaDeclaration, g: TsGenerator, semi: boolean): string {
    // Map kind to JS keyword and determine if async
    const isAsync = node.kind === 'figendum' || node.kind === 'variandum';
    const kind = node.kind === 'varia' || node.kind === 'variandum' ? 'let' : 'const';

    // Module-level: export when public
    const visibility = getVisibilityFromAnnotations(node.annotations);
    const exportMod = !g.inClass && visibility === 'public' ? 'export ' : '';

    let name: string;

    if (node.name.type === 'ArrayPattern') {
        // Generate array destructuring pattern with rest support
        // [a, b, ceteri rest] -> [a, b, ...rest]
        // [_, b, _] -> [, b, ]
        const elems = node.name.elements.map(elem => {
            if (elem.skip) {
                return ''; // skip position (underscore becomes empty slot)
            }
            if (elem.rest) {
                return `...${elem.name.name}`;
            }
            return elem.name.name;
        });

        name = `[${elems.join(', ')}]`;
    } else {
        name = node.name.name;
    }

    const typeAnno = node.typeAnnotation ? `: ${g.genType(node.typeAnnotation)}` : '';

    // Async bindings wrap initializer in await
    let init = '';
    if (node.init) {
        const expr = g.genExpression(node.init);
        init = isAsync ? ` = await ${expr}` : ` = ${expr}`;
    }

    return `${g.ind()}${exportMod}${kind} ${name}${typeAnno}${init}${semi ? ';' : ''}`;
}
