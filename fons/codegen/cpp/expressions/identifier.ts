/**
 * C++23 Code Generator - Identifier
 *
 * TRANSFORMS:
 *   verum   -> true
 *   falsum  -> false
 *   nihil   -> nullptr
 *   ego     -> this
 *   other   -> other
 */

import type { Identifier } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genIdentifier(node: Identifier, g: CppGenerator): string {
    switch (node.name) {
        case 'verum':
            return 'true';
        case 'falsum':
            return 'false';
        case 'nihil':
            return 'nullptr';
        case 'ego':
            return 'this';
        default:
            return node.name;
    }
}
