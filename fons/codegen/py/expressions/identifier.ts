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
import { getMathesisConstant } from '../norma/mathesis';

/**
 * Python tempus (time) duration constants.
 *
 * WHY: Duration constants are in milliseconds, same as TypeScript.
 */
const PY_TEMPUS_CONSTANTS: Record<string, string> = {
    MILLISECUNDUM: '1',
    SECUNDUM: '1000',
    MINUTUM: '60000',
    HORA: '3600000',
    DIES: '86400000',
};

export function genIdentifier(node: Identifier, g: PyGenerator): string {
    // Check for tempus constants (MILLISECUNDUM, SECUNDUM, etc.)
    const tempusConst = PY_TEMPUS_CONSTANTS[node.name];
    if (tempusConst) {
        return tempusConst;
    }

    // Check for mathesis constants (PI, E, TAU)
    const mathesisConst = getMathesisConstant(node.name);
    if (mathesisConst) {
        if (mathesisConst.requiresMath) {
            g.features.math = true;
        }
        return mathesisConst.py;
    }

    return node.name;
}
