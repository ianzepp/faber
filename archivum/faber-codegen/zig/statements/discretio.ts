/**
 * Zig Code Generator - DiscretioDeclaration (tagged union/enum)
 *
 * TRANSFORMS:
 *   discretio Event { Click { numerus x, numerus y }, Quit }
 *   -> const Event = union(enum) { click: struct { x: i64, y: i64 }, quit };
 *
 * TARGET: Zig has native tagged unions via union(enum).
 */

import type { DiscretioDeclaration } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genDiscretioDeclaration(node: DiscretioDeclaration, g: ZigGenerator): string {
    const name = node.name.name;
    const lines: string[] = [];

    lines.push(`${g.ind()}const ${name} = union(enum) {`);
    g.depth++;

    for (const variant of node.variants) {
        const variantName = variant.name.name.toLowerCase();

        if (variant.fields.length === 0) {
            // Unit variant
            lines.push(`${g.ind()}${variantName},`);
        } else {
            // Variant with payload - use anonymous struct
            const fields = variant.fields
                .map((field: (typeof variant.fields)[0]) => {
                    const fieldName = field.name.name;
                    const fieldType = g.genType(field.fieldType);
                    return `${fieldName}: ${fieldType}`;
                })
                .join(', ');

            lines.push(`${g.ind()}${variantName}: struct { ${fields} },`);
        }
    }

    g.depth--;
    lines.push(`${g.ind()}};`);

    return lines.join('\n');
}
