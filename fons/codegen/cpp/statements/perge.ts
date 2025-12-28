/**
 * C++23 Code Generator - PergeStatement
 *
 * TRANSFORMS:
 *   perge -> continue;
 */

import type { CppGenerator } from '../generator';

export function genPergeStatement(g: CppGenerator): string {
    return `${g.ind()}continue;`;
}
