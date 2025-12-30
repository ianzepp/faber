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
import { getMathesisConstant } from '../norma/mathesis';
import { getTempusConstant } from '../norma/tempus';

export function genIdentifier(node: Identifier, g: ZigGenerator): string {
    // Suppress unused parameter warning
    void g;

    // Check for mathesis constants (PI, E, TAU)
    const mathesisConst = getMathesisConstant(node.name);
    if (mathesisConst) {
        return mathesisConst;
    }

    // Check for tempus constants (MILLISECUNDUM, SECUNDUM, MINUTUM, HORA, DIES)
    const tempusConst = getTempusConstant(node.name);
    if (tempusConst) {
        return tempusConst;
    }

    return node.name;
}
