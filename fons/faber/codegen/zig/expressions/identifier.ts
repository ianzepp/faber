/**
 * Zig Code Generator - Identifier Expression
 *
 * TRANSFORMS:
 *   name -> name (unchanged)
 *   PI -> std.math.pi
 *   E -> std.math.e
 *   TAU -> std.math.tau
 *   SECUNDUM -> 1000
 *
 * NOTE: verum/falsum/nihil are parsed as Literals, not Identifiers,
 *       so they're handled by literal.ts, not here.
 */

import type { Identifier } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

// WHY: Constants are inlined for simplicity. These match the .fab definitions.
const ZIG_CONSTANTS: Record<string, string> = {
    // mathesis constants
    PI: 'std.math.pi',
    E: 'std.math.e',
    TAU: 'std.math.tau',
    // tempus duration constants (milliseconds)
    MILLISECUNDUM: '1',
    SECUNDUM: '1000',
    MINUTUM: '60000',
    HORA: '3600000',
    DIES: '86400000',
};

export function genIdentifier(node: Identifier, _g: ZigGenerator): string {
    // Check for constants (mathesis + tempus)
    const constant = ZIG_CONSTANTS[node.name];
    if (constant) {
        return constant;
    }

    return node.name;
}
