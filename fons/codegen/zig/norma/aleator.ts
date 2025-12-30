/**
 * Aleator Function Registry - Zig translations for Latin random functions
 *
 * COMPILER PHASE
 * ==============
 * codegen (Zig target)
 *
 * TARGET MAPPING
 * ==============
 * | Faber      | Zig                                            |
 * |------------|------------------------------------------------|
 * | fractus    | prng.random() (f64)                            |
 * | inter      | prng.intRangeAtMost(i64, min, max)             |
 * | octeti     | prng.bytes(&buf)                               |
 * | uuid       | (generate UUID v4 format)                      |
 * | selige     | items[prng.uintLessThan(usize, items.len)]     |
 * | misce      | std.rand.shuffle(T, items, prng)               |
 * | semen      | prng.seed(n)                                   |
 *
 * WHY: Zig uses std.rand with explicit PRNG instances.
 *      We use std.rand.DefaultPrng seeded with std.crypto.random.
 *
 * NOTE: Zig requires allocator for dynamic allocation. The curator parameter
 *       provides the current allocator context.
 */

// =============================================================================
// TYPES
// =============================================================================

export type AleatorGenerator = (args: string[], curator?: string) => string;

export interface AleatorEntry {
    latin: string;
    zig: string | AleatorGenerator;
}

// =============================================================================
// FUNCTION REGISTRY
// =============================================================================

export const ALEATOR_FUNCTIONS: Record<string, AleatorEntry> = {
    // Basic generation
    // WHY: Create PRNG inline seeded from crypto random. This is simple but
    //      creates new PRNG each call. Production code would use shared state.
    fractus: {
        latin: 'fractus',
        zig: () =>
            `blk: { var seed: u64 = undefined; std.crypto.random.bytes(std.mem.asBytes(&seed)); var prng = std.rand.DefaultPrng.init(seed); break :blk prng.random().float(f64); }`,
    },
    inter: {
        latin: 'inter',
        zig: args =>
            `blk: { var seed: u64 = undefined; std.crypto.random.bytes(std.mem.asBytes(&seed)); var prng = std.rand.DefaultPrng.init(seed); break :blk prng.random().intRangeAtMost(i64, ${args[0]}, ${args[1]}); }`,
    },
    octeti: {
        latin: 'octeti',
        zig: (args, curator) => {
            const alloc = curator || 'std.heap.page_allocator';
            return `blk: { var buf = ${alloc}.alloc(u8, ${args[0]}) catch unreachable; std.crypto.random.bytes(buf); break :blk buf; }`;
        },
    },
    uuid: {
        latin: 'uuid',
        // WHY: Generate UUID v4 format manually since Zig stdlib doesn't have uuid
        zig: () =>
            `blk: { var buf: [36]u8 = undefined; var bytes: [16]u8 = undefined; std.crypto.random.bytes(&bytes); bytes[6] = (bytes[6] & 0x0f) | 0x40; bytes[8] = (bytes[8] & 0x3f) | 0x80; const hex = "0123456789abcdef"; var i: usize = 0; for (bytes, 0..) |b, j| { if (j == 4 or j == 6 or j == 8 or j == 10) { buf[i] = '-'; i += 1; } buf[i] = hex[b >> 4]; buf[i + 1] = hex[b & 0x0f]; i += 2; } break :blk buf; }`,
    },

    // Collection operations
    selige: {
        latin: 'selige',
        zig: args =>
            `blk: { var seed: u64 = undefined; std.crypto.random.bytes(std.mem.asBytes(&seed)); var prng = std.rand.DefaultPrng.init(seed); break :blk ${args[0]}[prng.random().uintLessThan(usize, ${args[0]}.len)]; }`,
    },
    misce: {
        latin: 'misce',
        // WHY: Shuffle requires mutable slice. Clone first for functional semantics.
        zig: (args, curator) => {
            const alloc = curator || 'std.heap.page_allocator';
            return `blk: { var seed: u64 = undefined; std.crypto.random.bytes(std.mem.asBytes(&seed)); var prng = std.rand.DefaultPrng.init(seed); var copy = ${alloc}.dupe(@TypeOf(${args[0]}[0]), ${args[0]}) catch unreachable; prng.random().shuffle(@TypeOf(copy[0]), copy); break :blk copy; }`;
        },
    },

    // Seeding
    // WHY: Zig PRNGs require explicit seeding. This is a no-op since each call
    //      creates its own PRNG with crypto-random seed.
    semen: {
        latin: 'semen',
        zig: args => `@compileLog("seed(${args[0]}) - Zig requires explicit PRNG state management")`,
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
