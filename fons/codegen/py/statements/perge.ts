/**
 * Python Code Generator - PergeStatement
 *
 * TRANSFORMS:
 *   perge -> continue
 */

import type { PyGenerator } from '../generator';

export function genPergeStatement(g: PyGenerator): string {
    return `${g.ind()}continue`;
}
