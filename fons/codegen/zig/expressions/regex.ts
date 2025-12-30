/**
 * Zig Code Generator - Regex Literal Expression
 *
 * TRANSFORMS:
 *   sed "\\d+"          -> "\\d+"
 *   sed "hello" i       -> "(?i)hello"
 *
 * WHY: Zig has no stdlib regex. We emit pattern as a string literal
 *      for use with external libraries. Flags are injected as inline prefix.
 */

import type { RegexLiteral } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genRegexLiteral(node: RegexLiteral, _g: ZigGenerator): string {
    // WHY: Zig requires double-escaped backslashes in string literals
    const pattern = node.pattern.replace(/\\/g, '\\\\');
    const flags = node.flags;

    // If flags present, inject as inline prefix
    if (flags) {
        return `"(?${flags})${pattern}"`;
    }

    return `"${pattern}"`;
}
