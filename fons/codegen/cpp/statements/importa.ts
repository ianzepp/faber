/**
 * C++23 Code Generator - ImportaDeclaration
 *
 * TRANSFORMS:
 *   ex iostream importa * -> #include <iostream>
 *   ex "mylib" importa foo -> #include "mylib" (handled in includes)
 *
 * WHY: C++ uses #include, not import (until C++20 modules are widespread).
 *      For now, we just track that an import was requested.
 */

import type { ImportaDeclaration } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genImportaDeclaration(node: ImportaDeclaration, g: CppGenerator): string {
    // Add to includes set - will be rendered at top
    const source = node.source;

    // Check if it's a standard library or local
    if (source.startsWith('<') || !source.includes('/')) {
        g.includes.add(`<${source}>`);
    } else {
        g.includes.add(`"${source}"`);
    }

    return ''; // Actual include rendered at top
}
