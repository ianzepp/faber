/**
 * Python Code Generator - TypeAliasDeclaration
 *
 * TRANSFORMS:
 *   typus ID = textus -> ID = str  (or TypeAlias)
 */

import type { TypeAliasDeclaration } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genTypeAliasDeclaration(node: TypeAliasDeclaration, g: PyGenerator): string {
    const name = node.name.name;
    const typeAnno = g.genType(node.typeAnnotation);
    return `${g.ind()}${name} = ${typeAnno}`;
}
