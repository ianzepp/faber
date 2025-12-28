/**
 * Zig Code Generator - PergeStatement (continue)
 *
 * TRANSFORMS:
 *   perge -> continue
 *
 * TARGET: Zig uses 'continue' keyword, same as most languages.
 */

import type { ZigGenerator } from '../generator';

export function genPergeStatement(g: ZigGenerator): string {
    return `${g.ind()}continue;`;
}
