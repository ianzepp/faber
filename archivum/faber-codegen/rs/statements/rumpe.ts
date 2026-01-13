/**
 * Rust Code Generator - RumpeStatement
 *
 * TRANSFORMS:
 *   rumpe -> break;
 */

import type { RsGenerator } from '../generator';

export function genRumpeStatement(g: RsGenerator): string {
    return `${g.ind()}break;`;
}
