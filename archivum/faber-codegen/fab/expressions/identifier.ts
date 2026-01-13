/**
 * Faber Code Generator - Identifier
 */

import type { Identifier } from '../../../parser/ast';

export function genIdentifier(node: Identifier): string {
    return node.name;
}
