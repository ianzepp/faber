/**
 * TypeScript Code Generator - TypeAliasDeclaration
 *
 * TRANSFORMS:
 *   typus ID = textus -> type ID = string;
 *   typus UserID = numerus<32, Naturalis> -> type UserID = number;
 *   typus ConfigTypus = typus config -> type ConfigTypus = typeof config;
 *
 * WHY: TypeScript type aliases provide semantic naming without runtime cost.
 *      The `typus x` form extracts the type of a value at compile time.
 */

import type { TypeAliasDeclaration } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { getVisibilityFromAnnotations } from '../../types';

export function genTypeAliasDeclaration(node: TypeAliasDeclaration, g: TsGenerator, semi: boolean): string {
    const name = node.name.name;

    // Module-level: export when public
    const visibility = getVisibilityFromAnnotations(node.annotations);
    const exportMod = visibility === 'public' ? 'export ' : '';

    // Check for typeof form: `typus X = typus y`
    if (node.typeofTarget) {
        const target = node.typeofTarget.name;
        return `${g.ind()}${exportMod}type ${name} = typeof ${target}${semi ? ';' : ''}`;
    }

    const typeAnno = g.genType(node.typeAnnotation);

    return `${g.ind()}${exportMod}type ${name} = ${typeAnno}${semi ? ';' : ''}`;
}
