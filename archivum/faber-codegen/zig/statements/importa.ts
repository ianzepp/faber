/**
 * Zig Code Generator - ImportaDeclaration
 *
 * TRANSFORMS:
 *   ex norma importa * -> const norma = @import("norma");
 *   ex norma importa scribe, lege -> const _norma = @import("norma");
 *                                     const scribe = _norma.scribe;
 *                                     const lege = _norma.lege;
 *   ex norma importa scribe ut s -> const _norma = @import("norma");
 *                                    const s = _norma.scribe;
 *
 * TARGET: Zig uses @import() builtin. Specific imports create const bindings.
 */

import type { ImportaDeclaration } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genImportaDeclaration(node: ImportaDeclaration, g: ZigGenerator): string {
    const source = node.source;

    if (node.wildcard) {
        // WHY: Use explicit alias if provided, otherwise use source as name
        const alias = node.wildcardAlias?.name ?? source;
        return `${g.ind()}const ${alias} = @import("${source}");`;
    }

    // WHY: Import module once with underscore prefix, then bind specific names
    const lines: string[] = [];
    const modVar = `_${source}`;

    lines.push(`${g.ind()}const ${modVar} = @import("${source}");`);
    for (const spec of node.specifiers) {
        // WHY: ImportSpecifier has imported/local - use local for binding, imported for access
        const localName = spec.local.name;
        const importedName = spec.imported.name;
        lines.push(`${g.ind()}const ${localName} = ${modVar}.${importedName};`);
    }

    return lines.join('\n');
}
