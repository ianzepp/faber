/**
 * Python Code Generator - Identifier Expression
 *
 * TRANSFORMS:
 *   verum  -> True
 *   falsum -> False
 *   nihil  -> None
 *   other  -> other (unchanged)
 *
 * WHY: Latin boolean/null literals map to Python's capitalized equivalents.
 */

import type { Identifier } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genIdentifier(node: Identifier, _g: PyGenerator): string {
    switch (node.name) {
        case 'verum':
            return 'True';
        case 'falsum':
            return 'False';
        case 'nihil':
            return 'None';
        default:
            return node.name;
    }
}
