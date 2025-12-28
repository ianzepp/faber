/**
 * Python Code Generator - Literal Expression
 *
 * TRANSFORMS:
 *   null       -> None
 *   "string"   -> "string" (JSON escaped)
 *   true/false -> True/False
 *   123n       -> 123 (bigint suffix stripped)
 *   0xFF       -> 0xFF (preserved format)
 *
 * WHY: Python integers are arbitrary precision, no 'n' suffix needed.
 * WHY: Use raw to preserve original format (hex: 0xFF, decimal: 123).
 */

import type { Literal } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genLiteral(node: Literal, _g: PyGenerator): string {
    if (node.value === null) {
        return 'None';
    }

    if (typeof node.value === 'string') {
        // WHY: Use raw to preserve escape sequences like \u0048, \n, \t as-is.
        // JSON.stringify would double-escape backslashes.
        return node.raw;
    }

    if (typeof node.value === 'boolean') {
        return node.value ? 'True' : 'False';
    }

    // WHY: Python integers are arbitrary precision, no 'n' suffix needed
    if (typeof node.value === 'bigint') {
        // Strip 'n' suffix from raw (e.g., "0xFFn" -> "0xFF")
        return node.raw.replace(/n$/, '');
    }

    // WHY: Use raw to preserve original format (hex: 0xFF, decimal: 123)
    if (typeof node.value === 'number') {
        return node.raw;
    }

    return String(node.value);
}
