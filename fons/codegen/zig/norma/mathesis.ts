/**
 * Mathesis Function Registry - Zig translations for Latin math functions
 *
 * COMPILER PHASE
 * ==============
 * codegen (Zig target)
 *
 * TARGET MAPPING
 * ==============
 * | Faber         | Zig                       |
 * |---------------|---------------------------|
 * | pavimentum    | @floor(x)                 |
 * | tectum        | @ceil(x)                  |
 * | rotundum      | @round(x)                 |
 * | truncatum     | @trunc(x)                 |
 * | radix         | @sqrt(x)                  |
 * | potentia      | std.math.pow(x, n)        |
 * | logarithmus   | @log(x)                   |
 * | logarithmus10 | @log10(x)                 |
 * | exponens      | @exp(x)                   |
 * | sinus         | @sin(x)                   |
 * | cosinus       | @cos(x)                   |
 * | tangens       | @tan(x)                   |
 * | absolutum     | @abs(x)                   |
 * | signum        | std.math.sign(x)          |
 * | minimus       | @min(x, y)                |
 * | maximus       | @max(x, y)                |
 * | constringens  | std.math.clamp(x, lo, hi) |
 * | PI            | std.math.pi               |
 * | E             | std.math.e                |
 * | TAU           | std.math.tau              |
 *
 * NOTE: Zig builtins (@floor, @sqrt, etc.) work on f32/f64.
 *       std.math functions are used where no builtin exists.
 */

// =============================================================================
// TYPES
// =============================================================================

export type MathesisGenerator = (args: string[]) => string;

export interface MathesisEntry {
    latin: string;
    zig: string | MathesisGenerator;
}

// =============================================================================
// FUNCTION REGISTRY
// =============================================================================

export const MATHESIS_FUNCTIONS: Record<string, MathesisEntry> = {
    // Rounding
    pavimentum: { latin: 'pavimentum', zig: args => `@floor(${args[0]})` },
    tectum: { latin: 'tectum', zig: args => `@ceil(${args[0]})` },
    rotundum: { latin: 'rotundum', zig: args => `@round(${args[0]})` },
    truncatum: { latin: 'truncatum', zig: args => `@trunc(${args[0]})` },

    // Powers and roots
    radix: { latin: 'radix', zig: args => `@sqrt(${args[0]})` },
    potentia: { latin: 'potentia', zig: args => `std.math.pow(f64, ${args[0]}, ${args[1]})` },
    logarithmus: { latin: 'logarithmus', zig: args => `@log(${args[0]})` },
    logarithmus10: { latin: 'logarithmus10', zig: args => `@log10(${args[0]})` },
    exponens: { latin: 'exponens', zig: args => `@exp(${args[0]})` },

    // Trigonometry
    sinus: { latin: 'sinus', zig: args => `@sin(${args[0]})` },
    cosinus: { latin: 'cosinus', zig: args => `@cos(${args[0]})` },
    tangens: { latin: 'tangens', zig: args => `@tan(${args[0]})` },

    // Absolute and sign
    absolutum: { latin: 'absolutum', zig: args => `@abs(${args[0]})` },
    signum: { latin: 'signum', zig: args => `std.math.sign(${args[0]})` },

    // Min/max/clamp
    minimus: { latin: 'minimus', zig: args => `@min(${args[0]}, ${args[1]})` },
    maximus: { latin: 'maximus', zig: args => `@max(${args[0]}, ${args[1]})` },
    constringens: {
        latin: 'constringens',
        zig: args => `std.math.clamp(${args[0]}, ${args[1]}, ${args[2]})`,
    },
};

// =============================================================================
// CONSTANT REGISTRY
// =============================================================================

export const MATHESIS_CONSTANTS: Record<string, string> = {
    PI: 'std.math.pi',
    E: 'std.math.e',
    TAU: 'std.math.tau',
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
