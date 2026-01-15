import { test, expect, describe, beforeEach, afterEach } from 'bun:test';
import { thesaurus, Subscriptio, Nuntius } from './thesaurus';

describe('thesaurus HAL', () => {
    beforeEach(() => {
        thesaurus._reset();
    });

    afterEach(() => {
        thesaurus._reset();
    });

    describe('key-value cache', () => {
        test('cape returns null for missing key', async () => {
            const result = await thesaurus.cape('missing');
            expect(result).toBe(null);
        });

        test('pone and cape store and retrieve values', async () => {
            await thesaurus.pone('name', 'Alice');
            expect(await thesaurus.cape('name')).toBe('Alice');

            await thesaurus.pone('count', '42');
            expect(await thesaurus.cape('count')).toBe('42');
        });

        test('pone overwrites existing values', async () => {
            await thesaurus.pone('key', 'first');
            await thesaurus.pone('key', 'second');
            expect(await thesaurus.cape('key')).toBe('second');
        });

        test('poneNovum only sets if key does not exist', async () => {
            const first = await thesaurus.poneNovum('unique', 'value1');
            expect(first).toBe(true);
            expect(await thesaurus.cape('unique')).toBe('value1');

            const second = await thesaurus.poneNovum('unique', 'value2');
            expect(second).toBe(false);
            expect(await thesaurus.cape('unique')).toBe('value1');
        });

        test('dele removes keys and returns count', async () => {
            await thesaurus.pone('a', '1');
            await thesaurus.pone('b', '2');
            await thesaurus.pone('c', '3');

            const count = await thesaurus.dele(['a', 'c', 'nonexistent']);
            expect(count).toBe(2);
            expect(await thesaurus.cape('a')).toBe(null);
            expect(await thesaurus.cape('b')).toBe('2');
            expect(await thesaurus.cape('c')).toBe(null);
        });

        test('exstat checks key existence', async () => {
            expect(await thesaurus.exstat('key')).toBe(false);
            await thesaurus.pone('key', 'value');
            expect(await thesaurus.exstat('key')).toBe(true);
        });
    });

    describe('TTL', () => {
        test('ttl returns -2 for missing key', async () => {
            expect(await thesaurus.ttl('missing')).toBe(-2);
        });

        test('ttl returns -1 for key without expiry', async () => {
            await thesaurus.pone('permanent', 'value');
            expect(await thesaurus.ttl('permanent')).toBe(-1);
        });

        test('ttl returns remaining seconds', async () => {
            await thesaurus.pone('temp', 'value', 10);
            const remaining = await thesaurus.ttl('temp');
            expect(remaining).toBeGreaterThan(8);
            expect(remaining).toBeLessThanOrEqual(10);
        });

        test('TTL expires entries', async () => {
            await thesaurus.pone('expiring', 'value', 1);
            expect(await thesaurus.cape('expiring')).toBe('value');

            // Wait for expiry
            await new Promise(resolve => setTimeout(resolve, 1100));

            expect(await thesaurus.cape('expiring')).toBe(null);
            expect(await thesaurus.exstat('expiring')).toBe(false);
            expect(await thesaurus.ttl('expiring')).toBe(-2);
        });

        test('expira sets expiry on existing key', async () => {
            await thesaurus.pone('key', 'value');
            expect(await thesaurus.ttl('key')).toBe(-1);

            const result = await thesaurus.expira('key', 5);
            expect(result).toBe(true);

            const ttl = await thesaurus.ttl('key');
            expect(ttl).toBeGreaterThan(3);
            expect(ttl).toBeLessThanOrEqual(5);
        });

        test('expira returns false for missing key', async () => {
            const result = await thesaurus.expira('missing', 5);
            expect(result).toBe(false);
        });

        test('poneNovum succeeds after TTL expiry', async () => {
            await thesaurus.pone('key', 'old', 1);
            await new Promise(resolve => setTimeout(resolve, 1100));

            const result = await thesaurus.poneNovum('key', 'new');
            expect(result).toBe(true);
            expect(await thesaurus.cape('key')).toBe('new');
        });
    });

    describe('numeric operations', () => {
        test('incr increments by 1', async () => {
            expect(await thesaurus.incr('counter')).toBe(1);
            expect(await thesaurus.incr('counter')).toBe(2);
            expect(await thesaurus.incr('counter')).toBe(3);
        });

        test('incrPer increments by amount', async () => {
            expect(await thesaurus.incrPer('score', 10)).toBe(10);
            expect(await thesaurus.incrPer('score', 5)).toBe(15);
            expect(await thesaurus.incrPer('score', -3)).toBe(12);
        });

        test('decr decrements by 1', async () => {
            await thesaurus.pone('count', '10');
            expect(await thesaurus.decr('count')).toBe(9);
            expect(await thesaurus.decr('count')).toBe(8);
        });

        test('incr/decr work on string numeric values', async () => {
            await thesaurus.pone('num', '100');
            expect(await thesaurus.incr('num')).toBe(101);
            expect(await thesaurus.decr('num')).toBe(100);
        });

        test('incr treats non-numeric as 0', async () => {
            await thesaurus.pone('text', 'hello');
            expect(await thesaurus.incr('text')).toBe(1);
        });

        test('incr preserves TTL', async () => {
            await thesaurus.pone('counter', '0', 60);
            await thesaurus.incr('counter');

            const ttl = await thesaurus.ttl('counter');
            expect(ttl).toBeGreaterThan(55);
        });
    });

    describe('key queries', () => {
        beforeEach(async () => {
            await thesaurus.pone('user:1', 'alice');
            await thesaurus.pone('user:2', 'bob');
            await thesaurus.pone('user:10', 'charlie');
            await thesaurus.pone('session:abc', 'data1');
            await thesaurus.pone('config', 'settings');
        });

        test('claves matches exact key', async () => {
            const keys = await thesaurus.claves('config');
            expect(keys).toEqual(['config']);
        });

        test('claves matches * wildcard', async () => {
            const keys = await thesaurus.claves('user:*');
            expect(keys.sort()).toEqual(['user:1', 'user:10', 'user:2']);
        });

        test('claves matches ? single char', async () => {
            const keys = await thesaurus.claves('user:?');
            expect(keys.sort()).toEqual(['user:1', 'user:2']);
        });

        test('claves matches all with *', async () => {
            const keys = await thesaurus.claves('*');
            expect(keys.length).toBe(5);
        });

        test('claves returns empty for no match', async () => {
            const keys = await thesaurus.claves('nomatch:*');
            expect(keys).toEqual([]);
        });

        test('claves excludes expired keys', async () => {
            await thesaurus.pone('temp:1', 'value', 1);
            await new Promise(resolve => setTimeout(resolve, 1100));

            const keys = await thesaurus.claves('temp:*');
            expect(keys).toEqual([]);
        });
    });

    describe('pub/sub', () => {
        test('subscribe returns Subscriptio', async () => {
            const sub = await thesaurus.subscribe(['topic']);
            expect(sub).toBeInstanceOf(Subscriptio);
            sub.claude();
        });

        test('publica returns subscriber count', async () => {
            const sub1 = await thesaurus.subscribe(['news']);
            const sub2 = await thesaurus.subscribe(['news']);
            const sub3 = await thesaurus.subscribe(['other']);

            const count = await thesaurus.publica('news', 'hello');
            expect(count).toBe(2);

            sub1.claude();
            sub2.claude();
            sub3.claude();
        });

        test('subscription receives published messages', async () => {
            const sub = await thesaurus.subscribe(['events']);
            const messages: Nuntius[] = [];

            // Start consuming in background
            const consumer = (async () => {
                for await (const msg of sub.nuntii()) {
                    messages.push(msg);
                    if (messages.length >= 2) break;
                }
            })();

            // Publish messages
            await thesaurus.publica('events', 'first');
            await thesaurus.publica('events', 'second');

            await consumer;

            expect(messages.length).toBe(2);
            expect(messages[0].corpus()).toBe('first');
            expect(messages[1].corpus()).toBe('second');
            expect(messages[0].thema()).toBe('events');

            sub.claude();
        });

        test('Nuntius has correct properties', async () => {
            const sub = await thesaurus.subscribe(['test']);
            const before = Date.now();

            const consumer = (async () => {
                for await (const msg of sub.nuntii()) {
                    return msg;
                }
            })();

            await thesaurus.publica('test', 'payload');
            const msg = await Promise.race([
                consumer,
                new Promise<Nuntius>((_, reject) => setTimeout(() => reject(new Error('timeout')), 1000)),
            ]);

            expect(msg!.thema()).toBe('test');
            expect(msg!.corpus()).toBe('payload');
            expect(msg!.tempus()).toBeGreaterThanOrEqual(before);
            expect(msg!.tempus()).toBeLessThanOrEqual(Date.now());

            sub.claude();
        });

        test('claude() stops subscription', async () => {
            const sub = await thesaurus.subscribe(['channel']);
            let iterations = 0;

            const consumer = (async () => {
                for await (const _msg of sub.nuntii()) {
                    iterations++;
                }
            })();

            await thesaurus.publica('channel', 'msg1');
            await new Promise(resolve => setTimeout(resolve, 50));

            sub.claude();
            await consumer;

            // Should have received one message then stopped
            expect(iterations).toBe(1);

            // Further publishes should not be received
            const count = await thesaurus.publica('channel', 'msg2');
            expect(count).toBe(0);
        });

        test('pattern * matches single segment', async () => {
            const sub = await thesaurus.subscribe(['events/*']);
            const messages: string[] = [];

            const consumer = (async () => {
                for await (const msg of sub.nuntii()) {
                    messages.push(msg.thema());
                    if (messages.length >= 2) break;
                }
            })();

            await thesaurus.publica('events/click', 'data');
            await thesaurus.publica('events/scroll', 'data');
            await thesaurus.publica('events/mouse/move', 'data'); // Should not match

            await Promise.race([
                consumer,
                new Promise(resolve => setTimeout(resolve, 100)),
            ]);

            expect(messages).toEqual(['events/click', 'events/scroll']);
            sub.claude();
        });

        test('pattern ** matches multiple segments', async () => {
            const sub = await thesaurus.subscribe(['logs/**']);
            const messages: string[] = [];

            const consumer = (async () => {
                for await (const msg of sub.nuntii()) {
                    messages.push(msg.thema());
                    if (messages.length >= 3) break;
                }
            })();

            await thesaurus.publica('logs/error', 'data');
            await thesaurus.publica('logs/app/debug', 'data');
            await thesaurus.publica('logs/app/module/trace', 'data');
            await thesaurus.publica('other/log', 'data'); // Should not match

            await Promise.race([
                consumer,
                new Promise(resolve => setTimeout(resolve, 100)),
            ]);

            expect(messages.sort()).toEqual([
                'logs/app/debug',
                'logs/app/module/trace',
                'logs/error',
            ]);
            sub.claude();
        });

        test('multiple patterns in single subscription', async () => {
            const sub = await thesaurus.subscribe(['news', 'alerts/*']);
            const messages: string[] = [];

            const consumer = (async () => {
                for await (const msg of sub.nuntii()) {
                    messages.push(msg.thema());
                    if (messages.length >= 2) break;
                }
            })();

            await thesaurus.publica('news', 'headline');
            await thesaurus.publica('alerts/critical', 'warning');

            await Promise.race([
                consumer,
                new Promise(resolve => setTimeout(resolve, 100)),
            ]);

            expect(messages.sort()).toEqual(['alerts/critical', 'news']);
            sub.claude();
        });
    });
});
