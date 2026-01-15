/**
 * nuncius.ts - Inter-Process Communication Implementation
 *
 * Native TypeScript implementation of the HAL IPC interface.
 * Uses MessageChannel for ports and Atomics for synchronization primitives.
 */

// =============================================================================
// MESSAGE PORT PAIR
// =============================================================================

export class ParPortarum {
    private portA: Porta;
    private portB: Porta;

    constructor() {
        const channel = new MessageChannel();
        this.portA = new Porta(channel.port1);
        this.portB = new Porta(channel.port2);
    }

    a(): Porta {
        return this.portA;
    }

    b(): Porta {
        return this.portB;
    }
}

// =============================================================================
// MESSAGE PORT
// =============================================================================

export class Porta {
    private port: MessagePort;
    private messageQueue: unknown[] = [];
    private pendingResolvers: Array<{ resolve: (value: unknown) => void; reject: (err: Error) => void }> = [];
    private closed = false;

    constructor(port: MessagePort) {
        this.port = port;
        this.port.onmessage = (event) => {
            if (this.pendingResolvers.length > 0) {
                const { resolve } = this.pendingResolvers.shift()!;
                resolve(event.data);
            }
            else {
                this.messageQueue.push(event.data);
            }
        };
        this.port.start();
    }

    mitte(nuntius: unknown): void {
        if (this.closed) {
            throw new Error('Port is closed');
        }
        this.port.postMessage(nuntius);
    }

    async recipe(): Promise<unknown> {
        if (this.closed) {
            throw new Error('Port is closed');
        }
        if (this.messageQueue.length > 0) {
            return this.messageQueue.shift();
        }
        return new Promise((resolve, reject) => {
            this.pendingResolvers.push({ resolve, reject });
        });
    }

    paratum(): boolean {
        return this.messageQueue.length > 0;
    }

    claude(): void {
        this.closed = true;
        this.port.close();
        // Reject all pending receivers
        for (const { reject } of this.pendingResolvers) {
            reject(new Error('Port is closed'));
        }
        this.pendingResolvers = [];
    }
}

// =============================================================================
// MUTEX
// =============================================================================

export class Mutex {
    private view: Int32Array;
    private index: number;

    // Lock states: 0 = unlocked, 1 = locked
    private static readonly UNLOCKED = 0;
    private static readonly LOCKED = 1;

    constructor(memoria: Uint8Array, offset: number) {
        // Ensure proper alignment for Int32Array
        const buffer = memoria.buffer;
        const byteOffset = memoria.byteOffset + offset;
        this.view = new Int32Array(buffer, byteOffset, 1);
        this.index = 0;
    }

    obstringe(): void {
        while (true) {
            const oldValue = Atomics.compareExchange(
                this.view,
                this.index,
                Mutex.UNLOCKED,
                Mutex.LOCKED
            );
            if (oldValue === Mutex.UNLOCKED) {
                return; // Successfully acquired
            }
            // Wait for unlock notification
            Atomics.wait(this.view, this.index, Mutex.LOCKED);
        }
    }

    tempta(): boolean {
        const oldValue = Atomics.compareExchange(
            this.view,
            this.index,
            Mutex.UNLOCKED,
            Mutex.LOCKED
        );
        return oldValue === Mutex.UNLOCKED;
    }

    solve(): void {
        Atomics.store(this.view, this.index, Mutex.UNLOCKED);
        Atomics.notify(this.view, this.index, 1);
    }
}

// =============================================================================
// SEMAPHORE
// =============================================================================

export class Semaphorum {
    private view: Int32Array;
    private index: number;

    constructor(memoria: Uint8Array, offset: number, valor: number) {
        const buffer = memoria.buffer;
        const byteOffset = memoria.byteOffset + offset;
        this.view = new Int32Array(buffer, byteOffset, 1);
        this.index = 0;
        Atomics.store(this.view, this.index, valor);
    }

    exspecta(): void {
        while (true) {
            const current = Atomics.load(this.view, this.index);
            if (current > 0) {
                const oldValue = Atomics.compareExchange(
                    this.view,
                    this.index,
                    current,
                    current - 1
                );
                if (oldValue === current) {
                    return; // Successfully decremented
                }
                // Value changed, retry
                continue;
            }
            // Wait for signal
            Atomics.wait(this.view, this.index, 0);
        }
    }

    tempta(): boolean {
        while (true) {
            const current = Atomics.load(this.view, this.index);
            if (current <= 0) {
                return false;
            }
            const oldValue = Atomics.compareExchange(
                this.view,
                this.index,
                current,
                current - 1
            );
            if (oldValue === current) {
                return true;
            }
            // Value changed between load and CAS, retry
        }
    }

    signa(): void {
        Atomics.add(this.view, this.index, 1);
        Atomics.notify(this.view, this.index, 1);
    }

    valor(): number {
        return Atomics.load(this.view, this.index);
    }
}

// =============================================================================
// CONDITION VARIABLE
// =============================================================================

export class Conditio {
    private view: Int32Array;
    private index: number;

    constructor(memoria: Uint8Array, offset: number) {
        const buffer = memoria.buffer;
        const byteOffset = memoria.byteOffset + offset;
        this.view = new Int32Array(buffer, byteOffset, 1);
        this.index = 0;
        Atomics.store(this.view, this.index, 0);
    }

    exspecta(mutex: Mutex): void {
        const waitValue = Atomics.load(this.view, this.index);
        mutex.solve();
        Atomics.wait(this.view, this.index, waitValue);
        mutex.obstringe();
    }

    exspectaUsque(mutex: Mutex, ms: number): boolean {
        const waitValue = Atomics.load(this.view, this.index);
        mutex.solve();
        const result = Atomics.wait(this.view, this.index, waitValue, ms);
        mutex.obstringe();
        return result !== 'timed-out';
    }

    signa(): void {
        Atomics.add(this.view, this.index, 1);
        Atomics.notify(this.view, this.index, 1);
    }

    diffunde(): void {
        Atomics.add(this.view, this.index, 1);
        Atomics.notify(this.view, this.index, Infinity);
    }
}

// =============================================================================
// NUNCIUS MODULE
// =============================================================================

export const nuncius = {
    // =========================================================================
    // SHARED MEMORY
    // =========================================================================

    alloca(magnitudo: number): Uint8Array {
        const buffer = new SharedArrayBuffer(magnitudo);
        return new Uint8Array(buffer);
    },

    // =========================================================================
    // MESSAGE PORTS
    // =========================================================================

    portae(): ParPortarum {
        return new ParPortarum();
    },

    // =========================================================================
    // SYNCHRONIZATION PRIMITIVES
    // =========================================================================

    mutex(memoria: Uint8Array, offset: number): Mutex {
        return new Mutex(memoria, offset);
    },

    semaphorum(memoria: Uint8Array, offset: number, valor: number): Semaphorum {
        return new Semaphorum(memoria, offset, valor);
    },

    conditio(memoria: Uint8Array, offset: number): Conditio {
        return new Conditio(memoria, offset);
    },
};
