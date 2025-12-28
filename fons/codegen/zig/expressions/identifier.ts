/**
 * Zig Code Generator - Identifier Expression
 *
 * TRANSFORMS:
 *   verum  -> true
 *   falsum -> false
 *   nihil  -> null
 *   other  -> other (unchanged, or m_name for module constants)
 *
 * WHY: Latin boolean/null literals map to Zig's lowercase equivalents.
 *      Module constants use m_ prefix to avoid shadowing.
 */

import type { Identifier } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genIdentifier(node: Identifier, g: ZigGenerator): string {
    switch (node.name) {
        case 'verum':
            return 'true';
        case 'falsum':
            return 'false';
        case 'nihil':
            return 'null';
        default:
            // Use m_ prefix for module constants to match declaration
            if (g.hasModuleConstant(node.name)) {
                return `m_${node.name}`;
            }
            return node.name;
    }
}
