/**
 * Faber Code Generator - Regex Literal Expression (roundtrip)
 *
 * TRANSFORMS:
 *   sed "\\d+"          -> sed "\d+"
 *   sed "hello" i       -> sed "hello" i
 *
 * WHY: Roundtrip to canonical Faber source.
 */

import type { RegexLiteral } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genRegexLiteral(node: RegexLiteral, _g: FabGenerator): string {
    const pattern = node.pattern;
    const flags = node.flags;

    if (flags) {
        return `sed "${pattern}" ${flags}`;
    }

    return `sed "${pattern}"`;
}
