/**
 * Rust Code Generator - Literal Expression
 *
 * TRANSFORMS:
 *   42 -> 42
 *   "hello" -> String::from("hello")
 *   verum -> true
 *   falsum -> false
 *   nullum -> None
 *   123n -> 123i128
 */

import type { Literal } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genLiteral(node: Literal, _g: RsGenerator): string {
    if (node.value === null) {
        return 'None';
    }

    if (typeof node.value === 'string') {
        return `String::from(${JSON.stringify(node.value)})`;
    }

    if (typeof node.value === 'boolean') {
        return node.value ? 'true' : 'false';
    }

    if (typeof node.value === 'bigint') {
        return `${node.raw}i128`;
    }

    return node.raw;
}
