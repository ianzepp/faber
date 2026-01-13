/**
 * C++23 Code Generator - RumpeStatement
 *
 * TRANSFORMS:
 *   rumpe -> break;
 */

import type { CppGenerator } from '../generator';

export function genRumpeStatement(g: CppGenerator): string {
    return `${g.ind()}break;`;
}
