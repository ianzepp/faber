/**
 * Aleator Function Registry - C++ translations for Latin random functions
 *
 * COMPILER PHASE
 * ==============
 * codegen (C++ target)
 *
 * TARGET MAPPING
 * ==============
 * | Faber      | C++                                              |
 * |------------|--------------------------------------------------|
 * | fractus    | std::uniform_real_distribution<>(0.0, 1.0)(gen)  |
 * | inter      | std::uniform_int_distribution<>(min, max)(gen)   |
 * | octeti     | (generate random bytes into vector)              |
 * | uuid       | (generate UUID v4 string)                        |
 * | selige     | arr[dist(gen)]                                   |
 * | misce      | std::ranges::shuffle(copy, gen)                  |
 * | semen      | gen.seed(n)                                      |
 *
 * WHY: C++ uses <random> header with generator + distribution pattern.
 *      We use std::random_device for seeding and std::mt19937 for generation.
 */

// =============================================================================
// TYPES
// =============================================================================

export type AleatorGenerator = (args: string[]) => string;

export interface AleatorEntry {
    latin: string;
    cpp: string | AleatorGenerator;
    headers: string[];
}

// =============================================================================
// FUNCTION REGISTRY
// =============================================================================

export const ALEATOR_FUNCTIONS: Record<string, AleatorEntry> = {
    // Basic generation
    fractus: {
        latin: 'fractus',
        cpp: () => `[&]{ std::random_device rd; std::mt19937 gen(rd()); std::uniform_real_distribution<> dis(0.0, 1.0); return dis(gen); }()`,
        headers: ['<random>'],
    },
    inter: {
        latin: 'inter',
        cpp: args =>
            `[&]{ std::random_device rd; std::mt19937 gen(rd()); std::uniform_int_distribution<> dis(${args[0]}, ${args[1]}); return dis(gen); }()`,
        headers: ['<random>'],
    },
    octeti: {
        latin: 'octeti',
        cpp: args =>
            `[&]{ std::vector<uint8_t> bytes(${args[0]}); std::random_device rd; for (auto& b : bytes) b = static_cast<uint8_t>(rd()); return bytes; }()`,
        headers: ['<random>', '<vector>', '<cstdint>'],
    },
    uuid: {
        latin: 'uuid',
        // WHY: Generate UUID v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
        cpp: () =>
            `[&]{ std::random_device rd; std::mt19937 gen(rd()); std::uniform_int_distribution<> dis(0, 15); auto hex = [](int n) { return "0123456789abcdef"[n]; }; std::string uuid; for (int i = 0; i < 36; ++i) { if (i == 8 || i == 13 || i == 18 || i == 23) uuid += '-'; else if (i == 14) uuid += '4'; else if (i == 19) uuid += hex((dis(gen) & 0x3) | 0x8); else uuid += hex(dis(gen)); } return uuid; }()`,
        headers: ['<random>', '<string>'],
    },

    // Collection operations
    selige: {
        latin: 'selige',
        cpp: args =>
            `[&]{ std::random_device rd; std::mt19937 gen(rd()); std::uniform_int_distribution<size_t> dis(0, ${args[0]}.size() - 1); return ${args[0]}[dis(gen)]; }()`,
        headers: ['<random>'],
    },
    misce: {
        latin: 'misce',
        cpp: args => `[&]{ auto v = ${args[0]}; std::random_device rd; std::mt19937 gen(rd()); std::ranges::shuffle(v, gen); return v; }()`,
        headers: ['<random>', '<algorithm>'],
    },

    // Seeding
    // WHY: C++ requires explicit generator management. We use a static generator
    //      that can be seeded. This is a simplified approach - real code would
    //      need a global/thread-local generator.
    semen: {
        latin: 'semen',
        cpp: args => `/* seed(${args[0]}) - C++ requires explicit generator management */`,
        headers: [],
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

export function getAleatorFunction(name: string): AleatorEntry | undefined {
    return ALEATOR_FUNCTIONS[name];
}

export function getAleatorHeaders(name: string): string[] {
    return ALEATOR_FUNCTIONS[name]?.headers ?? [];
}

export function isAleatorFunction(name: string): boolean {
    return name in ALEATOR_FUNCTIONS;
}
