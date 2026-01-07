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

    // Set up I/O helper functions if used
    // WHY: Zig 0.15 "Writergate" changed the I/O API - can't have module-level writers
    if (features.stdout) {
        lines.push('');
        lines.push('fn print(comptime fmt: []const u8, args: anytype) void {');
        lines.push('    var w = std.fs.File.stdout().writer(&.{});');
        lines.push('    w.interface.print(fmt, args) catch {};');
        lines.push('}');
    }
    if (features.stderr) {
        lines.push('');
        lines.push('fn eprint(comptime fmt: []const u8, args: anytype) void {');
        lines.push('    var w = std.fs.File.stderr().writer(&.{});');
        lines.push('    w.interface.print(fmt, args) catch {};');
        lines.push('}');
    }
    if (features.stdin) {
        lines.push('const stdin = std.fs.File.stdin();');
    }

    return lines.join('\n') + '\n';
}

/**
 * Check if collections are used (needs arena allocator in main).
 */
export function usesCollections(features: RequiredFeatures): boolean {
    return features.lista || features.tabula || features.copia;
}
