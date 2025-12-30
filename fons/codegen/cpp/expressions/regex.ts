/**
 * C++ Code Generator - Regex Literal Expression
 *
 * TRANSFORMS:
 *   sed "\\d+"          -> std::regex("\\d+")
 *   sed "hello" i       -> std::regex("hello") // flags not supported inline
 *
 * WHY: C++ <regex> does not support inline flags. Only flagless patterns work.
 *      Flags require manual translation to std::regex_constants.
 *      We emit the pattern only and warn in documentation that flags are ignored.
 */

import type { RegexLiteral } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genRegexLiteral(node: RegexLiteral, _g: CppGenerator): string {
    // WHY: C++ requires double-escaped backslashes in string literals
    // The pattern comes from Faber source, already single-escaped
    const pattern = node.pattern.replace(/\\/g, '\\\\');

    // WHY: C++ doesn't support inline flags like (?i)
    // We emit only the pattern; flags are silently ignored
    // Documentation warns users about this limitation
    return `std::regex("${pattern}")`;
}
