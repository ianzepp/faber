/**
 * Zig Code Generator - Lege Expression (read stdin)
 *
 * TRANSFORMS:
 *   lege        -> stdin.readAllAlloc(alloc, 10 * 1024 * 1024) catch ""
 *   lege lineam -> stdin.readUntilDelimiter('\n', buf) or readLine helper
 *
 * TARGET: Zig's std.io.getStdIn().reader() for stdin input.
 *
 * WHY: Bootstrap compiler needs to read source from stdin.
 *      Returns empty string on error for simplicity.
 */

import type { LegeExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genLegeExpression(node: LegeExpression, g: ZigGenerator): string {
    g.features.stdin = true;
    const curator = g.getCurator();

    if (node.mode === 'line') {
        // WHY: readUntilDelimiterAlloc reads until newline, allocating the result
        // Returns null on EOF, so we use orelse "" for empty fallback
        return `(stdin.readUntilDelimiterAlloc(${curator}, '\\n', 4096) catch "") orelse ""`;
    }

    // Read all of stdin, max 10MB
    return `stdin.readAllAlloc(${curator}, 10 * 1024 * 1024) catch ""`;
}
