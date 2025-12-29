/**
 * Python Code Generator - Identifier Expression
 *
 * TRANSFORMS:
 *   other -> other (unchanged)
 *
 * NOTE: verum/falsum/nihil are parsed as Literals, not Identifiers,
 *       so they're handled by literal.ts, not here.
 */

import type { Identifier } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genIdentifier(node: Identifier, _g: PyGenerator): string {
    return node.name;
}
