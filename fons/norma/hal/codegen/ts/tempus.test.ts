import { test, expect, describe } from 'bun:test';
import { tempus } from './tempus';

describe('tempus HAL', () => {
    describe('wall clock', () => {
        test('nunc returns current epoch milliseconds', () => {
            const before = Date.now();
            const result = tempus.nunc();
            const after = Date.now();

            expect(result).toBeGreaterThanOrEqual(before);
            expect(result).toBeLessThanOrEqual(after);
        });

        test('nuncNano returns nanoseconds', () => {
            const result = tempus.nuncNano();

            expect(typeof result).toBe('bigint');
            // Should be roughly current epoch in nanoseconds (13+ digits after ~2001)
            expect(result).toBeGreaterThan(1_000_000_000_000_000_000n);
        });

        test('nuncSecunda returns current epoch seconds', () => {
            const result = tempus.nuncSecunda();
            const expected = Math.floor(Date.now() / 1000);

            // Allow 1 second tolerance for execution time
            expect(result).toBeGreaterThanOrEqual(expected - 1);
            expect(result).toBeLessThanOrEqual(expected + 1);
        });
    });

    describe('monotonic clock', () => {
        test('monotonicum returns monotonically increasing bigint', () => {
            const first = tempus.monotonicum();
            const second = tempus.monotonicum();
            const third = tempus.monotonicum();

            expect(typeof first).toBe('bigint');
            expect(second).toBeGreaterThanOrEqual(first);
            expect(third).toBeGreaterThanOrEqual(second);
        });

        test('activum returns non-negative uptime', () => {
            const result = tempus.activum();

            expect(typeof result).toBe('number');
            expect(result).toBeGreaterThanOrEqual(0);
        });

        test('activum increases over time', async () => {
            const first = tempus.activum();
            await tempus.dormi(10);
            const second = tempus.activum();

            expect(second).toBeGreaterThan(first);
        });
    });

    describe('sleep', () => {
        test('dormi delays execution', async () => {
            const start = tempus.nunc();
            await tempus.dormi(50);
            const elapsed = tempus.nunc() - start;

            // Should have waited at least 50ms (allow some tolerance)
            expect(elapsed).toBeGreaterThanOrEqual(45);
        });

        test('dormi returns a promise', () => {
            const result = tempus.dormi(1);
            expect(result).toBeInstanceOf(Promise);
        });
    });

    describe('scheduled callbacks', () => {
        test('post fires once after delay', async () => {
            let callCount = 0;
            const handle = tempus.post(20, () => {
                callCount++;
            });

            expect(typeof handle).toBe('number');
            expect(callCount).toBe(0);

            await tempus.dormi(50);
            expect(callCount).toBe(1);

            // Should not fire again
            await tempus.dormi(50);
            expect(callCount).toBe(1);
        });

        test('intervallum fires repeatedly', async () => {
            let callCount = 0;
            const handle = tempus.intervallum(15, () => {
                callCount++;
            });

            expect(typeof handle).toBe('number');

            await tempus.dormi(80);
            tempus.siste(handle);

            // Should have fired multiple times (roughly 80/15 = 5 times, but timing varies)
            expect(callCount).toBeGreaterThanOrEqual(3);
        });

        test('siste cancels post callback', async () => {
            let called = false;
            const handle = tempus.post(30, () => {
                called = true;
            });

            tempus.siste(handle);
            await tempus.dormi(60);

            expect(called).toBe(false);
        });

        test('siste cancels intervallum callback', async () => {
            let callCount = 0;
            const handle = tempus.intervallum(10, () => {
                callCount++;
            });

            await tempus.dormi(35);
            const countBeforeStop = callCount;
            tempus.siste(handle);

            await tempus.dormi(50);
            // Count should not have increased after stopping
            expect(callCount).toBe(countBeforeStop);
        });
    });

    describe('duration constants', () => {
        test('MILLISECUNDUM is 1', () => {
            expect(tempus.MILLISECUNDUM()).toBe(1);
        });

        test('SECUNDUM is 1000', () => {
            expect(tempus.SECUNDUM()).toBe(1000);
        });

        test('MINUTUM is 60000', () => {
            expect(tempus.MINUTUM()).toBe(60_000);
        });

        test('HORA is 3600000', () => {
            expect(tempus.HORA()).toBe(3_600_000);
        });

        test('DIES is 86400000', () => {
            expect(tempus.DIES()).toBe(86_400_000);
        });

        test('duration constants are consistent', () => {
            expect(tempus.SECUNDUM()).toBe(1000 * tempus.MILLISECUNDUM());
            expect(tempus.MINUTUM()).toBe(60 * tempus.SECUNDUM());
            expect(tempus.HORA()).toBe(60 * tempus.MINUTUM());
            expect(tempus.DIES()).toBe(24 * tempus.HORA());
        });
    });
});
