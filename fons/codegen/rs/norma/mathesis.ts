/**
 * Mathesis Function Registry - Rust translations for Latin math functions
 *
 * COMPILER PHASE
 * ==============
 * codegen (Rust target)
 *
 * TARGET MAPPING
 * ==============
 * | Faber         | Rust                      |
 * |---------------|---------------------------|
 * | pavimentum    | x.floor()                 |
 * | tectum        | x.ceil()                  |
 * | rotundum      | x.round()                 |
 * | truncatum     | x.trunc()                 |
 * | radix         | x.sqrt()                  |
 * | potentia      | x.powf(n)                 |
 * | logarithmus   | x.ln()                    |
 * | logarithmus10 | x.log10()                 |
 * | exponens      | x.exp()                   |
 * | sinus         | x.sin()                   |
 * | cosinus       | x.cos()                   |
 * | tangens       | x.tan()                   |
 * | absolutum     | x.abs()                   |
 * | signum        | x.signum()                |
 * | minimus       | x.min(y)                  |
 * | maximus       | x.max(y)                  |
 * | constringens  | x.clamp(lo, hi)           |
 * | PI            | std::f64::consts::PI      |
 * | E             | std::f64::consts::E       |
 * | TAU           | std::f64::consts::TAU     |
 */

// =============================================================================
// TYPES
// =============================================================================

export type MathesisGenerator = (args: string[]) => string;

export interface MathesisEntry {
    latin: string;
    rs: string | MathesisGenerator;
}

// =============================================================================
// FUNCTION REGISTRY
// =============================================================================

export const MATHESIS_FUNCTIONS: Record<string, MathesisEntry> = {
    // Rounding
    pavimentum: { latin: 'pavimentum', rs: args => `(${args[0]} as f64).floor()` },
    tectum: { latin: 'tectum', rs: args => `(${args[0]} as f64).ceil()` },
    rotundum: { latin: 'rotundum', rs: args => `(${args[0]} as f64).round()` },
    truncatum: { latin: 'truncatum', rs: args => `(${args[0]} as f64).trunc()` },

    // Powers and roots
    radix: { latin: 'radix', rs: args => `(${args[0]} as f64).sqrt()` },
    potentia: { latin: 'potentia', rs: args => `(${args[0]} as f64).powf(${args[1]} as f64)` },
    logarithmus: { latin: 'logarithmus', rs: args => `(${args[0]} as f64).ln()` },
    logarithmus10: { latin: 'logarithmus10', rs: args => `(${args[0]} as f64).log10()` },
    exponens: { latin: 'exponens', rs: args => `(${args[0]} as f64).exp()` },

    // Trigonometry
    sinus: { latin: 'sinus', rs: args => `(${args[0]} as f64).sin()` },
    cosinus: { latin: 'cosinus', rs: args => `(${args[0]} as f64).cos()` },
    tangens: { latin: 'tangens', rs: args => `(${args[0]} as f64).tan()` },

    // Absolute and sign
    absolutum: { latin: 'absolutum', rs: args => `(${args[0]} as f64).abs()` },
    signum: { latin: 'signum', rs: args => `(${args[0]} as f64).signum()` },

    // Min/max/clamp
    minimus: { latin: 'minimus', rs: args => `(${args[0]} as f64).min(${args[1]} as f64)` },
    maximus: { latin: 'maximus', rs: args => `(${args[0]} as f64).max(${args[1]} as f64)` },
    constringens: {
        latin: 'constringens',
        rs: args => `(${args[0]} as f64).clamp(${args[1]} as f64, ${args[2]} as f64)`,
    },
};

// =============================================================================
// CONSTANT REGISTRY
// =============================================================================

export const MATHESIS_CONSTANTS: Record<string, string> = {
    PI: 'std::f64::consts::PI',
    E: 'std::f64::consts::E',
    TAU: 'std::f64::consts::TAU',
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

export function getMathesisFunction(name: string): MathesisEntry | undefined {
    return MATHESIS_FUNCTIONS[name];
}

export function getMathesisConstant(name: string): string | undefined {
    return MATHESIS_CONSTANTS[name];
}

export function isMathesisFunction(name: string): boolean {
    return name in MATHESIS_FUNCTIONS;
}

export function isMathesisConstant(name: string): boolean {
    return name in MATHESIS_CONSTANTS;
}
