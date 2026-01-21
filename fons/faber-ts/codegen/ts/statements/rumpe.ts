/**
 * TypeScript Code Generator - RumpeStatement
 *
 * TRANSFORMS:
 *   rumpe -> break;
 */

import type { TsGenerator } from '../generator';

export function genRumpeStatement(g: TsGenerator, semi: boolean): string {
    return `${g.ind()}break${semi ? ';' : ''}`;
}
