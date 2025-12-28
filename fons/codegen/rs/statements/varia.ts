/**
 * Rust Code Generator - VariaDeclaration
 *
 * TRANSFORMS:
 *   varia x: numerus = 5 -> let mut x: f64 = 5;
 *   fixum y: textus = "hello" -> let y: String = "hello";
 *   fixum [a, b, c] = coords -> let [a, b, c] = coords;
 *   figendum data = fetch() -> let data = fetch().await;
 *
 * NOTE: Object destructuring uses DestructureDeclaration with ex-prefix.
 */

import type { VariaDeclaration } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genVariaDeclaration(node: VariaDeclaration, g: RsGenerator): string {
    const isAsync = node.kind === 'figendum' || node.kind === 'variandum';
    const mutKeyword = node.kind === 'varia' || node.kind === 'variandum' ? 'mut ' : '';

    let name: string;

    if (node.name.type === 'ArrayPattern') {
        // Generate array destructuring pattern
        const elems = node.name.elements.map(elem => {
            if (elem.skip) {
                return '_';
            }
            if (elem.rest) {
                return `${elem.name.name}@..`;
            }
            return elem.name.name;
        });
        name = `[${elems.join(', ')}]`;
    } else {
        name = node.name.name;
    }

    const typeAnno = node.typeAnnotation ? `: ${g.genType(node.typeAnnotation)}` : '';

    let init = '';
    if (node.init) {
        const expr = g.genExpression(node.init);
        init = isAsync ? ` = ${expr}.await` : ` = ${expr}`;
    }

    return `${g.ind()}let ${mutKeyword}${name}${typeAnno}${init};`;
}
