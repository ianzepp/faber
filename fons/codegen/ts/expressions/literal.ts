/**
 * TypeScript Code Generator - Literal Expression
 *
 * TRANSFORMS:
 *   "hello" -> "hello" (JSON-escaped)
 *   42 -> 42
 *   verum -> true
 *   nihil -> null
 *
 * WHY: JSON.stringify ensures proper escaping of string literals.
 */

import type { Literal } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genLiteral(node: Literal, _g: TsGenerator): string {
    if (node.value === null) {
        return 'null';
    }

    if (typeof node.value === 'string') {
        return JSON.stringify(node.value);
    }

    if (typeof node.value === 'boolean') {
        return node.value ? 'true' : 'false';
    }

    if (typeof node.value === 'bigint') {
        // WHY: Use raw to preserve hex format, raw already includes 'n' suffix
        return node.raw;
    }

    // WHY: Use raw to preserve original format (hex: 0xFF, decimal: 123)
    if (typeof node.value === 'number') {
        return node.raw;
    }

    return String(node.value);
}
