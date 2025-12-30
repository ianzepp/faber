/**
 * Aleator Function Registry - Rust translations for Latin random functions
 *
 * COMPILER PHASE
 * ==============
 * codegen (Rust target)
 *
 * TARGET MAPPING
 * ==============
 * | Faber      | Rust (using rand crate)                |
 * |------------|----------------------------------------|
 * | fractus    | rand::random::<f64>()                  |
 * | inter      | rand::thread_rng().gen_range(min..=max)|
 * | octeti     | rand::thread_rng().gen::<[u8; N]>()    |
 * | uuid       | uuid::Uuid::new_v4().to_string()       |
 * | selige     | vec.choose(&mut rand::thread_rng())    |
 * | misce      | vec.shuffle(&mut rand::thread_rng())   |
 * | semen      | (no-op - rand uses thread_rng)         |
 *
 * WHY: Rust's standard library doesn't include random number generation.
 *      The `rand` crate is the de facto standard. For UUID, use `uuid` crate.
 *
 * CRATE DEPENDENCIES:
 *   rand = "0.8"
 *   uuid = { version = "1", features = ["v4"] }
 */

// =============================================================================
// TYPES
// =============================================================================

export type AleatorGenerator = (args: string[]) => string;

export interface AleatorEntry {
    latin: string;
    rs: string | AleatorGenerator;
}

// =============================================================================
// FUNCTION REGISTRY
// =============================================================================

export const ALEATOR_FUNCTIONS: Record<string, AleatorEntry> = {
    // Basic generation
    fractus: {
        latin: 'fractus',
        rs: () => `rand::random::<f64>()`,
    },
    inter: {
        latin: 'inter',
        rs: args => `rand::thread_rng().gen_range(${args[0]}..=${args[1]})`,
    },
    octeti: {
        latin: 'octeti',
        // WHY: Generate a Vec<u8> of random bytes
        rs: args => `{ let mut bytes = vec![0u8; ${args[0]}]; rand::thread_rng().fill(&mut bytes[..]); bytes }`,
    },
    uuid: {
        latin: 'uuid',
        rs: () => `uuid::Uuid::new_v4().to_string()`,
    },

    // Collection operations
    selige: {
        latin: 'selige',
        // WHY: choose() returns Option<&T>, unwrap and clone for owned value
        rs: args => `${args[0]}.choose(&mut rand::thread_rng()).unwrap().clone()`,
    },
    misce: {
        latin: 'misce',
        // WHY: shuffle() is in-place, so clone first to return new vec
        rs: args => `{ let mut v = ${args[0]}.clone(); v.shuffle(&mut rand::thread_rng()); v }`,
    },

    // Seeding
    // WHY: rand's thread_rng is automatically seeded. For reproducible results,
    //      users would need to use StdRng::seed_from_u64() explicitly.
    semen: {
        latin: 'semen',
        rs: args => `/* seed(${args[0]}) - use rand::SeedableRng for reproducible results */`,
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

export function getAleatorFunction(name: string): AleatorEntry | undefined {
    return ALEATOR_FUNCTIONS[name];
}

export function isAleatorFunction(name: string): boolean {
    return name in ALEATOR_FUNCTIONS;
}
