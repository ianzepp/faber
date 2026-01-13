/**
 * Rust Code Generator - PergeStatement
 *
 * TRANSFORMS:
 *   perge -> continue;
 */

import type { RsGenerator } from '../generator';

export function genPergeStatement(g: RsGenerator): string {
    return `${g.ind()}continue;`;
}
