/**
 * Zig Code Generator - IncipietStatement (async entry point)
 *
 * TARGET: Zig does not support async main.
 *         This generates a compile error comment.
 */

import type { IncipietStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genIncipietStatement(_node: IncipietStatement, g: ZigGenerator): string {
    // WHY: Zig has no async main - the evented I/O model is different.
    //      We emit a compile-time error to make this explicit.
    return `${g.ind()}@compileError("incipiet (async main) is not supported in Zig - use incipit with explicit async handling")`;
}
