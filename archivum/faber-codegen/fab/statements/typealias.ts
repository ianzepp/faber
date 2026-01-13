/**
 * Faber Code Generator - TypeAliasDeclaration
 */

import type { TypeAliasDeclaration } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genTypeAliasDeclaration(node: TypeAliasDeclaration, g: FabGenerator): string {
    if (node.typeofTarget) {
        return `${g.ind()}typus ${node.name.name} = typus ${node.typeofTarget.name}`;
    }
    return `${g.ind()}typus ${node.name.name} = ${g.genType(node.typeAnnotation)}`;
}
