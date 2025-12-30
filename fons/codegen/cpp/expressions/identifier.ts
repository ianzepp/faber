/**
 * C++23 Code Generator - Identifier
 *
 * TRANSFORMS:
 *   ego   -> this
 *   PI    -> std::numbers::pi
 *   E     -> std::numbers::e
 *   TAU   -> (std::numbers::pi * 2)
 *   SECUNDUM -> 1000 (tempus duration constant)
 *   other -> other
 *
 * NOTE: verum/falsum/nihil are parsed as Literals, not Identifiers,
 *       so they're handled by literal.ts, not here.
 */

import type { Identifier } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { getMathesisConstant, getMathesisHeaders } from '../norma/mathesis';

/**
 * C++ tempus (time) duration constants.
 *
 * WHY: Duration constants are in milliseconds, same as TypeScript.
 *      Use int64_t for consistent sizing with chrono duration counts.
 */
const CPP_TEMPUS_CONSTANTS: Record<string, string> = {
    MILLISECUNDUM: '1LL',
    SECUNDUM: '1000LL',
    MINUTUM: '60000LL',
    HORA: '3600000LL',
    DIES: '86400000LL',
};

export function genIdentifier(node: Identifier, g: CppGenerator): string {
    if (node.name === 'ego') {
        return 'this';
    }

    // Check tempus constants (MILLISECUNDUM, SECUNDUM, etc.)
    const tempusConst = CPP_TEMPUS_CONSTANTS[node.name];
    if (tempusConst) {
        return tempusConst;
    }

    // Check mathesis constants (PI, E, TAU)
    const mathConst = getMathesisConstant(node.name);
    if (mathConst) {
        for (const header of getMathesisHeaders(node.name)) {
            g.includes.add(header);
        }
        return mathConst.cpp;
    }

    return node.name;
}
