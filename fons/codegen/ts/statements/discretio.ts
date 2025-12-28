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

export function genDiscretioDeclaration(node: DiscretioDeclaration, g: TsGenerator): string {
    const name = node.name.name;

    // Generate type parameters if present
    let typeParams = '';
    if (node.typeParameters && node.typeParameters.length > 0) {
        typeParams = '<' + node.typeParameters.map(p => p.name).join(', ') + '>';
    }

    // Generate variant types
    const variants = node.variants.map(variant => {
        const variantName = variant.name.name;

        if (variant.fields.length === 0) {
            // Unit variant: just the tag
            return `{ tag: '${variantName}' }`;
        }

        // Variant with fields: tag + field properties
        const fields = variant.fields.map(field => {
            const fieldName = field.name.name;
            const fieldType = g.genType(field.fieldType);
            return `${fieldName}: ${fieldType}`;
        });

        return `{ tag: '${variantName}'; ${fields.join('; ')} }`;
    });

    return `${g.ind()}type ${name}${typeParams} = ${variants.join(' | ')};`;
}
