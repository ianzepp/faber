/**
 * Python Code Generator - DiscretioDeclaration
 *
 * TRANSFORMS:
 *   discretio Event { Click { numerus x, numerus y }, Quit }
 *   -> @dataclass classes with discriminant property
 *
 * WHY: Python uses dataclasses with a 'tag' attribute for discriminated unions.
 *      TypedDict or Union types could also work, but dataclasses are more idiomatic.
 */

import type { DiscretioDeclaration } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genDiscretioDeclaration(node: DiscretioDeclaration, g: PyGenerator): string {
    g.features.dataclass = true;

    const lines: string[] = [];
    const baseName = node.name.name;

    // Generate a dataclass for each variant
    for (const variant of node.variants) {
        const variantName = `${baseName}_${variant.name.name}`;

        lines.push(`${g.ind()}@dataclass`);
        lines.push(`${g.ind()}class ${variantName}:`);
        g.depth++;

        lines.push(`${g.ind()}tag: str = '${variant.name.name}'`);

        if (variant.fields.length > 0) {
            for (const field of variant.fields) {
                const fieldName = field.name.name;
                const fieldType = g.genType(field.fieldType);
                lines.push(`${g.ind()}${fieldName}: ${fieldType}`);
            }
        }

        g.depth--;
        lines.push('');
    }

    // Generate union type alias
    const variantTypes = node.variants.map((v: (typeof node.variants)[0]) => `${baseName}_${v.name.name}`).join(' | ');
    lines.push(`${g.ind()}${baseName} = ${variantTypes}`);

    return lines.join('\n');
}
