/**
 * Faber Code Generator - ImportaDeclaration
 *
 * TRANSFORMS:
 *   ImportaDeclaration -> ex source importa specifiers
 */

import type { ImportaDeclaration } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genImportaDeclaration(node: ImportaDeclaration, g: FabGenerator): string {
    const source = node.source.includes('/') ? `"${node.source}"` : node.source;

    if (node.wildcard) {
        const alias = node.wildcardAlias ? ` ut ${node.wildcardAlias.name}` : '';
        return `${g.ind()}ex ${source} importa *${alias}`;
    }

    const specifiers = node.specifiers.map(spec => {
        if (spec.imported.name === spec.local.name) {
            return spec.imported.name;
        }
        return `${spec.imported.name} ut ${spec.local.name}`;
    });

    return `${g.ind()}ex ${source} importa ${specifiers.join(', ')}`;
}
