/**
 * TypeScript Code Generator - DiscretioDeclaration
 *
 * TRANSFORMS:
 *   discretio Event { Click { numerus x, numerus y }, Quit }
 *   -> type Event = { tag: 'Click'; x: number; y: number } | { tag: 'Quit' };
 *
 * WHY: TypeScript discriminated unions use a 'tag' property for narrowing.
 */

import type { DiscretioDeclaration } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { getVisibilityFromAnnotations } from '../../types';

export function genDiscretioDeclaration(node: DiscretioDeclaration, g: TsGenerator): string {
    const name = node.name.name;

    // Module-level: export when public
    const visibility = getVisibilityFromAnnotations(node.annotations);
    const exportMod = visibility === 'public' ? 'export ' : '';

    // Generate type parameters if present
    const typeParams = node.typeParameters && node.typeParameters.length > 0 ? '<' + node.typeParameters.map(p => p.name).join(', ') + '>' : '';

    // Generate each variant as its own named type.
    // WHY: Variants may reference sibling variants (or themselves). Emitting
    //      `Extract<Union, ...>` aliases creates circular dependencies like:
    //      Union -> VariantAlias -> Union.
    const variantTypes = node.variants.map(variant => {
        const variantName = variant.name.name;

        if (variant.fields.length === 0) {
            // Unit variant: just the tag
            return `${g.ind()}${exportMod}type ${variantName}${typeParams} = { tag: '${variantName}' };`;
        }

        // Variant with fields: tag + field properties
        const fields = variant.fields.map(field => {
            const fieldName = field.name.name;
            const fieldType = g.genType(field.fieldType);
            return `${fieldName}: ${fieldType}`;
        });

        return `${g.ind()}${exportMod}type ${variantName}${typeParams} = { tag: '${variantName}'; ${fields.join('; ')} };`;
    });

    // Union type built from the named variants.
    const unionMembers = node.variants.map(variant => `${variant.name.name}${typeParams}`);
    const unionType = `${g.ind()}${exportMod}type ${name}${typeParams} = ${unionMembers.join(' | ')};`;

    return [...variantTypes, unionType].join('\n');
}
