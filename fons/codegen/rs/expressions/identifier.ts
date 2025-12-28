/**
 * Rust Code Generator - Identifier Expression
 *
 * TRANSFORMS:
 *   myVar -> myVar
 */

import type { Identifier } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genIdentifier(node: Identifier, _g: RsGenerator): string {
    return node.name;
}
