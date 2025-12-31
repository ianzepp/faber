/**
 * Zig Code Generator - Lege Expression (read stdin)
 *
 * TRANSFORMS:
 *   lege() -> stdin.readAllAlloc(alloc, 10 * 1024 * 1024) catch ""
 *
 * TARGET: Zig's std.io.getStdIn().reader() for stdin input.
 *
 * WHY: Bootstrap compiler needs to read source from stdin.
 *      Returns empty string on error for simplicity.
 */

import type { LegeExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genLegeExpression(_node: LegeExpression, g: ZigGenerator): string {
    g.features.stdin = true;
    const curator = g.getCurator();

    // Read all of stdin, max 10MB
    return `stdin.readAllAlloc(${curator}, 10 * 1024 * 1024) catch ""`;
}
