/**
 * Mathesis Function Registry - C++ translations for Latin math functions
 *
 * COMPILER PHASE
 * ==============
 * codegen (C++ target)
 *
 * ARCHITECTURE
 * ============
 * This module defines C++ translations for mathesis (math) module functions.
 * These are imported via `ex "norma/mathesis" importa pavimentum, radix, PI`.
 *
 * C++ IDIOMS
 * ==========
 * | Faber         | C++                |
 * |---------------|--------------------|
 * | pavimentum    | std::floor         |
 * | tectum        | std::ceil          |
 * | rotundum      | std::round         |
 * | truncatum     | std::trunc         |
 * | radix         | std::sqrt          |
 * | potentia      | std::pow           |
 * | logarithmus   | std::log           |
 * | logarithmus10 | std::log10         |
 * | exponens      | std::exp           |
 * | sinus         | std::sin           |
 * | cosinus       | std::cos           |
 * | tangens       | std::tan           |
 * | absolutum     | std::abs           |
 * | signum        | (x > 0) - (x < 0)  |
 * | minimus       | std::min           |
 * | maximus       | std::max           |
 * | constringens  | std::clamp         |
 * | PI            | std::numbers::pi   |
 * | E             | std::numbers::e    |
 * | TAU           | (std::numbers::pi * 2) |
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Latin function/constant name
 * OUTPUT: C++ code string
 * ERRORS: Returns undefined if name not recognized
 */

// =============================================================================
// TYPES
// =============================================================================

export type MathesisGenerator = (args: string[]) => string;

export interface MathesisEntry {
    latin: string;
    cpp: string | MathesisGenerator;
    headers: string[];
}

// =============================================================================
// FUNCTION REGISTRY
// =============================================================================

export const MATHESIS_FUNCTIONS: Record<string, MathesisEntry> = {
    // Rounding
    pavimentum: {
        latin: 'pavimentum',
        cpp: args => `std::floor(${args[0]})`,
        headers: ['<cmath>'],
    },
    tectum: {
        latin: 'tectum',
        cpp: args => `std::ceil(${args[0]})`,
        headers: ['<cmath>'],
    },
    rotundum: {
        latin: 'rotundum',
        cpp: args => `std::round(${args[0]})`,
        headers: ['<cmath>'],
    },
    truncatum: {
        latin: 'truncatum',
        cpp: args => `std::trunc(${args[0]})`,
        headers: ['<cmath>'],
    },

    // Powers and roots
    radix: {
        latin: 'radix',
        cpp: args => `std::sqrt(${args[0]})`,
        headers: ['<cmath>'],
    },
    potentia: {
        latin: 'potentia',
        cpp: args => `std::pow(${args[0]}, ${args[1]})`,
        headers: ['<cmath>'],
    },
    logarithmus: {
        latin: 'logarithmus',
        cpp: args => `std::log(${args[0]})`,
        headers: ['<cmath>'],
    },
    logarithmus10: {
        latin: 'logarithmus10',
        cpp: args => `std::log10(${args[0]})`,
        headers: ['<cmath>'],
    },
    exponens: {
        latin: 'exponens',
        cpp: args => `std::exp(${args[0]})`,
        headers: ['<cmath>'],
    },

    // Trigonometry
    sinus: {
        latin: 'sinus',
        cpp: args => `std::sin(${args[0]})`,
        headers: ['<cmath>'],
    },
    cosinus: {
        latin: 'cosinus',
        cpp: args => `std::cos(${args[0]})`,
        headers: ['<cmath>'],
    },
    tangens: {
        latin: 'tangens',
        cpp: args => `std::tan(${args[0]})`,
        headers: ['<cmath>'],
    },

    // Absolute and sign
    absolutum: {
        latin: 'absolutum',
        cpp: args => `std::abs(${args[0]})`,
        headers: ['<cmath>'],
    },
    signum: {
        latin: 'signum',
        // WHY: C++ has no std::sign. Use comparison trick: (x > 0) - (x < 0)
        // Returns 1 for positive, -1 for negative, 0 for zero.
        cpp: args => `((${args[0]} > 0) - (${args[0]} < 0))`,
        headers: [],
    },

    // Min/max/clamp
    minimus: {
        latin: 'minimus',
        cpp: args => `std::min(${args[0]}, ${args[1]})`,
        headers: ['<algorithm>'],
    },
    maximus: {
        latin: 'maximus',
        cpp: args => `std::max(${args[0]}, ${args[1]})`,
        headers: ['<algorithm>'],
    },
    constringens: {
        latin: 'constringens',
        cpp: args => `std::clamp(${args[0]}, ${args[1]}, ${args[2]})`,
        headers: ['<algorithm>'],
    },
};

// =============================================================================
// CONSTANT REGISTRY
// =============================================================================

export interface MathesisConstant {
    latin: string;
    cpp: string;
    headers: string[];
}

export const MATHESIS_CONSTANTS: Record<string, MathesisConstant> = {
    PI: {
        latin: 'PI',
        cpp: 'std::numbers::pi',
        headers: ['<numbers>'],
    },
    E: {
        latin: 'E',
        cpp: 'std::numbers::e',
        headers: ['<numbers>'],
    },
    TAU: {
        latin: 'TAU',
        cpp: '(std::numbers::pi * 2)',
        headers: ['<numbers>'],
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

export function getMathesisFunction(name: string): MathesisEntry | undefined {
    return MATHESIS_FUNCTIONS[name];
}

export function getMathesisConstant(name: string): MathesisConstant | undefined {
    return MATHESIS_CONSTANTS[name];
}

export function isMathesisFunction(name: string): boolean {
    return name in MATHESIS_FUNCTIONS;
}

export function isMathesisConstant(name: string): boolean {
    return name in MATHESIS_CONSTANTS;
}

export function getMathesisHeaders(name: string): string[] {
    const func = MATHESIS_FUNCTIONS[name];
    if (func) return func.headers;
    const constant = MATHESIS_CONSTANTS[name];
    if (constant) return constant.headers;
    return [];
}
