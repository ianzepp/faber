/**
 * pressura.ts - Compression Device Implementation
 *
 * Native TypeScript implementation of the HAL compression interface.
 * Uses Bun's built-in compression functions for gzip and deflate.
 */

type Algorithm = 'gzip' | 'deflate' | 'brotli' | 'zstd';

function validateAlgorithm(algo: string): Algorithm {
    if (algo === 'gzip' || algo === 'deflate') {
        return algo;
    }
    if (algo === 'brotli' || algo === 'zstd') {
        throw new Error(`Algorithm "${algo}" not supported`);
    }
    throw new Error(`Unknown algorithm: ${algo}`);
}

/**
 * Compressor stream - accumulates data and compresses on finish
 */
export class Compressor {
    private algorithm: Algorithm;
    private chunks: Uint8Array[] = [];

    constructor(algorithm: Algorithm) {
        this.algorithm = algorithm;
    }

    adde(data: Uint8Array): void {
        this.chunks.push(data);
    }

    fini(): Uint8Array {
        // Concatenate all chunks
        const totalLength = this.chunks.reduce((sum, chunk) => sum + chunk.length, 0);
        const combined = new Uint8Array(totalLength);
        let offset = 0;
        for (const chunk of this.chunks) {
            combined.set(chunk, offset);
            offset += chunk.length;
        }

        // Compress the combined data
        if (this.algorithm === 'gzip') {
            return Bun.gzipSync(combined);
        }
        else {
            return Bun.deflateSync(combined);
        }
    }
}

/**
 * Decompressor stream - accumulates compressed data and decompresses
 */
export class Decompressor {
    private algorithm: Algorithm;
    private chunks: Uint8Array[] = [];

    constructor(algorithm: Algorithm) {
        this.algorithm = algorithm;
    }

    adde(data: Uint8Array): void {
        this.chunks.push(data);
    }

    cape(): Uint8Array {
        // Return partial decompression (attempt to decompress what we have)
        // WHY: Partial decompression may fail if stream is incomplete,
        // so we return empty if decompression fails
        if (this.chunks.length === 0) {
            return new Uint8Array(0);
        }

        try {
            const combined = this.combineChunks();
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

    fini(): Uint8Array {
        const combined = this.combineChunks();

        if (this.algorithm === 'gzip') {
            return Bun.gunzipSync(combined);
        }
        else {
            return Bun.inflateSync(combined);
        }
    }

    private combineChunks(): Uint8Array {
        const totalLength = this.chunks.reduce((sum, chunk) => sum + chunk.length, 0);
        const combined = new Uint8Array(totalLength);
        let offset = 0;
        for (const chunk of this.chunks) {
            combined.set(chunk, offset);
            offset += chunk.length;
        }
        return combined;
    }
}

export const pressura = {
    // =========================================================================
    // COMPRESSION
    // =========================================================================

    comprime(algorithmus: string, data: Uint8Array): Uint8Array {
        const algo = validateAlgorithm(algorithmus);

        if (algo === 'gzip') {
            return Bun.gzipSync(data);
        }
        else {
            return Bun.deflateSync(data);
        }
    },

    comprimeNivel(algorithmus: string, data: Uint8Array, nivel: number): Uint8Array {
        const algo = validateAlgorithm(algorithmus);
        const level = Math.max(1, Math.min(9, nivel));

        if (algo === 'gzip') {
            return Bun.gzipSync(data, { level });
        }
        else {
            return Bun.deflateSync(data, { level });
        }
    },

    comprimeTextum(algorithmus: string, text: string): Uint8Array {
        const encoder = new TextEncoder();
        const data = encoder.encode(text);
        return this.comprime(algorithmus, data);
    },

    // =========================================================================
    // DECOMPRESSION
    // =========================================================================

    decomprimen(algorithmus: string, data: Uint8Array): Uint8Array {
        const algo = validateAlgorithm(algorithmus);

        if (algo === 'gzip') {
            return Bun.gunzipSync(data);
        }
        else {
            return Bun.inflateSync(data);
        }
    },

    decomprimeTextum(algorithmus: string, data: Uint8Array): string {
        const decompressed = this.decomprimen(algorithmus, data);
        const decoder = new TextDecoder();
        return decoder.decode(decompressed);
    },

    // =========================================================================
    // STREAMING
    // =========================================================================

    comprimens(algorithmus: string): Compressor {
        const algo = validateAlgorithm(algorithmus);
        return new Compressor(algo);
    },

    decomprimens(algorithmus: string): Decompressor {
        const algo = validateAlgorithm(algorithmus);
        return new Decompressor(algo);
    },
};
