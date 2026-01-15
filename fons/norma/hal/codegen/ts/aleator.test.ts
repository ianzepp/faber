import { test, expect, describe, beforeEach } from 'bun:test';
import { aleator } from './aleator';

describe('aleator HAL', () => {
    beforeEach(() => {
        // Reset to true random before each test
        aleator.semen(0);
    });

    describe('fractus', () => {
        test('returns value in [0, 1)', () => {
            for (let i = 0; i < 100; i++) {
                const val = aleator.fractus();
                expect(val).toBeGreaterThanOrEqual(0);
                expect(val).toBeLessThan(1);
            }
        });
    });

    describe('inter', () => {
        test('returns value in [min, max] inclusive', () => {
            for (let i = 0; i < 100; i++) {
                const val = aleator.inter(5, 10);
                expect(val).toBeGreaterThanOrEqual(5);
                expect(val).toBeLessThanOrEqual(10);
                expect(Number.isInteger(val)).toBe(true);
            }
        });

        test('handles single value range', () => {
            for (let i = 0; i < 10; i++) {
                expect(aleator.inter(7, 7)).toBe(7);
            }
        });

        test('handles negative ranges', () => {
            for (let i = 0; i < 50; i++) {
                const val = aleator.inter(-10, -5);
                expect(val).toBeGreaterThanOrEqual(-10);
                expect(val).toBeLessThanOrEqual(-5);
            }
        });
    });

    describe('octeti', () => {
        test('returns n random bytes', () => {
            const bytes = aleator.octeti(16);
            expect(bytes).toBeInstanceOf(Uint8Array);
            expect(bytes.length).toBe(16);
        });

        test('returns empty array for n=0', () => {
            const bytes = aleator.octeti(0);
            expect(bytes.length).toBe(0);
        });

        test('bytes are in valid range [0, 255]', () => {
            const bytes = aleator.octeti(100);
            for (const b of bytes) {
                expect(b).toBeGreaterThanOrEqual(0);
                expect(b).toBeLessThanOrEqual(255);
            }
        });
    });

    describe('uuid', () => {
        test('returns valid UUID v4 format', () => {
            const uuid = aleator.uuid();
            // UUID v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
            const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
            expect(uuid).toMatch(uuidRegex);
        });

        test('generates unique UUIDs', () => {
            const uuids = new Set<string>();
            for (let i = 0; i < 100; i++) {
                uuids.add(aleator.uuid());
            }
            expect(uuids.size).toBe(100);
        });
    });

    describe('semen (seeding)', () => {
        test('same seed produces same fractus sequence', () => {
            aleator.semen(12345);
            const seq1 = [aleator.fractus(), aleator.fractus(), aleator.fractus()];

            aleator.semen(12345);
            const seq2 = [aleator.fractus(), aleator.fractus(), aleator.fractus()];

            expect(seq1).toEqual(seq2);
        });

        test('same seed produces same inter sequence', () => {
            aleator.semen(42);
            const seq1 = [aleator.inter(0, 100), aleator.inter(0, 100), aleator.inter(0, 100)];

            aleator.semen(42);
            const seq2 = [aleator.inter(0, 100), aleator.inter(0, 100), aleator.inter(0, 100)];

            expect(seq1).toEqual(seq2);
        });

        test('different seeds produce different sequences', () => {
            aleator.semen(111);
            const seq1 = [aleator.fractus(), aleator.fractus(), aleator.fractus()];

            aleator.semen(222);
            const seq2 = [aleator.fractus(), aleator.fractus(), aleator.fractus()];

            expect(seq1).not.toEqual(seq2);
        });

        test('semen(0) resets to true random', () => {
            aleator.semen(12345);
            aleator.fractus();

            aleator.semen(0);
            // After reset, we can't predict values, but we can verify it works
            const val = aleator.fractus();
            expect(val).toBeGreaterThanOrEqual(0);
            expect(val).toBeLessThan(1);
        });

        test('negative seed also resets to true random', () => {
            aleator.semen(12345);
            aleator.fractus();

            aleator.semen(-1);
            const val = aleator.fractus();
            expect(val).toBeGreaterThanOrEqual(0);
            expect(val).toBeLessThan(1);
        });
    });
});
