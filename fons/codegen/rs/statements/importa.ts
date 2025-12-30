/**
 * Rust Code Generator - ImportaDeclaration
 *
 * TRANSFORMS:
 *   ex "std::collections" importa HashMap -> use std::collections::HashMap;
 *   ex "std::io" importa Read, Write -> use std::io::{Read, Write};
 *   ex "crate::utils" importa helper ut h -> use crate::utils::helper as h;
 */

import type { ImportaDeclaration } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genImportaDeclaration(node: ImportaDeclaration, g: RsGenerator): string {
    const source = node.source;

    // Skip norma imports - handled via intrinsics
    if (source === 'norma' || source.startsWith('norma/')) {
        return '';
    }

    if (node.wildcard) {
        const alias = node.wildcardAlias ? ` as ${node.wildcardAlias.name}` : '';
        return `${g.ind()}use ${source}::*${alias};`;
    }

    // WHY: ImportSpecifier has imported/local - emit "imported as local" when different
    if (node.specifiers.length === 1) {
        const s = node.specifiers[0]!;
        if (s.imported.name === s.local.name) {
            return `${g.ind()}use ${source}::${s.imported.name};`;
        }
        return `${g.ind()}use ${source}::${s.imported.name} as ${s.local.name};`;
    }

    // Multiple specifiers: use braces
    const names = node.specifiers
        .map(s => {
            if (s.imported.name === s.local.name) {
                return s.imported.name;
            }
            return `${s.imported.name} as ${s.local.name}`;
        })
        .join(', ');

    return `${g.ind()}use ${source}::{${names}};`;
}
