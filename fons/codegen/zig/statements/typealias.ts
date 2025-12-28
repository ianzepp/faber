/**
 * Zig Code Generator - TypeAliasDeclaration
 *
 * TRANSFORMS:
 *   typus ID = textus -> const ID = []const u8;
 *
 * TARGET: Zig uses const for type aliases.
 */

import type { TypeAliasDeclaration } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genTypeAliasDeclaration(node: TypeAliasDeclaration, g: ZigGenerator): string {
    const name = node.name.name;
    const typeAnno = g.genType(node.typeAnnotation);

    return `${g.ind()}const ${name} = ${typeAnno};`;
}
