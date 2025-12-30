/**
 * TypeScript Code Generator - Regex Literal Expression
 *
 * TRANSFORMS:
 *   sed "\\d+"          -> /\d+/
 *   sed "hello" i       -> /hello/i
 *   sed "^start" im     -> /^start/im
 *
 * WHY: TypeScript uses native regex literals with suffix flags.
 *      Pattern is passed through verbatim - no escaping transformation.
 */

import type { RegexLiteral } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genRegexLiteral(node: RegexLiteral, _g: TsGenerator): string {
    // WHY: Pattern passes through verbatim. Faber doesn't validate regex syntax.
    // Forward slashes must be escaped in JS/TS regex literals.
    const pattern = node.pattern.replace(/\//g, '\\/');
    const flags = node.flags;

    return `/${pattern}/${flags}`;
}
