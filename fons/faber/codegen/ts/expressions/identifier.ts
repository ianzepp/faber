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
import { getNormaTranslation } from '../../norma';

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

    // Check for mathesis constants (PI, E, TAU) via norma registry
    // WHY: Only match zero-arg functions that act as constants.
    //      Functions like radix(x) and signum(x) require parameters and
    //      should NOT be matched here - they're local variable names.
    const mathesisConst = getNormaTranslation('ts', 'mathesis', node.name);
    if (mathesisConst?.template && mathesisConst?.params?.length === 0) {
        return mathesisConst.template;
    }

    return node.name;
}
