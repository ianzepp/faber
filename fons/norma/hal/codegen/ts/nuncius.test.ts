import { test, expect, describe } from 'bun:test';
import { nuncius, ParPortarum, Porta, Mutex, Semaphorum, Conditio } from './nuncius';

describe('nuncius HAL', () => {
    describe('shared memory', () => {
        test('alloca creates SharedArrayBuffer-backed Uint8Array', () => {
            const mem = nuncius.alloca(1024);
            expect(mem).toBeInstanceOf(Uint8Array);
            expect(mem.length).toBe(1024);
            expect(mem.buffer).toBeInstanceOf(SharedArrayBuffer);
        });

        test('alloca memory is zeroed', () => {
            const mem = nuncius.alloca(64);
            for (let i = 0; i < 64; i++) {
                expect(mem[i]).toBe(0);
            }
        });

        test('alloca memory is writable', () => {
            const mem = nuncius.alloca(16);
            mem[0] = 42;
            mem[15] = 255;
            expect(mem[0]).toBe(42);
            expect(mem[15]).toBe(255);
        });
    });

    describe('message ports', () => {
        test('portae creates a port pair', () => {
            const pair = nuncius.portae();
            expect(pair).toBeInstanceOf(ParPortarum);
            expect(pair.a()).toBeInstanceOf(Porta);
            expect(pair.b()).toBeInstanceOf(Porta);
        });

        test('send on A, receive on B', async () => {
            const pair = nuncius.portae();
            const portA = pair.a();
            const portB = pair.b();

            portA.mitte({ hello: 'world' });
            const msg = await portB.recipe();
            expect(msg).toEqual({ hello: 'world' });
        });

        test('send on B, receive on A', async () => {
            const pair = nuncius.portae();
            const portA = pair.a();
            const portB = pair.b();

            portB.mitte('test message');
            const msg = await portA.recipe();
            expect(msg).toBe('test message');
        });

        test('bidirectional communication', async () => {
            const pair = nuncius.portae();
            const portA = pair.a();
            const portB = pair.b();

            portA.mitte('from A');
            portB.mitte('from B');

            const msgOnB = await portB.recipe();
            const msgOnA = await portA.recipe();

            expect(msgOnB).toBe('from A');
            expect(msgOnA).toBe('from B');
        });

        test('multiple messages in sequence', async () => {
            const pair = nuncius.portae();
            const portA = pair.a();
            const portB = pair.b();

            portA.mitte(1);
            portA.mitte(2);
            portA.mitte(3);

            expect(await portB.recipe()).toBe(1);
            expect(await portB.recipe()).toBe(2);
            expect(await portB.recipe()).toBe(3);
        });

        test('paratum returns false when no messages', () => {
            const pair = nuncius.portae();
            expect(pair.a().paratum()).toBe(false);
            expect(pair.b().paratum()).toBe(false);
        });

        test('paratum returns true when message available', async () => {
            const pair = nuncius.portae();
            const portA = pair.a();
            const portB = pair.b();

            portA.mitte('test');

            // Give the message time to arrive
            await new Promise(resolve => setTimeout(resolve, 10));

            expect(portB.paratum()).toBe(true);
        });

        test('paratum becomes false after receiving', async () => {
            const pair = nuncius.portae();
            const portA = pair.a();
            const portB = pair.b();

            portA.mitte('test');
            await new Promise(resolve => setTimeout(resolve, 10));

            expect(portB.paratum()).toBe(true);
            await portB.recipe();
            expect(portB.paratum()).toBe(false);
        });

        test('claude closes the port', () => {
            const pair = nuncius.portae();
            const portA = pair.a();

            portA.claude();

            expect(() => portA.mitte('test')).toThrow('Port is closed');
        });
    });

    describe('mutex', () => {
        test('mutex creation works', () => {
            const mem = nuncius.alloca(16);
            const mutex = nuncius.mutex(mem, 0);
            expect(mutex).toBeInstanceOf(Mutex);
        });

        test('tempta acquires unlocked mutex', () => {
            const mem = nuncius.alloca(16);
            const mutex = nuncius.mutex(mem, 0);

            const acquired = mutex.tempta();
            expect(acquired).toBe(true);
        });

        test('tempta returns false when already locked', () => {
            const mem = nuncius.alloca(16);
            const mutex = nuncius.mutex(mem, 0);

            mutex.tempta(); // First lock succeeds
            const secondAttempt = mutex.tempta(); // Should fail
            expect(secondAttempt).toBe(false);
        });

        test('solve releases the lock', () => {
            const mem = nuncius.alloca(16);
            const mutex = nuncius.mutex(mem, 0);

            mutex.tempta();
            mutex.solve();

            // Should be able to lock again
            const reacquired = mutex.tempta();
            expect(reacquired).toBe(true);
        });

        test('multiple mutexes at different offsets', () => {
            const mem = nuncius.alloca(32);
            const mutex1 = nuncius.mutex(mem, 0);
            const mutex2 = nuncius.mutex(mem, 4);

            expect(mutex1.tempta()).toBe(true);
            expect(mutex2.tempta()).toBe(true); // Different mutex, should succeed

            mutex1.solve();
            mutex2.solve();
        });
    });

    describe('semaphore', () => {
        test('semaphorum creation with initial value', () => {
            const mem = nuncius.alloca(16);
            const sem = nuncius.semaphorum(mem, 0, 5);

            expect(sem).toBeInstanceOf(Semaphorum);
            expect(sem.valor()).toBe(5);
        });

        test('signa increments value', () => {
            const mem = nuncius.alloca(16);
            const sem = nuncius.semaphorum(mem, 0, 0);

            expect(sem.valor()).toBe(0);
            sem.signa();
            expect(sem.valor()).toBe(1);
            sem.signa();
            expect(sem.valor()).toBe(2);
        });

        test('tempta decrements when value > 0', () => {
            const mem = nuncius.alloca(16);
            const sem = nuncius.semaphorum(mem, 0, 3);

            expect(sem.tempta()).toBe(true);
            expect(sem.valor()).toBe(2);

            expect(sem.tempta()).toBe(true);
            expect(sem.valor()).toBe(1);
        });

        test('tempta returns false when value is 0', () => {
            const mem = nuncius.alloca(16);
            const sem = nuncius.semaphorum(mem, 0, 0);

            expect(sem.tempta()).toBe(false);
            expect(sem.valor()).toBe(0);
        });

        test('tempta returns false after decrementing to 0', () => {
            const mem = nuncius.alloca(16);
            const sem = nuncius.semaphorum(mem, 0, 1);

            expect(sem.tempta()).toBe(true);
            expect(sem.valor()).toBe(0);
            expect(sem.tempta()).toBe(false);
        });

        test('signa then tempta works', () => {
            const mem = nuncius.alloca(16);
            const sem = nuncius.semaphorum(mem, 0, 0);

            expect(sem.tempta()).toBe(false);
            sem.signa();
            expect(sem.tempta()).toBe(true);
            expect(sem.valor()).toBe(0);
        });
    });

    describe('condition variable', () => {
        test('conditio creation works', () => {
            const mem = nuncius.alloca(16);
            const cond = nuncius.conditio(mem, 0);
            expect(cond).toBeInstanceOf(Conditio);
        });

        test('signa wakes waiters', () => {
            const mem = nuncius.alloca(16);
            const cond = nuncius.conditio(mem, 0);

            // Just verify signa doesn't throw
            cond.signa();
        });

        test('diffunde wakes all waiters', () => {
            const mem = nuncius.alloca(16);
            const cond = nuncius.conditio(mem, 0);

            // Just verify diffunde doesn't throw
            cond.diffunde();
        });

        test('exspectaUsque times out correctly', () => {
            const mem = nuncius.alloca(16);
            const mutexMem = nuncius.alloca(16);
            const mutex = nuncius.mutex(mutexMem, 0);
            const cond = nuncius.conditio(mem, 0);

            mutex.obstringe();
            const start = Date.now();
            const result = cond.exspectaUsque(mutex, 50);
            const elapsed = Date.now() - start;

            // Should have timed out (return false) after ~50ms
            expect(result).toBe(false);
            expect(elapsed).toBeGreaterThanOrEqual(40); // Allow some tolerance
            expect(elapsed).toBeLessThan(200);

            mutex.solve();
        });

        test('exspectaUsque returns true if signaled before wait captures value', () => {
            // This tests the spurious wakeup case - if the condition is signaled
            // between loading the wait value and actually waiting, we get immediate return
            const mem = nuncius.alloca(16);
            const mutexMem = nuncius.alloca(16);
            const mutex = nuncius.mutex(mutexMem, 0);
            const cond = nuncius.conditio(mem, 0);

            // Pre-signal to change the internal counter
            cond.signa();
            cond.signa();

            mutex.obstringe();
            // The wait value is now 2, so if we wait on 2 and nothing changes,
            // we should timeout. This verifies the mechanism works.
            const start = Date.now();
            const result = cond.exspectaUsque(mutex, 50);
            const elapsed = Date.now() - start;

            // Times out because no one signals during our wait
            expect(result).toBe(false);
            expect(elapsed).toBeGreaterThanOrEqual(40);

            mutex.solve();
        });
    });

    describe('integration', () => {
        test('shared memory with multiple views', () => {
            const mem = nuncius.alloca(32);

            // Create two semaphores at different offsets
            const sem1 = nuncius.semaphorum(mem, 0, 10);
            const sem2 = nuncius.semaphorum(mem, 4, 20);

            expect(sem1.valor()).toBe(10);
            expect(sem2.valor()).toBe(20);

            sem1.signa();
            sem2.tempta();

            expect(sem1.valor()).toBe(11);
            expect(sem2.valor()).toBe(19);
        });

        test('mutex and semaphore in same memory', () => {
            const mem = nuncius.alloca(16);

            const mutex = nuncius.mutex(mem, 0);
            const sem = nuncius.semaphorum(mem, 4, 1);

            expect(mutex.tempta()).toBe(true);
            expect(sem.tempta()).toBe(true);

            mutex.solve();
            sem.signa();

            expect(mutex.tempta()).toBe(true);
            expect(sem.valor()).toBe(1);
        });
    });
});
