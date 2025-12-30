/**
 * Rust Code Generator - Identifier Expression
 *
 * TRANSFORMS:
 *   myVar -> myVar
 *   PI -> std::f64::consts::PI
 *   E -> std::f64::consts::E
 *   TAU -> std::f64::consts::TAU
 */

import type { Identifier } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { getMathesisConstant } from '../norma/mathesis';

export function genIdentifier(node: Identifier, _g: RsGenerator): string {
    // Check for mathesis constants (PI, E, TAU)
    const mathesisConst = getMathesisConstant(node.name);
    if (mathesisConst) {
        return mathesisConst;
    }

    return node.name;
}
