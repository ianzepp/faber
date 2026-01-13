/**
 * C++23 Code Generator - DiscretioDeclaration
 *
 * TRANSFORMS:
 *   discretio Event { Click { numerus x, numerus y }, Quit }
 *   -> struct Click { int64_t x; int64_t y; };
 *      struct Quit {};
 *      using Event = std::variant<Click, Quit>;
 *
 * WHY: std::variant is C++17's type-safe union. Each variant becomes
 *      a struct, and we use std::variant to hold any of them.
 */

import type { DiscretioDeclaration } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genDiscretioDeclaration(node: DiscretioDeclaration, g: CppGenerator): string {
    g.includes.add('<variant>');

    const name = node.name.name;
    const lines: string[] = [];
    const variantNames: string[] = [];

    // Generate a struct for each variant
    for (const variant of node.variants) {
        const variantName = variant.name.name;
        variantNames.push(variantName);

        if (variant.fields.length === 0) {
            // Unit variant: empty struct
            lines.push(`${g.ind()}struct ${variantName} {};`);
        } else {
            // Variant with fields
            lines.push(`${g.ind()}struct ${variantName} {`);
            g.depth++;

            for (const field of variant.fields) {
                const fieldName = field.name.name;
                const fieldType = g.genType(field.fieldType);
                lines.push(`${g.ind()}${fieldType} ${fieldName};`);
            }

            g.depth--;
            lines.push(`${g.ind()}};`);
        }
    }

    // Generate the variant type alias
    lines.push(`${g.ind()}using ${name} = std::variant<${variantNames.join(', ')}>;`);

    return lines.join('\n');
}
