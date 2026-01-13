/**
 * Rust Code Generator - TypeAliasDeclaration
 *
 * TRANSFORMS:
 *   typus UserId = textus -> type UserId = String;
 */

import type { TypeAliasDeclaration } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genTypeAliasDeclaration(node: TypeAliasDeclaration, g: RsGenerator): string {
    const name = node.name.name;
    const typeAnno = g.genType(node.typeAnnotation);

    return `${g.ind()}type ${name} = ${typeAnno};`;
}
