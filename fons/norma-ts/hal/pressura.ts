/**
 * pressura.ts - Compression Device Implementation
 *
 * Native TypeScript implementation of the HAL compression interface.
 * Uses Bun's built-in compression functions for gzip and deflate.
 *
 * Verbs:
 *   - comprime: compress (press together)
 *   - solve: decompress (loosen/release)
 *   - funde/hauri/claude: stream operations (pour/draw/close)
 */

type Algorithm = 'gzip' | 'deflate';

function validateAlgorithm(algo: string): Algorithm {
    if (algo === 'gzip' || algo === 'deflate') {
        return algo;
    }
    if (algo === 'brotli' || algo === 'zstd') {
        throw new Error(`Algorithm "${algo}" not yet supported in JS runtime`);
    }
    throw new Error(`Unknown algorithm: ${algo}`);
}

function combineChunks(chunks: Uint8Array[]): Uint8Array {
    const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
    const combined = new Uint8Array(totalLength);
    let offset = 0;
    for (const chunk of chunks) {
        combined.set(chunk, offset);
        offset += chunk.length;
    }
    return combined;
}

/**
 * Compressor stream
 */
export class Compressor {
    private algorithm: Algorithm;
    private level: number;
    private chunks: Uint8Array[] = [];

    constructor(algorithm: Algorithm, level?: number) {
        this.algorithm = algorithm;
        this.level = level !== undefined ? Math.max(1, Math.min(9, level)) : 6;
    }

    funde(data: Uint8Array): void {
        this.chunks.push(data);
    }

    claude(): Uint8Array {
        const combined = combineChunks(this.chunks);

        if (this.algorithm === 'gzip') {
            return Bun.gzipSync(combined, { level: this.level });
        }
        else {
            return Bun.deflateSync(combined, { level: this.level });
        }
    }
}

/**
 * Decompressor stream
 */
export class Decompressor {
    private algorithm: Algorithm;
    private chunks: Uint8Array[] = [];

    constructor(algorithm: Algorithm) {
        this.algorithm = algorithm;
    }

    funde(data: Uint8Array): void {
        this.chunks.push(data);
    }

    hauri(): Uint8Array {
        if (this.chunks.length === 0) {
            return new Uint8Array(0);
        }

        try {
            const combined = combineChunks(this.chunks);
            if (this.algorithm === 'gzip') {
                return Bun.gunzipSync(combined);
            }
            else {
                return Bun.inflateSync(combined);
            }
        }
        catch {
            // Incomplete stream - return empty
            return new Uint8Array(0);
        }
    }

    claude(): Uint8Array {
        const combined = combineChunks(this.chunks);

        if (this.algorithm === 'gzip') {
            return Bun.gunzipSync(combined);
        }
        else {
            return Bun.inflateSync(combined);
        }
    }
}

export const pressura = {
    // =========================================================================
    // ONE-SHOT OPERATIONS
    // =========================================================================

    comprime(algorithmus: string, data: Uint8Array, nivel?: number): Uint8Array {
        const algo = validateAlgorithm(algorithmus);
        const level = nivel !== undefined ? Math.max(1, Math.min(9, nivel)) : 6;

        if (algo === 'gzip') {
            return Bun.gzipSync(data, { level });
        }
        else {
            return Bun.deflateSync(data, { level });
        }
    },

    solve(algorithmus: string, data: Uint8Array): Uint8Array {
        const algo = validateAlgorithm(algorithmus);

        if (algo === 'gzip') {
            return Bun.gunzipSync(data);
        }
        else {
            return Bun.inflateSync(data);
        }
    },

    // =========================================================================
    // STREAMING
    // =========================================================================

    comprimens(algorithmus: string, nivel?: number): Compressor {
        const algo = validateAlgorithm(algorithmus);
        return new Compressor(algo, nivel);
    },

    solvens(algorithmus: string): Decompressor {
        const algo = validateAlgorithm(algorithmus);
        return new Decompressor(algo);
    },
};
