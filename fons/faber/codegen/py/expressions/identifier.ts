/**
 * Python Code Generator - Identifier Expression
 *
 * TRANSFORMS:
 *   other -> other (unchanged)
 *   PI -> math.pi (mathesis constant)
 *   SECUNDUM -> 1000 (tempus duration constant)
 *
 * NOTE: verum/falsum/nihil are parsed as Literals, not Identifiers,
 *       so they're handled by literal.ts, not here.
 */

import type { Identifier } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

// WHY: Constants are inlined for simplicity. These match the .fab definitions.
const PY_CONSTANTS: Record<string, { value: string; requiresMath?: boolean }> = {
    // mathesis constants
    PI: { value: 'math.pi', requiresMath: true },
    E: { value: 'math.e', requiresMath: true },
    TAU: { value: 'math.tau', requiresMath: true },
    // tempus duration constants (milliseconds)
    MILLISECUNDUM: { value: '1' },
    SECUNDUM: { value: '1000' },
    MINUTUM: { value: '60000' },
    HORA: { value: '3600000' },
    DIES: { value: '86400000' },
};

export function genIdentifier(node: Identifier, g: PyGenerator): string {
    // Check for constants (mathesis + tempus)
    const constant = PY_CONSTANTS[node.name];
    if (constant) {
        if (constant.requiresMath) {
            g.features.math = true;
        }
        return constant.value;
    }

    return node.name;
}
