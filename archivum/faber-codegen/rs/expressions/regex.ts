/**
 * Rust Code Generator - Regex Literal Expression
 *
 * TRANSFORMS:
 *   sed "\\d+"          -> Regex::new(r"\d+").unwrap()
 *   sed "hello" i       -> Regex::new(r"(?i)hello").unwrap()
 *   sed "^start" im     -> Regex::new(r"(?im)^start").unwrap()
 *
 * WHY: Rust uses the regex crate with inline flags prefixed to the pattern.
 *      Requires `use regex::Regex;` in preamble.
 */

import type { RegexLiteral } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genRegexLiteral(node: RegexLiteral, g: RsGenerator): string {
    // WHY: Mark that regex is used so use regex::Regex; can be added to preamble
    g.features.usesRegex = true;

    const pattern = node.pattern;
    const flags = node.flags;

    // If flags present, inject as inline prefix
    if (flags) {
        return `Regex::new(r"(?${flags})${pattern}").unwrap()`;
    }

    return `Regex::new(r"${pattern}").unwrap()`;
}
