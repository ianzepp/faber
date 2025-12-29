/**
 * C++23 Code Generator - Identifier
 *
 * TRANSFORMS:
 *   ego   -> this
 *   other -> other
 *
 * NOTE: verum/falsum/nihil are parsed as Literals, not Identifiers,
 *       so they're handled by literal.ts, not here.
 */

import type { Identifier } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genIdentifier(node: Identifier, _g: CppGenerator): string {
    if (node.name === 'ego') {
        return 'this';
    }
    return node.name;
}
