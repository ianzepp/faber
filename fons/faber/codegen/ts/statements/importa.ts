/**
 * TypeScript Code Generator - ImportaDeclaration
 *
 * TRANSFORMS:
 *   ex "norma/tempus" importa nunc -> (no output, handled via intrinsics)
 *   ex "@hono/hono" importa Hono -> import { Hono } from "@hono/hono"
 *   ex "../../norma/hal/consolum" importa consolum -> import { consolum } from "../../norma/hal/codegen/ts/consolum"
 *
 * WHY: norma/* imports are compiler-handled via intrinsics, not runtime imports.
 *      External packages pass through as native imports.
 *      HAL imports with @subsidia are rewritten to point to native implementations.
 */

import type { ImportaDeclaration } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { dirname, join, resolve } from 'node:path';

export function genImportaDeclaration(node: ImportaDeclaration, g: TsGenerator, semi: boolean): string {
    let source = node.source;

    // Check if this import has a @subsidia mapping (HAL import)
    // WHY: HAL pactums declare native implementations via @subsidia annotation.
    //      We need to rewrite the import to point to the native implementation.
    const targetMappings = g.subsidiaImports.get(source);
    if (targetMappings) {
        const subsidiaPath = targetMappings.get('ts');
        if (subsidiaPath) {
            // WHY: Subsidia path is relative to the declaring .fab file.
            //      When compiling to a temp file (faber run), we need absolute paths.
            // Example: "../../norma/hal/consolum" + "codegen/ts/consolum.ts"
            //       -> "/abs/path/to/fons/norma/hal/codegen/ts/consolum"
            if (g.sourceFilePath) {
                // Resolve the import source relative to the source file
                const sourceDir = dirname(g.sourceFilePath);
                const importedFileDir = resolve(sourceDir, dirname(source));
                const absolutePath = join(importedFileDir, subsidiaPath);
                // Remove .ts extension (TypeScript imports don't need it)
                source = absolutePath.replace(/\.ts$/, '');
            }
            else {
                // Fallback: relative path (may not work for temp file execution)
                const sourceDir = dirname(source);
                const fullPath = join(sourceDir, subsidiaPath);
                source = fullPath.replace(/\.ts$/, '');
            }
        }
    }

    // Skip norma imports - these are handled via intrinsics
    if (source === 'norma' || source.startsWith('norma/')) {
        return '';
    }

    if (node.wildcard) {
        // WHY: Pass through literally. If TS requires an alias and none provided,
        // the TS compiler will error - that's the developer's responsibility.
        const alias = node.wildcardAlias?.name ?? '';
        const asClause = alias ? ` as ${alias}` : '';
        return `${g.ind()}import *${asClause} from "${source}"${semi ? ';' : ''}`;
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
