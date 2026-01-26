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
        test('capiet returns null for missing key', async () => {
            const result = await thesaurus.capiet('missing');
            expect(result).toBe(null);
        });

        test('ponet and capiet store and retrieve values', async () => {
            await thesaurus.ponet('name', 'Alice');
            expect(await thesaurus.capiet('name')).toBe('Alice');

            await thesaurus.ponet('count', '42');
            expect(await thesaurus.capiet('count')).toBe('42');
        });

        test('ponet overwrites existing values', async () => {
            await thesaurus.ponet('key', 'first');
            await thesaurus.ponet('key', 'second');
            expect(await thesaurus.capiet('key')).toBe('second');
        });

        test('ponetNovum only sets if key does not exist', async () => {
            const first = await thesaurus.ponetNovum('unique', 'value1');
            expect(first).toBe(true);
            expect(await thesaurus.capiet('unique')).toBe('value1');

            const second = await thesaurus.ponetNovum('unique', 'value2');
            expect(second).toBe(false);
            expect(await thesaurus.capiet('unique')).toBe('value1');
        });

        test('delet removes keys and returns count', async () => {
            await thesaurus.ponet('a', '1');
            await thesaurus.ponet('b', '2');
            await thesaurus.ponet('c', '3');

            const count = await thesaurus.delet(['a', 'c', 'nonexistent']);
            expect(count).toBe(2);
            expect(await thesaurus.capiet('a')).toBe(null);
            expect(await thesaurus.capiet('b')).toBe('2');
            expect(await thesaurus.capiet('c')).toBe(null);
        });

        test('exstabit checks key existence', async () => {
            expect(await thesaurus.exstabit('key')).toBe(false);
            await thesaurus.ponet('key', 'value');
            expect(await thesaurus.exstabit('key')).toBe(true);
        });
    });

    describe('TTL', () => {
        test('ttl returns -2 for missing key', async () => {
            expect(await thesaurus.ttl('missing')).toBe(-2);
        });

        test('ttl returns -1 for key without expiry', async () => {
            await thesaurus.ponet('permanent', 'value');
            expect(await thesaurus.ttl('permanent')).toBe(-1);
        });

        test('ttl returns remaining seconds', async () => {
            await thesaurus.ponet('temp', 'value', 10);
            const remaining = await thesaurus.ttl('temp');
            expect(remaining).toBeGreaterThan(8);
            expect(remaining).toBeLessThanOrEqual(10);
        });

        test('TTL expires entries', async () => {
            await thesaurus.ponet('expiring', 'value', 1);
            expect(await thesaurus.capiet('expiring')).toBe('value');

            // Wait for expiry
            await new Promise(resolve => setTimeout(resolve, 1100));

            expect(await thesaurus.capiet('expiring')).toBe(null);
            expect(await thesaurus.exstabit('expiring')).toBe(false);
            expect(await thesaurus.ttl('expiring')).toBe(-2);
        });

        test('expirabit sets expiry on existing key', async () => {
            await thesaurus.ponet('key', 'value');
            expect(await thesaurus.ttl('key')).toBe(-1);

            const result = await thesaurus.expirabit('key', 5);
            expect(result).toBe(true);

            const ttl = await thesaurus.ttl('key');
            expect(ttl).toBeGreaterThan(3);
            expect(ttl).toBeLessThanOrEqual(5);
        });

        test('expirabit returns false for missing key', async () => {
            const result = await thesaurus.expirabit('missing', 5);
            expect(result).toBe(false);
        });

        test('ponetNovum succeeds after TTL expiry', async () => {
            await thesaurus.ponet('key', 'old', 1);
            await new Promise(resolve => setTimeout(resolve, 1100));

            const result = await thesaurus.ponetNovum('key', 'new');
            expect(result).toBe(true);
            expect(await thesaurus.capiet('key')).toBe('new');
        });
    });

    describe('numeric operations', () => {
        test('augebit increments by 1 by default', async () => {
            expect(await thesaurus.augebit('counter')).toBe(1);
            expect(await thesaurus.augebit('counter')).toBe(2);
            expect(await thesaurus.augebit('counter')).toBe(3);
        });

        test('augebit increments by amount', async () => {
            expect(await thesaurus.augebit('score', 10)).toBe(10);
            expect(await thesaurus.augebit('score', 5)).toBe(15);
            expect(await thesaurus.augebit('score', -3)).toBe(12);
        });

        test('minuet decrements by 1', async () => {
            await thesaurus.ponet('count', '10');
            expect(await thesaurus.minuet('count')).toBe(9);
            expect(await thesaurus.minuet('count')).toBe(8);
        });

        test('augebit/minuet work on string numeric values', async () => {
            await thesaurus.ponet('num', '100');
            expect(await thesaurus.augebit('num')).toBe(101);
            expect(await thesaurus.minuet('num')).toBe(100);
        });

        test('augebit treats non-numeric as 0', async () => {
            await thesaurus.ponet('text', 'hello');
            expect(await thesaurus.augebit('text')).toBe(1);
        });

        test('augebit preserves TTL', async () => {
            await thesaurus.ponet('counter', '0', 60);
            await thesaurus.augebit('counter');

            const ttl = await thesaurus.ttl('counter');
            expect(ttl).toBeGreaterThan(55);
        });
    });

    describe('key queries', () => {
        beforeEach(async () => {
            await thesaurus.ponet('user:1', 'alice');
            await thesaurus.ponet('user:2', 'bob');
            await thesaurus.ponet('user:10', 'charlie');
            await thesaurus.ponet('session:abc', 'data1');
            await thesaurus.ponet('config', 'settings');
        });

        test('quaeret matches exact key', async () => {
            const keys = await thesaurus.quaeret('config');
            expect(keys).toEqual(['config']);
        });

        test('quaeret matches * wildcard', async () => {
            const keys = await thesaurus.quaeret('user:*');
            expect(keys.sort()).toEqual(['user:1', 'user:10', 'user:2']);
        });

        test('quaeret matches ? single char', async () => {
            const keys = await thesaurus.quaeret('user:?');
            expect(keys.sort()).toEqual(['user:1', 'user:2']);
        });

        test('quaeret matches all with *', async () => {
            const keys = await thesaurus.quaeret('*');
            expect(keys.length).toBe(5);
        });

        test('quaeret returns empty for no match', async () => {
            const keys = await thesaurus.quaeret('nomatch:*');
            expect(keys).toEqual([]);
        });

        test('quaeret excludes expired keys', async () => {
            await thesaurus.ponet('temp:1', 'value', 1);
            await new Promise(resolve => setTimeout(resolve, 1100));

            const keys = await thesaurus.quaeret('temp:*');
            expect(keys).toEqual([]);
        });
    });

    describe('pub/sub', () => {
        test('auscultabit returns Subscriptio', async () => {
            const sub = await thesaurus.auscultabit(['topic']);
            expect(sub).toBeInstanceOf(Subscriptio);
            sub.claude();
        });

        test('publicabit returns subscriber count', async () => {
            const sub1 = await thesaurus.auscultabit(['news']);
            const sub2 = await thesaurus.auscultabit(['news']);
            const sub3 = await thesaurus.auscultabit(['other']);

            const count = await thesaurus.publicabit('news', 'hello');
            expect(count).toBe(2);

            sub1.claude();
            sub2.claude();
            sub3.claude();
        });

        test('subscription receives published messages', async () => {
            const sub = await thesaurus.auscultabit(['events']);
            const messages: Nuntius[] = [];

            // Start consuming in background
            const consumer = (async () => {
                for await (const msg of sub.nuntient()) {
                    messages.push(msg);
                    if (messages.length >= 2) break;
                }
            })();

            // Publish messages
            await thesaurus.publicabit('events', 'first');
            await thesaurus.publicabit('events', 'second');

            await consumer;

            expect(messages.length).toBe(2);
            expect(messages[0].corpus()).toBe('first');
            expect(messages[1].corpus()).toBe('second');
            expect(messages[0].thema()).toBe('events');

            sub.claude();
        });

        test('Nuntius has correct properties', async () => {
            const sub = await thesaurus.auscultabit(['test']);
            const before = Date.now();

            const consumer = (async () => {
                for await (const msg of sub.nuntient()) {
                    return msg;
                }
            })();

            await thesaurus.publicabit('test', 'payload');
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
            const sub = await thesaurus.auscultabit(['channel']);
            let iterations = 0;

            const consumer = (async () => {
                for await (const _msg of sub.nuntient()) {
                    iterations++;
                }
            })();

            await thesaurus.publicabit('channel', 'msg1');
            await new Promise(resolve => setTimeout(resolve, 50));

            sub.claude();
            await consumer;

            // Should have received one message then stopped
            expect(iterations).toBe(1);

            // Further publishes should not be received
            const count = await thesaurus.publicabit('channel', 'msg2');
            expect(count).toBe(0);
        });

        test('pattern * matches single segment', async () => {
            const sub = await thesaurus.auscultabit(['events/*']);
            const messages: string[] = [];

            const consumer = (async () => {
                for await (const msg of sub.nuntient()) {
                    messages.push(msg.thema());
                    if (messages.length >= 2) break;
                }
            })();

            await thesaurus.publicabit('events/click', 'data');
            await thesaurus.publicabit('events/scroll', 'data');
            await thesaurus.publicabit('events/mouse/move', 'data'); // Should not match

            await Promise.race([
                consumer,
                new Promise(resolve => setTimeout(resolve, 100)),
            ]);

            expect(messages).toEqual(['events/click', 'events/scroll']);
            sub.claude();
        });

        test('pattern ** matches multiple segments', async () => {
            const sub = await thesaurus.auscultabit(['logs/**']);
            const messages: string[] = [];

            const consumer = (async () => {
                for await (const msg of sub.nuntient()) {
                    messages.push(msg.thema());
                    if (messages.length >= 3) break;
                }
            })();

            await thesaurus.publicabit('logs/error', 'data');
            await thesaurus.publicabit('logs/app/debug', 'data');
            await thesaurus.publicabit('logs/app/module/trace', 'data');
            await thesaurus.publicabit('other/log', 'data'); // Should not match

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
            const sub = await thesaurus.auscultabit(['news', 'alerts/*']);
            const messages: string[] = [];

            const consumer = (async () => {
                for await (const msg of sub.nuntient()) {
                    messages.push(msg.thema());
                    if (messages.length >= 2) break;
                }
            })();

            await thesaurus.publicabit('news', 'headline');
            await thesaurus.publicabit('alerts/critical', 'warning');

            await Promise.race([
                consumer,
                new Promise(resolve => setTimeout(resolve, 100)),
            ]);

            expect(messages.sort()).toEqual(['alerts/critical', 'news']);
            sub.claude();
        });
    });
});
