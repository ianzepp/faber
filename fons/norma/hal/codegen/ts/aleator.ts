/**
 * aleator.ts - Entropy Device Implementation (Bun/Node)
 *
 * Native TypeScript implementation of the HAL entropy interface.
 *
 * Design:
 *   - fractum/sortire use Math.random by default (fast, seedable for tests)
 *   - octetos/uuid use crypto.getRandomValues (always secure, not seedable)
 *   - semina seeds the RNG for reproducible tests
 */

// Seedable RNG state
let seededRng: (() => number) | null = null;

// Mulberry32 PRNG (fast, good distribution, 32-bit state)
function mulberry32(seed: number): () => number {
    return () => {
        let t = (seed += 0x6d2b79f5);
        t = Math.imul(t ^ (t >>> 15), t | 1);
        t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
}

// Get current RNG (seeded or Math.random)
function rng(): number {
    return seededRng ? seededRng() : Math.random();
}

export const aleator = {
    // =========================================================================
    // BASIC GENERATION
    // =========================================================================

    fractum(): number {
        return rng();
    },

    sortire(min: number, max: number): number {
        return Math.floor(rng() * (max - min + 1)) + min;
    },

    // =========================================================================
    // CRYPTOGRAPHIC
    // =========================================================================

    octetos(n: number): Uint8Array {
        const buffer = new Uint8Array(n);
        crypto.getRandomValues(buffer);
        return buffer;
    },

    uuid(): string {
        return crypto.randomUUID();
    },

    // =========================================================================
    // SEEDING
    // =========================================================================

    semina(n: number): void {
        if (n <= 0) {
            seededRng = null;
        }
        else {
            seededRng = mulberry32(n);
        }
    },
};
