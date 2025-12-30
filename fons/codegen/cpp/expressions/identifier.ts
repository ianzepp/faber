/**
 * C++23 Code Generator - Identifier
 *
 * TRANSFORMS:
 *   ego   -> this
 *   PI    -> std::numbers::pi
 *   E     -> std::numbers::e
 *   TAU   -> (std::numbers::pi * 2)
 *   other -> other
 *
 * NOTE: verum/falsum/nihil are parsed as Literals, not Identifiers,
 *       so they're handled by literal.ts, not here.
 */

import type { Identifier } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { getMathesisConstant, getMathesisHeaders } from '../norma/mathesis';

export function genIdentifier(node: Identifier, g: CppGenerator): string {
    if (node.name === 'ego') {
        return 'this';
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
