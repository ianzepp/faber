/**
 * Faber Code Generator - VariaDeclaration
 *
 * TRANSFORMS:
 *   VariaDeclaration -> kind type? name = init?
 */

import type { VariaDeclaration } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genVariaDeclaration(node: VariaDeclaration, g: FabGenerator): string {
    let name: string;

    if (node.name.type === 'ArrayPattern') {
        const elems = node.name.elements.map(elem => {
            if (elem.skip) {
                return '_';
            }
            if (elem.rest) {
                return `ceteri ${elem.name.name}`;
            }
            return elem.name.name;
        });
        name = `[${elems.join(', ')}]`;
    } else {
        name = node.name.name;
    }

    const typeAnno = node.typeAnnotation ? `${g.genType(node.typeAnnotation)} ` : '';
    const init = node.init ? ` = ${g.genExpression(node.init)}` : '';

    return `${g.ind()}${node.kind} ${typeAnno}${name}${init}`;
}
