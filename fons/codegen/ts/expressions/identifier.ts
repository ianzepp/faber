/**
 * TypeScript Code Generator - Identifier Expression
 *
 * TRANSFORMS:
 *   x -> x
 *   nunc -> Date.now() (intrinsic constant)
 *   SECUNDUM -> 1000 (duration constant)
 *
 * WHY: Some identifiers map to intrinsic constants (norma/tempus durations).
 */

import type { Identifier } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

/**
 * TypeScript constant intrinsics.
 *
 * Maps Latin constant names to literal values.
 * Used for identifier references (not function calls).
 */
const TS_CONSTANTS: Record<string, string> = {
    // norma/tempus - Duration constants (milliseconds)
    MILLISECUNDUM: '1',
    SECUNDUM: '1000',
    MINUTUM: '60000',
    HORA: '3600000',
    DIES: '86400000',
};

export function genIdentifier(node: Identifier, _g: TsGenerator): string {
    // Check for constant intrinsics (norma/tempus duration constants, etc.)
    const constant = TS_CONSTANTS[node.name];
    if (constant) {
        return constant;
    }
    return node.name;
}
