/**
 * C++23 Code Generator - TypeAliasDeclaration
 *
 * TRANSFORMS:
 *   typus ID = textus -> using ID = std::string;
 */

import type { TypeAliasDeclaration } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genTypeAliasDeclaration(node: TypeAliasDeclaration, g: CppGenerator): string {
    const name = node.name.name;
    const type = g.genType(node.typeAnnotation);

    return `${g.ind()}using ${name} = ${type};`;
}
