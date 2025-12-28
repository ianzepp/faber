/**
 * TypeScript Code Generator - ImportaDeclaration
 *
 * TRANSFORMS:
 *   ex "norma/tempus" importa nunc -> (no output, handled via intrinsics)
 *   ex "@hono/hono" importa Hono -> import { Hono } from "@hono/hono"
 *
 * WHY: norma/* imports are compiler-handled via intrinsics, not runtime imports.
 *      External packages pass through as native imports.
 */

import type { ImportaDeclaration } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genImportaDeclaration(node: ImportaDeclaration, g: TsGenerator, semi: boolean): string {
    const source = node.source;

    // Skip norma imports - these are handled via intrinsics
    if (source === 'norma' || source.startsWith('norma/')) {
        return '';
    }

    if (node.wildcard) {
        return `${g.ind()}import * as ${source} from "${source}"${semi ? ';' : ''}`;
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

    return `${g.ind()}import { ${names} } from "${source}"${semi ? ';' : ''}`;
}
