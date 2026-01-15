import { test, expect, describe } from 'bun:test';
import { json } from './json';

describe('json HAL', () => {
    describe('serialization', () => {
        test('solve serializes to compact JSON', () => {
            expect(json.solve({ a: 1, b: 2 })).toBe('{"a":1,"b":2}');
            expect(json.solve([1, 2, 3])).toBe('[1,2,3]');
            expect(json.solve('hello')).toBe('"hello"');
            expect(json.solve(42)).toBe('42');
            expect(json.solve(true)).toBe('true');
            expect(json.solve(null)).toBe('null');
        });

        test('solvePulchre serializes with indentation', () => {
            const result = json.solvePulchre({ a: 1 }, 2);
            expect(result).toBe('{\n  "a": 1\n}');
        });
    });

    describe('deserialization', () => {
        test('pange parses valid JSON', () => {
            expect(json.pange('{"a":1}')).toEqual({ a: 1 });
            expect(json.pange('[1,2,3]')).toEqual([1, 2, 3]);
            expect(json.pange('"hello"')).toBe('hello');
            expect(json.pange('42')).toBe(42);
            expect(json.pange('true')).toBe(true);
            expect(json.pange('null')).toBe(null);
        });

        test('pange throws on invalid JSON', () => {
            expect(() => json.pange('invalid')).toThrow();
            expect(() => json.pange('{')).toThrow();
        });

        test('pangeTuto returns null on invalid JSON', () => {
            expect(json.pangeTuto('invalid')).toBe(null);
            expect(json.pangeTuto('{')).toBe(null);
            expect(json.pangeTuto('{"a":1}')).toEqual({ a: 1 });
        });
    });

    describe('type checking', () => {
        test('estNihil', () => {
            expect(json.estNihil(null)).toBe(true);
            expect(json.estNihil(undefined)).toBe(true);
            expect(json.estNihil(0)).toBe(false);
            expect(json.estNihil('')).toBe(false);
            expect(json.estNihil(false)).toBe(false);
        });

        test('estBivalens', () => {
            expect(json.estBivalens(true)).toBe(true);
            expect(json.estBivalens(false)).toBe(true);
            expect(json.estBivalens(0)).toBe(false);
            expect(json.estBivalens('true')).toBe(false);
        });

        test('estNumerus', () => {
            expect(json.estNumerus(42)).toBe(true);
            expect(json.estNumerus(3.14)).toBe(true);
            expect(json.estNumerus('42')).toBe(false);
            expect(json.estNumerus(NaN)).toBe(true); // NaN is typeof number
        });

        test('estTextus', () => {
            expect(json.estTextus('hello')).toBe(true);
            expect(json.estTextus('')).toBe(true);
            expect(json.estTextus(42)).toBe(false);
        });

        test('estLista', () => {
            expect(json.estLista([])).toBe(true);
            expect(json.estLista([1, 2, 3])).toBe(true);
            expect(json.estLista({})).toBe(false);
            expect(json.estLista('array')).toBe(false);
        });

        test('estTabula', () => {
            expect(json.estTabula({})).toBe(true);
            expect(json.estTabula({ a: 1 })).toBe(true);
            expect(json.estTabula([])).toBe(false);
            expect(json.estTabula(null)).toBe(false);
        });
    });

    describe('value extraction', () => {
        test('utTextus', () => {
            expect(json.utTextus('hello', 'default')).toBe('hello');
            expect(json.utTextus(42, 'default')).toBe('default');
            expect(json.utTextus(null, 'default')).toBe('default');
        });

        test('utNumerus', () => {
            expect(json.utNumerus(42, 0)).toBe(42);
            expect(json.utNumerus('42', 0)).toBe(0);
            expect(json.utNumerus(null, -1)).toBe(-1);
        });

        test('utBivalens', () => {
            expect(json.utBivalens(true, false)).toBe(true);
            expect(json.utBivalens(false, true)).toBe(false);
            expect(json.utBivalens('true', false)).toBe(false);
            expect(json.utBivalens(null, true)).toBe(true);
        });
    });

    describe('object/array access', () => {
        test('cape gets object property', () => {
            const obj = { name: 'Alice', age: 30 };
            expect(json.cape(obj, 'name')).toBe('Alice');
            expect(json.cape(obj, 'age')).toBe(30);
            expect(json.cape(obj, 'missing')).toBe(null);
        });

        test('cape returns null for non-objects', () => {
            expect(json.cape([1, 2, 3], '0')).toBe(null);
            expect(json.cape('string', 'length')).toBe(null);
            expect(json.cape(null, 'x')).toBe(null);
        });

        test('capeIndice gets array element', () => {
            const arr = ['a', 'b', 'c'];
            expect(json.capeIndice(arr, 0)).toBe('a');
            expect(json.capeIndice(arr, 2)).toBe('c');
            expect(json.capeIndice(arr, 3)).toBe(null);
            expect(json.capeIndice(arr, -1)).toBe(null);
        });

        test('capeIndice returns null for non-arrays', () => {
            expect(json.capeIndice({ 0: 'a' }, 0)).toBe(null);
            expect(json.capeIndice('abc', 0)).toBe(null);
        });

        test('capeVia gets nested value', () => {
            const obj = {
                user: {
                    name: 'Alice',
                    address: {
                        city: 'Paris',
                    },
                },
                tags: ['admin', 'user'],
            };

            expect(json.capeVia(obj, 'user.name')).toBe('Alice');
            expect(json.capeVia(obj, 'user.address.city')).toBe('Paris');
            expect(json.capeVia(obj, 'user.missing')).toBe(null);
            expect(json.capeVia(obj, 'tags.0')).toBe('admin');
            expect(json.capeVia(obj, 'tags.1')).toBe('user');
        });

        test('capeVia handles missing paths', () => {
            expect(json.capeVia(null, 'x')).toBe(null);
            expect(json.capeVia({}, 'a.b.c')).toBe(null);
            expect(json.capeVia({ a: null }, 'a.b')).toBe(null);
        });
    });

    describe('roundtrip', () => {
        test('serialize then parse preserves data', () => {
            const original = {
                string: 'hello',
                number: 42,
                float: 3.14,
                bool: true,
                nil: null,
                array: [1, 2, 3],
                nested: { a: { b: { c: 'deep' } } },
            };

            const serialized = json.solve(original);
            const parsed = json.pange(serialized);

            expect(parsed).toEqual(original);
        });
    });
});
