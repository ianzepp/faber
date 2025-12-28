/**
 * Python Code Generator - RumpeStatement
 *
 * TRANSFORMS:
 *   rumpe -> break
 */

import type { PyGenerator } from '../generator';

export function genRumpeStatement(g: PyGenerator): string {
    return `${g.ind()}break`;
}
