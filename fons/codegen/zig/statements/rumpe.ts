/**
 * Zig Code Generator - RumpeStatement (break)
 *
 * TRANSFORMS:
 *   rumpe -> break
 *
 * TARGET: Zig uses 'break' keyword, same as most languages.
 */

import type { ZigGenerator } from '../generator';

export function genRumpeStatement(g: ZigGenerator): string {
    return `${g.ind()}break;`;
}
