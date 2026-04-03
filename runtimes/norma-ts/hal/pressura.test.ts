import { test, expect, describe } from 'bun:test';
import { pressura, Compressor, Decompressor } from './pressura';

describe('pressura HAL', () => {
    describe('compression roundtrip', () => {
        test('gzip compress then solve returns original', () => {
            const original = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            const compressed = pressura.comprime('gzip', original);
            const decompressed = pressura.solve('gzip', compressed);

            expect(decompressed).toEqual(original);
        });

        test('deflate compress then solve returns original', () => {
            const original = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            const compressed = pressura.comprime('deflate', original);
            const decompressed = pressura.solve('deflate', compressed);

            expect(decompressed).toEqual(original);
        });

        test('roundtrip with larger data', () => {
            const original = new Uint8Array(1000);
            for (let i = 0; i < original.length; i++) {
                original[i] = i % 256;
            }

            const gzipCompressed = pressura.comprime('gzip', original);
            const gzipDecompressed = pressura.solve('gzip', gzipCompressed);
            expect(gzipDecompressed).toEqual(original);

            const deflateCompressed = pressura.comprime('deflate', original);
            const deflateDecompressed = pressura.solve('deflate', deflateCompressed);
            expect(deflateDecompressed).toEqual(original);
        });
    });

    describe('text compression (manual encode/decode)', () => {
        test('text roundtrip with gzip', () => {
            const original = 'Hello, world! This is a test string for compression.';
            const encoded = new TextEncoder().encode(original);
            const compressed = pressura.comprime('gzip', encoded);
            const decompressed = pressura.solve('gzip', compressed);
            const decoded = new TextDecoder().decode(decompressed);

            expect(decoded).toBe(original);
        });

        test('text roundtrip with deflate', () => {
            const original = 'Lorem ipsum dolor sit amet, consectetur adipiscing elit.';
            const encoded = new TextEncoder().encode(original);
            const compressed = pressura.comprime('deflate', encoded);
            const decompressed = pressura.solve('deflate', compressed);
            const decoded = new TextDecoder().decode(decompressed);

            expect(decoded).toBe(original);
        });

        test('handles unicode text', () => {
            const original = 'Hello world';
            const encoded = new TextEncoder().encode(original);
            const compressed = pressura.comprime('gzip', encoded);
            const decompressed = pressura.solve('gzip', compressed);
            const decoded = new TextDecoder().decode(decompressed);

            expect(decoded).toBe(original);
        });
    });

    describe('compression levels', () => {
        test('comprime accepts level parameter', () => {
            const data = new Uint8Array(100);
            data.fill(65); // Fill with 'A' - highly compressible

            const compressed1 = pressura.comprime('gzip', data, 1);
            const compressed9 = pressura.comprime('gzip', data, 9);

            // Both should decompress to original
            expect(pressura.solve('gzip', compressed1)).toEqual(data);
            expect(pressura.solve('gzip', compressed9)).toEqual(data);

            // Higher level should compress at least as well (often better)
            expect(compressed9.length).toBeLessThanOrEqual(compressed1.length);
        });

        test('level is clamped to valid range', () => {
            const data = new Uint8Array([1, 2, 3, 4, 5]);

            // Should not throw for out-of-range levels
            const compressed0 = pressura.comprime('gzip', data, 0);
            const compressed10 = pressura.comprime('gzip', data, 10);

            expect(pressura.solve('gzip', compressed0)).toEqual(data);
            expect(pressura.solve('gzip', compressed10)).toEqual(data);
        });
    });

    describe('compression effectiveness', () => {
        test('compressible data reduces in size', () => {
            // Create highly compressible data (repeated pattern)
            const original = new Uint8Array(10000);
            for (let i = 0; i < original.length; i++) {
                original[i] = i % 4; // Only 4 different values
            }

            const gzipCompressed = pressura.comprime('gzip', original);
            const deflateCompressed = pressura.comprime('deflate', original);

            expect(gzipCompressed.length).toBeLessThan(original.length);
            expect(deflateCompressed.length).toBeLessThan(original.length);
        });

        test('text compresses significantly', () => {
            const text = 'The quick brown fox jumps over the lazy dog. '.repeat(100);
            const original = new TextEncoder().encode(text);
            const compressed = pressura.comprime('gzip', original);

            // Should achieve significant compression on repeated text
            expect(compressed.length).toBeLessThan(original.length / 2);
        });
    });

    describe('unsupported algorithms', () => {
        test('brotli throws not supported', () => {
            const data = new Uint8Array([1, 2, 3]);
            expect(() => pressura.comprime('brotli', data)).toThrow('not yet supported');
        });

        test('zstd throws not supported', () => {
            const data = new Uint8Array([1, 2, 3]);
            expect(() => pressura.comprime('zstd', data)).toThrow('not yet supported');
        });

        test('unknown algorithm throws', () => {
            const data = new Uint8Array([1, 2, 3]);
            expect(() => pressura.comprime('lz4', data)).toThrow('Unknown algorithm: lz4');
        });
    });

    describe('streaming compressor', () => {
        test('Compressor accumulates and compresses', () => {
            const compressor = pressura.comprimens('gzip');

            compressor.funde(new Uint8Array([1, 2, 3]));
            compressor.funde(new Uint8Array([4, 5, 6]));
            compressor.funde(new Uint8Array([7, 8, 9]));

            const compressed = compressor.claude();
            const decompressed = pressura.solve('gzip', compressed);

            expect(decompressed).toEqual(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9]));
        });

        test('Compressor works with deflate', () => {
            const compressor = pressura.comprimens('deflate');

            compressor.funde(new Uint8Array([10, 20, 30]));
            compressor.funde(new Uint8Array([40, 50]));

            const compressed = compressor.claude();
            const decompressed = pressura.solve('deflate', compressed);

            expect(decompressed).toEqual(new Uint8Array([10, 20, 30, 40, 50]));
        });

        test('Compressor with level parameter', () => {
            const compressor = pressura.comprimens('gzip', 9);
            compressor.funde(new Uint8Array([1, 2, 3]));
            const result = compressor.claude();

            expect(result).toBeInstanceOf(Uint8Array);
            expect(result.length).toBeGreaterThan(0);
        });
    });

    describe('streaming decompressor', () => {
        test('Decompressor accumulates and decompresses', () => {
            const original = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            const compressed = pressura.comprime('gzip', original);

            const decompressor = pressura.solvens('gzip');
            decompressor.funde(compressed);

            const result = decompressor.claude();
            expect(result).toEqual(original);
        });

        test('Decompressor hauri returns partial output', () => {
            const original = new Uint8Array([1, 2, 3, 4, 5]);
            const compressed = pressura.comprime('deflate', original);

            const decompressor = pressura.solvens('deflate');
            decompressor.funde(compressed);

            // hauri should return decompressed data
            const partial = decompressor.hauri();
            expect(partial).toEqual(original);

            // claude should also work
            const final = decompressor.claude();
            expect(final).toEqual(original);
        });

        test('Decompressor hauri returns empty on incomplete data', () => {
            const decompressor = pressura.solvens('gzip');

            // Add incomplete compressed data
            decompressor.funde(new Uint8Array([31, 139])); // gzip magic bytes only

            // hauri should return empty (can't decompress incomplete stream)
            const partial = decompressor.hauri();
            expect(partial.length).toBe(0);
        });
    });

    describe('edge cases', () => {
        test('empty data roundtrip', () => {
            const empty = new Uint8Array(0);

            const gzipCompressed = pressura.comprime('gzip', empty);
            const gzipDecompressed = pressura.solve('gzip', gzipCompressed);
            expect(gzipDecompressed).toEqual(empty);

            const deflateCompressed = pressura.comprime('deflate', empty);
            const deflateDecompressed = pressura.solve('deflate', deflateCompressed);
            expect(deflateDecompressed).toEqual(empty);
        });

        test('empty text roundtrip', () => {
            const encoded = new TextEncoder().encode('');
            const compressed = pressura.comprime('gzip', encoded);
            const decompressed = pressura.solve('gzip', compressed);
            const decoded = new TextDecoder().decode(decompressed);
            expect(decoded).toBe('');
        });

        test('single byte roundtrip', () => {
            const single = new Uint8Array([42]);
            const compressed = pressura.comprime('deflate', single);
            const decompressed = pressura.solve('deflate', compressed);
            expect(decompressed).toEqual(single);
        });
    });
});
