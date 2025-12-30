/**
 * Python Code Generator - Regex Literal Expression
 *
 * TRANSFORMS:
 *   sed "\\d+"          -> re.compile(r'\d+')
 *   sed "hello" i       -> re.compile(r'(?i)hello')
 *   sed "^start" im     -> re.compile(r'(?im)^start')
 *
 * WHY: Python uses re.compile() with inline flags prefixed to the pattern.
 *      Flags are injected as (?flags) at the start of the pattern.
 */

import type { RegexLiteral } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genRegexLiteral(node: RegexLiteral, g: PyGenerator): string {
    // WHY: Mark that regex is used so import re can be added to preamble
    g.features.usesRegex = true;

    const pattern = node.pattern;
    const flags = node.flags;

    // If flags present, inject as inline prefix
    if (flags) {
        return `re.compile(r'(?${flags})${pattern}')`;
    }

    return `re.compile(r'${pattern}')`;
}
