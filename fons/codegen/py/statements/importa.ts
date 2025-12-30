/**
 * Python Code Generator - ImportaDeclaration
 *
 * TRANSFORMS:
 *   ex norma importa * -> import norma
 *   ex norma importa scribe, lege -> from norma import scribe, lege
 *   ex norma importa scribe ut s -> from norma import scribe as s
 */

import type { ImportaDeclaration } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

/**
 * Generate import declaration.
 *
 * TRANSFORMS:
 *   ex norma importa * -> import norma
 *   ex norma importa scribe, lege -> from norma import scribe, lege
 *   ex norma importa scribe ut s -> from norma import scribe as s
 */
export function genImportaDeclaration(node: ImportaDeclaration, g: PyGenerator): string {
    const source = node.source;

    if (node.wildcard) {
        const alias = node.wildcardAlias ? ` as ${node.wildcardAlias.name}` : '';
        return `${g.ind()}import ${source}${alias}`;
    }

    // WHY: ImportSpecifier has imported/local - emit "imported as local" when different
    const names = node.specifiers
        .map(s => {
            if (s.imported.name === s.local.name) {
                return s.imported.name;
            }
            return `${s.imported.name} as ${s.local.name}`;
        })
        .join(', ');

    return `${g.ind()}from ${source} import ${names}`;
}
