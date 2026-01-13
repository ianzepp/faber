/**
 * Rust Code Generator - Literal Expression
 *
 * TRANSFORMS:
 *   42 -> 42
 *   "hello" -> "hello"
 *   verum -> true
 *   falsum -> false
 *   nullum -> None
 *   123n -> 123i128
 *
 * WHY: String literals emit as &str (bare). Rust's type system handles conversions:
 * - Methods expecting &str receive it directly
 * - Variables typed as String get automatic coercion via type annotation
 * - No need for explicit String::from() wrapper
 */

import type { Literal } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genLiteral(node: Literal, _g: RsGenerator): string {
    if (node.value === null) {
        return 'None';
    }

    if (typeof node.value === 'string') {
        // WHY: Emit bare string literal as &str. Rust will coerce to String where needed.
        // Preserves escape sequences like \u0048, \n, \t via node.raw.
        return node.raw;
    }

    if (typeof node.value === 'boolean') {
        return node.value ? 'true' : 'false';
    }

    if (typeof node.value === 'bigint') {
        return `${node.raw}i128`;
    }

    return node.raw;
}
