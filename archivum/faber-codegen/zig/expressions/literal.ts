/**
 * Zig Code Generator - Literal Expression
 *
 * TRANSFORMS:
 *   "hello" -> "hello"
 *   42 -> 42
 *   verum -> true
 *   nihil -> null
 *
 * TARGET: Zig string literals use double quotes, no escaping needed for simple strings.
 */

import type { Literal } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genLiteral(node: Literal, _g: ZigGenerator): string {
    if (node.value === null) {
        return 'null';
    }

    if (typeof node.value === 'string') {
        // WHY: Use raw to preserve escape sequences like \u0048, \n, \t as-is.
        // Manual escaping would double-escape backslashes.
        return node.raw;
    }

    if (typeof node.value === 'boolean') {
        return node.value ? 'true' : 'false';
    }

    // WHY: Zig comptime_int is arbitrary precision, no 'n' suffix needed
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
