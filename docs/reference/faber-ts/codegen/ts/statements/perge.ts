/**
 * TypeScript Code Generator - PergeStatement
 *
 * TRANSFORMS:
 *   perge -> continue;
 */

import type { TsGenerator } from '../generator';

export function genPergeStatement(g: TsGenerator, semi: boolean): string {
    return `${g.ind()}continue${semi ? ';' : ''}`;
}
