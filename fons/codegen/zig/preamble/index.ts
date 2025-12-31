/**
 * Zig Preamble Generator
 *
 * Generates import statements for Zig output. The actual stdlib implementations
 * live in subsidia/zig/ and must be available as a module named "faber".
 *
 * WHY: Zig expects proper module imports, not inlined code. This approach:
 * - Keeps generated files small and readable
 * - Allows the stdlib to be compiled once and reused
 * - Follows Zig's module system conventions
 */

import type { RequiredFeatures } from '../../types';

/**
 * Generate preamble based on features used.
 *
 * @param features - Feature flags set during codegen traversal
 * @returns Preamble string with import statements
 */
export function genPreamble(features: RequiredFeatures): string {
    const lines: string[] = [];

    // Always import std
    lines.push('const std = @import("std");');

    // Import faber stdlib if any collections are used
    if (features.lista || features.tabula || features.copia) {
        lines.push('const faber = @import("faber");');

        // Re-export specific types for convenience
        if (features.lista) {
            lines.push('const Lista = faber.Lista;');
        }
        if (features.tabula) {
            lines.push('const Tabula = faber.Tabula;');
        }
        if (features.copia) {
            lines.push('const Copia = faber.Copia;');
        }
    }

    // Set up I/O streams if used
    if (features.stdout) {
        lines.push('const stdout = std.io.getStdOut().writer();');
    }
    if (features.stderr) {
        lines.push('const stderr = std.io.getStdErr().writer();');
    }
    if (features.stdin) {
        lines.push('const stdin = std.io.getStdIn().reader();');
    }

    return lines.join('\n') + '\n';
}

/**
 * Check if collections are used (needs arena allocator in main).
 */
export function usesCollections(features: RequiredFeatures): boolean {
    return features.lista || features.tabula || features.copia;
}
