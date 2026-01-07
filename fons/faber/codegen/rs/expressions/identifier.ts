/**
 * Rust Code Generator - Identifier Expression
 *
 * TRANSFORMS:
 *   myVar -> myVar
 *   PI -> std::f64::consts::PI
 *   E -> std::f64::consts::E
 *   TAU -> std::f64::consts::TAU
 *   SECUNDUM -> 1000_i64
 */

import type { Identifier } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

// WHY: Constants are inlined for simplicity. These match the .fab definitions.
const RS_CONSTANTS: Record<string, string> = {
    // mathesis constants
    PI: 'std::f64::consts::PI',
    E: 'std::f64::consts::E',
    TAU: 'std::f64::consts::TAU',
    // tempus duration constants (milliseconds)
    MILLISECUNDUM: '1_i64',
    SECUNDUM: '1000_i64',
    MINUTUM: '60000_i64',
    HORA: '3600000_i64',
    DIES: '86400000_i64',
};

export function genIdentifier(node: Identifier, _g: RsGenerator): string {
    // Check for constants (mathesis + tempus)
    const constant = RS_CONSTANTS[node.name];
    if (constant) {
        return constant;
    }

    return node.name;
}
