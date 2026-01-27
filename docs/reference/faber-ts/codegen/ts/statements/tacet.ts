/**
 * TypeScript Code Generator - TacetStatement
 *
 * TRANSFORMS:
 *   tacet -> {}
 */

import type { TsGenerator } from '../generator';

export function genTacetStatement(g: TsGenerator): string {
    return `${g.ind()}{}`;
}
