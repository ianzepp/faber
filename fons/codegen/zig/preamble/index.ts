/**
 * Zig Preamble Generator
 *
 * Generates preamble based on features used, including arena allocator setup
 * and collection type definitions.
 */

import { readFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import type { RequiredFeatures } from '../../types';

const __dirname = dirname(fileURLToPath(import.meta.url));

// WHY: subsidia/ contains actual Zig library files, not codegen logic.
// Path from fons/codegen/zig/preamble/ to subsidia/zig/
const SUBSIDIA_PATH = join(__dirname, '../../../../subsidia/zig');

// Read preamble files once at module load
const LISTA = readFileSync(join(SUBSIDIA_PATH, 'lista.zig'), 'utf-8');
const TABULA = readFileSync(join(SUBSIDIA_PATH, 'tabula.zig'), 'utf-8');

/**
 * Generate preamble based on features used.
 *
 * @param features - Feature flags set during codegen traversal
 * @returns Preamble string
 */
export function genPreamble(features: RequiredFeatures): string {
    const lines: string[] = [];

    // Always import std
    lines.push('const std = @import("std");');

    // Add Lista type when lista collections are used
    if (features.lista) {
        lines.push('');
        lines.push(LISTA);
    }

    // Add Tabula type when tabula collections are used
    if (features.tabula) {
        lines.push('');
        lines.push(TABULA);
    }

    return lines.join('\n') + '\n';
}

/**
 * Check if collections are used (needs arena allocator in main).
 */
export function usesCollections(features: RequiredFeatures): boolean {
    return features.lista || features.tabula || features.copia;
}
