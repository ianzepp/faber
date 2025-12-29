/**
 * Zig Code Generator - Identifier Expression
 *
 * TRANSFORMS:
 *   name -> name (unchanged)
 *   name -> m_name (for module constants to avoid shadowing)
 *
 * NOTE: verum/falsum/nihil are parsed as Literals, not Identifiers,
 *       so they're handled by literal.ts, not here.
 */

import type { Identifier } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genIdentifier(node: Identifier, g: ZigGenerator): string {
    // Use m_ prefix for module constants to match declaration
    if (g.hasModuleConstant(node.name)) {
        return `m_${node.name}`;
    }
    return node.name;
}
