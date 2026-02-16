/**
 * TypeScript Code Generator - ImportaDeclaration
 *
 * TRANSFORMS:
 *   ex "norma/tempus" importa nunc -> (no output, handled via intrinsics)
 *   ex "@hono/hono" importa Hono -> import { Hono } from "@hono/hono"
 *   ex "../../norma/hal/consolum" importa consolum -> handled based on keepRelativeImports
 *
 * WHY: norma/* imports are compiler-handled via intrinsics, not runtime imports.
 *      External packages pass through as native imports.
 *
 * When keepRelativeImports=true (build command):
 *   - Relative imports stay relative
 *   - HAL/subsidia imports stay relative (build script copies implementations)
 *
 * When keepRelativeImports=false (faber run):
 *   - Relative imports become absolute (temp file needs absolute resolution)
 *   - HAL/subsidia imports get codegen/ts/ path and become absolute
 */

import type { ImportaDeclaration } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { dirname, join, resolve } from 'node:path';

export function genImportaDeclaration(node: ImportaDeclaration, g: TsGenerator, semi: boolean): string {
    let source = node.source;

    // Skip norma imports first - these are handled via intrinsics
    if (source === 'norma' || source.startsWith('norma/')) {
        return '';
    }

    // When keepRelativeImports=true, don't transform any paths
    // Build scripts are responsible for copying dependencies to the right locations
    if (g.keepRelativeImports) {
        // Pass through as-is
    }
    else {
        // Check if this import has a @subsidia mapping (HAL import)
        // WHY: HAL pactums declare native implementations via @subsidia annotation.
        //      For temp file execution (faber run), we need absolute paths to codegen/ts/.
        const targetMappings = g.subsidiaImports.get(source);
        if (targetMappings) {
            const subsidiaPath = targetMappings.get('ts');
            if (subsidiaPath) {
                // WHY: Subsidia path is relative to the declaring .fab file.
                //      When compiling to a temp file (faber run), we need absolute paths.
                // Example: "../../norma/hal/consolum" + "codegen/ts/consolum.ts"
                //       -> "/abs/path/to/fons/norma/hal/codegen/ts/consolum"
                if (g.sourceFilePath) {
                    const sourceDir = dirname(g.sourceFilePath);
                    const importedFileDir = resolve(sourceDir, dirname(source));
                    const absolutePath = join(importedFileDir, subsidiaPath);
                    source = absolutePath.replace(/\.ts$/, '');
                }
                else {
                    const sourceDir = dirname(source);
                    const fullPath = join(sourceDir, subsidiaPath);
                    source = fullPath.replace(/\.ts$/, '');
                }
            }
        }

        // WHY: Relative imports need absolutizing when compiling to a temp file (faber run).
        // The temp file is in /tmp, so ./foo won't resolve relative to the source location.
        if (g.sourceFilePath && (source.startsWith('./') || source.startsWith('../'))) {
            const sourceDir = dirname(g.sourceFilePath);
            source = resolve(sourceDir, source);
        }
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
