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

// WHY: Constants are inlined for simplicity. These match the .fab definitions.
const CPP_CONSTANTS: Record<string, { value: string; headers?: string[] }> = {
    // mathesis constants
    PI: { value: 'std::numbers::pi', headers: ['<numbers>'] },
    E: { value: 'std::numbers::e', headers: ['<numbers>'] },
    TAU: { value: '(std::numbers::pi * 2)', headers: ['<numbers>'] },
    // tempus duration constants (milliseconds)
    MILLISECUNDUM: { value: '1LL' },
    SECUNDUM: { value: '1000LL' },
    MINUTUM: { value: '60000LL' },
    HORA: { value: '3600000LL' },
    DIES: { value: '86400000LL' },
};

export function genIdentifier(node: Identifier, g: CppGenerator): string {
    if (node.name === 'ego') {
        return 'this';
    }

    // Check for constants (mathesis + tempus)
    const constant = CPP_CONSTANTS[node.name];
    if (constant) {
        if (constant.headers) {
            for (const header of constant.headers) {
                g.includes.add(header);
            }
        }
        return constant.value;
    }

    return node.name;
}
