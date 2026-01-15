/**
 * tempus.ts - Clock and Timer Implementation
 *
 * Native TypeScript implementation of the HAL tempus interface.
 * Provides wall clock time, monotonic time, sleep, and scheduled callbacks.
 */

// Track initialization time for activum()
const initTime = Date.now();
const initHrTime = process.hrtime.bigint();

// Handle storage for timer cancellation
let nextHandleId = 1;
const timerHandles = new Map<number, ReturnType<typeof setTimeout> | ReturnType<typeof setInterval>>();

export const tempus = {
    // =========================================================================
    // WALL CLOCK (can jump forward/backward)
    // =========================================================================

    nunc(): number {
        return Date.now();
    },

    nuncNano(): bigint {
        // Convert milliseconds to nanoseconds, add high-res offset for precision
        const baseNano = BigInt(Date.now()) * 1_000_000n;
        const hrOffset = process.hrtime.bigint() - initHrTime;
        // Use only the sub-millisecond portion of hrtime
        return baseNano + (hrOffset % 1_000_000n);
    },

    nuncSecunda(): number {
        return Math.floor(Date.now() / 1000);
    },

    // =========================================================================
    // MONOTONIC CLOCK (never decreases)
    // =========================================================================

    monotonicum(): bigint {
        return process.hrtime.bigint();
    },

    activum(): number {
        return Date.now() - initTime;
    },

    // =========================================================================
    // SLEEP / DELAY
    // =========================================================================

    dormi(ms: number): Promise<void> {
        return new Promise((resolve) => setTimeout(resolve, ms));
    },

    // =========================================================================
    // SCHEDULED CALLBACKS
    // =========================================================================

    post(ms: number, fn: () => void): number {
        const id = nextHandleId++;
        const timer = setTimeout(() => {
            timerHandles.delete(id);
            fn();
        }, ms);
        timerHandles.set(id, timer);
        return id;
    },

    intervallum(ms: number, fn: () => void): number {
        const id = nextHandleId++;
        const timer = setInterval(fn, ms);
        timerHandles.set(id, timer);
        return id;
    },

    siste(handle: number): void {
        const timer = timerHandles.get(handle);
        if (timer) {
            clearTimeout(timer);
            clearInterval(timer);
            timerHandles.delete(handle);
        }
    },

    // =========================================================================
    // DURATION CONSTANTS (milliseconds)
    // =========================================================================

    MILLISECUNDUM(): number {
        return 1;
    },

    SECUNDUM(): number {
        return 1000;
    },

    MINUTUM(): number {
        return 60_000;
    },

    HORA(): number {
        return 3_600_000;
    },

    DIES(): number {
        return 86_400_000;
    },
};
