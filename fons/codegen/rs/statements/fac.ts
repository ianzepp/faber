/**
 * Rust Code Generator - FacBlockStatement
 *
 * TRANSFORMS:
 *   fac { ... } -> { ... }
 */

import type { FacBlockStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genFacBlockStatement(node: FacBlockStatement, g: RsGenerator): string {
    return `${g.ind()}${genBlockStatement(node.body, g)}`;
}
