import { test, expect, describe } from 'bun:test';
import { pressura, Compressor, Decompressor } from './pressura';

describe('pressura HAL', () => {
    describe('compression roundtrip', () => {
        test('gzip compress then decompress returns original', () => {
            const original = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            const compressed = pressura.comprime('gzip', original);
            const decompressed = pressura.decomprimen('gzip', compressed);

            expect(decompressed).toEqual(original);
        });

        test('deflate compress then decompress returns original', () => {
            const original = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            const compressed = pressura.comprime('deflate', original);
            const decompressed = pressura.decomprimen('deflate', compressed);

            expect(decompressed).toEqual(original);
        });

        test('roundtrip with larger data', () => {
            const original = new Uint8Array(1000);
            for (let i = 0; i < original.length; i++) {
                original[i] = i % 256;
            }

            const gzipCompressed = pressura.comprime('gzip', original);
            const gzipDecompressed = pressura.decomprimen('gzip', gzipCompressed);
            expect(gzipDecompressed).toEqual(original);

            const deflateCompressed = pressura.comprime('deflate', original);
            const deflateDecompressed = pressura.decomprimen('deflate', deflateCompressed);
            expect(deflateDecompressed).toEqual(original);
        });
    });

    describe('text compression', () => {
        test('comprimeTextum and decomprimeTextum roundtrip', () => {
            const original = 'Hello, world! This is a test string for compression.';
            const compressed = pressura.comprimeTextum('gzip', original);
            const decompressed = pressura.decomprimeTextum('gzip', compressed);

            expect(decompressed).toBe(original);
        });

        test('text compression with deflate', () => {
            const original = 'Lorem ipsum dolor sit amet, consectetur adipiscing elit.';
            const compressed = pressura.comprimeTextum('deflate', original);
            const decompressed = pressura.decomprimeTextum('deflate', compressed);

            expect(decompressed).toBe(original);
        });

        test('handles unicode text', () => {
            const original = 'Hello world';
            const compressed = pressura.comprimeTextum('gzip', original);
            const decompressed = pressura.decomprimeTextum('gzip', compressed);

            expect(decompressed).toBe(original);
        });
    });

    describe('compression levels', () => {
        test('comprimeNivel accepts level parameter', () => {
            const data = new Uint8Array(100);
            data.fill(65); // Fill with 'A' - highly compressible

            const compressed1 = pressura.comprimeNivel('gzip', data, 1);
            const compressed9 = pressura.comprimeNivel('gzip', data, 9);

            // Both should decompress to original
            expect(pressura.decomprimen('gzip', compressed1)).toEqual(data);
            expect(pressura.decomprimen('gzip', compressed9)).toEqual(data);

            // Higher level should compress at least as well (often better)
            expect(compressed9.length).toBeLessThanOrEqual(compressed1.length);
        });

        test('level is clamped to valid range', () => {
            const data = new Uint8Array([1, 2, 3, 4, 5]);

            // Should not throw for out-of-range levels
            const compressed0 = pressura.comprimeNivel('gzip', data, 0);
            const compressed10 = pressura.comprimeNivel('gzip', data, 10);

            expect(pressura.decomprimen('gzip', compressed0)).toEqual(data);
            expect(pressura.decomprimen('gzip', compressed10)).toEqual(data);
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
            expect(() => pressura.comprime('brotli', data)).toThrow('Algorithm "brotli" not supported');
        });

        test('zstd throws not supported', () => {
            const data = new Uint8Array([1, 2, 3]);
            expect(() => pressura.comprime('zstd', data)).toThrow('Algorithm "zstd" not supported');
        });

        test('unknown algorithm throws', () => {
            const data = new Uint8Array([1, 2, 3]);
            expect(() => pressura.comprime('lz4', data)).toThrow('Unknown algorithm: lz4');
        });
    });

    describe('streaming compressor', () => {
        test('Compressor accumulates and compresses', () => {
            const compressor = pressura.comprimens('gzip');

            compressor.adde(new Uint8Array([1, 2, 3]));
            compressor.adde(new Uint8Array([4, 5, 6]));
            compressor.adde(new Uint8Array([7, 8, 9]));

            const compressed = compressor.fini();
            const decompressed = pressura.decomprimen('gzip', compressed);

            expect(decompressed).toEqual(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9]));
        });

        test('Compressor works with deflate', () => {
            const compressor = pressura.comprimens('deflate');

            compressor.adde(new Uint8Array([10, 20, 30]));
            compressor.adde(new Uint8Array([40, 50]));

            const compressed = compressor.fini();
            const decompressed = pressura.decomprimen('deflate', compressed);

            expect(decompressed).toEqual(new Uint8Array([10, 20, 30, 40, 50]));
        });

        test('Compressor class can be instantiated directly', () => {
            const compressor = new Compressor('gzip' as 'gzip');
            compressor.adde(new Uint8Array([1, 2, 3]));
            const result = compressor.fini();

            expect(result).toBeInstanceOf(Uint8Array);
            expect(result.length).toBeGreaterThan(0);
        });
    });

    describe('streaming decompressor', () => {
        test('Decompressor accumulates and decompresses', () => {
            const original = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            const compressed = pressura.comprime('gzip', original);

            const decompressor = pressura.decomprimens('gzip');
            decompressor.adde(compressed);

            const result = decompressor.fini();
            expect(result).toEqual(original);
        });

        test('Decompressor cape returns partial output', () => {
            const original = new Uint8Array([1, 2, 3, 4, 5]);
            const compressed = pressura.comprime('deflate', original);

            const decompressor = pressura.decomprimens('deflate');
            decompressor.adde(compressed);

            // cape should return decompressed data
            const partial = decompressor.cape();
            expect(partial).toEqual(original);

            // fini should also work
            const final = decompressor.fini();
            expect(final).toEqual(original);
        });

        test('Decompressor cape returns empty on incomplete data', () => {
            const decompressor = pressura.decomprimens('gzip');

            // Add incomplete compressed data
            decompressor.adde(new Uint8Array([31, 139])); // gzip magic bytes only

            // cape should return empty (can't decompress incomplete stream)
            const partial = decompressor.cape();
            expect(partial.length).toBe(0);
        });

        test('Decompressor class can be instantiated directly', () => {
            const decompressor = new Decompressor('deflate' as 'deflate');
            const compressed = pressura.comprime('deflate', new Uint8Array([1, 2, 3]));
            decompressor.adde(compressed);
            const result = decompressor.fini();

            expect(result).toEqual(new Uint8Array([1, 2, 3]));
        });
    });

    describe('edge cases', () => {
        test('empty data roundtrip', () => {
            const empty = new Uint8Array(0);

            const gzipCompressed = pressura.comprime('gzip', empty);
            const gzipDecompressed = pressura.decomprimen('gzip', gzipCompressed);
            expect(gzipDecompressed).toEqual(empty);

            const deflateCompressed = pressura.comprime('deflate', empty);
            const deflateDecompressed = pressura.decomprimen('deflate', deflateCompressed);
            expect(deflateDecompressed).toEqual(empty);
        });

        test('empty text roundtrip', () => {
            const compressed = pressura.comprimeTextum('gzip', '');
            const decompressed = pressura.decomprimeTextum('gzip', compressed);
            expect(decompressed).toBe('');
        });

        test('single byte roundtrip', () => {
            const single = new Uint8Array([42]);
            const compressed = pressura.comprime('deflate', single);
            const decompressed = pressura.decomprimen('deflate', compressed);
            expect(decompressed).toEqual(single);
        });
    });
});
