import { test, expect, describe } from 'bun:test';
import { toml } from './toml';

describe('toml HAL', () => {
    // =========================================================================
    // SERIALIZATION (pange)
    // =========================================================================

    describe('pange (serialize)', () => {
        test('serializes to TOML', () => {
            const result = toml.pange({ name: 'Alice', age: 30 });
            expect(result).toContain('name');
            expect(result).toContain('Alice');
            expect(result).toContain('age');
            expect(result).toContain('30');
        });

        test('handles nested tables', () => {
            const result = toml.pange({
                database: {
                    host: 'localhost',
                    port: 5432,
                },
            });
            expect(result).toContain('[database]');
            expect(result).toContain('host');
            expect(result).toContain('localhost');
        });

        test('throws on non-object root', () => {
            expect(() => toml.pange('string')).toThrow('TOML root must be a table');
            expect(() => toml.pange(['array'])).toThrow('TOML root must be a table');
            expect(() => toml.pange(42)).toThrow('TOML root must be a table');
            expect(() => toml.pange(null)).toThrow('TOML root must be a table');
        });
    });

    // =========================================================================
    // PARSING (solve, tempta)
    // =========================================================================

    describe('solve (parse)', () => {
        test('parses TOML tables', () => {
            const result = toml.solve('name = "Alice"\nage = 30');
            expect(result).toEqual({ name: 'Alice', age: 30 });
        });

        test('parses nested tables', () => {
            const input = `
[database]
host = "localhost"
port = 5432
`;
            const result = toml.solve(input);
            expect(result).toEqual({
                database: {
                    host: 'localhost',
                    port: 5432,
                },
            });
        });

        test('parses arrays', () => {
            const result = toml.solve('items = [1, 2, 3]');
            expect(result).toEqual({ items: [1, 2, 3] });
        });

        test('parses array of tables', () => {
            const input = `
[[products]]
name = "Hammer"
price = 9.99

[[products]]
name = "Nail"
price = 0.05
`;
            const result = toml.solve(input) as { products: Array<{ name: string; price: number }> };
            expect(result.products).toHaveLength(2);
            expect(result.products[0]!.name).toBe('Hammer');
            expect(result.products[1]!.name).toBe('Nail');
        });

        test('throws on invalid TOML', () => {
            expect(() => toml.solve('invalid = = value')).toThrow();
            expect(() => toml.solve('key = "unclosed string')).toThrow();
        });
    });

    describe('tempta (try parse)', () => {
        test('returns parsed value on valid TOML', () => {
            expect(toml.tempta('valid = "toml"')).toEqual({ valid: 'toml' });
        });

        test('returns null on invalid TOML', () => {
            expect(toml.tempta('invalid = = value')).toBe(null);
            expect(toml.tempta('key = "unclosed string')).toBe(null);
        });
    });

    // =========================================================================
    // TYPE CHECKING
    // =========================================================================

    describe('type checking', () => {
        test('estNihil', () => {
            expect(toml.estNihil(null)).toBe(true);
            expect(toml.estNihil(undefined)).toBe(true);
            expect(toml.estNihil(0)).toBe(false);
        });

        test('estBivalens', () => {
            expect(toml.estBivalens(true)).toBe(true);
            expect(toml.estBivalens(false)).toBe(true);
            expect(toml.estBivalens(1)).toBe(false);
            expect(toml.estBivalens('true')).toBe(false);
        });

        test('estTextus', () => {
            expect(toml.estTextus('hello')).toBe(true);
            expect(toml.estTextus('')).toBe(true);
            expect(toml.estTextus(42)).toBe(false);
            expect(toml.estTextus(null)).toBe(false);
        });

        test('estInteger', () => {
            expect(toml.estInteger(42)).toBe(true);
            expect(toml.estInteger(0)).toBe(true);
            expect(toml.estInteger(-10)).toBe(true);
            expect(toml.estInteger(3.14)).toBe(false);
            expect(toml.estInteger('42')).toBe(false);
        });

        test('estFractus', () => {
            expect(toml.estFractus(3.14)).toBe(true);
            expect(toml.estFractus(0.5)).toBe(true);
            expect(toml.estFractus(42)).toBe(false);
            expect(toml.estFractus('3.14')).toBe(false);
        });

        test('estTempus', () => {
            expect(toml.estTempus(new Date())).toBe(true);
            expect(toml.estTempus(new Date('2024-01-01'))).toBe(true);
            expect(toml.estTempus('2024-01-01')).toBe(false);
            expect(toml.estTempus(1704067200000)).toBe(false);
        });

        test('estLista', () => {
            expect(toml.estLista([])).toBe(true);
            expect(toml.estLista([1, 2, 3])).toBe(true);
            expect(toml.estLista({})).toBe(false);
            expect(toml.estLista('array')).toBe(false);
        });

        test('estTabula', () => {
            expect(toml.estTabula({})).toBe(true);
            expect(toml.estTabula({ a: 1 })).toBe(true);
            expect(toml.estTabula([])).toBe(false);
            expect(toml.estTabula(null)).toBe(false);
            expect(toml.estTabula(new Date())).toBe(false);
        });
    });

    // =========================================================================
    // VALUE EXTRACTION
    // =========================================================================

    describe('value extraction', () => {
        test('utTextus', () => {
            expect(toml.utTextus('hello', 'default')).toBe('hello');
            expect(toml.utTextus(42, 'default')).toBe('default');
        });

        test('utNumerus', () => {
            expect(toml.utNumerus(42, 0)).toBe(42);
            expect(toml.utNumerus('42', 0)).toBe(0);
        });

        test('utBivalens', () => {
            expect(toml.utBivalens(true, false)).toBe(true);
            expect(toml.utBivalens('true', false)).toBe(false);
        });
    });

    // =========================================================================
    // VALUE ACCESS
    // =========================================================================

    describe('cape (get by key)', () => {
        test('retrieves top-level values', () => {
            const data = { name: 'Alice', age: 30 };
            expect(toml.cape(data, 'name')).toBe('Alice');
            expect(toml.cape(data, 'age')).toBe(30);
            expect(toml.cape(data, 'missing')).toBe(null);
        });

        test('returns null for non-objects', () => {
            expect(toml.cape(null, 'key')).toBe(null);
            expect(toml.cape(undefined, 'key')).toBe(null);
            expect(toml.cape([1, 2], 'key')).toBe(null);
        });
    });

    describe('carpe (pluck by index)', () => {
        test('gets array element', () => {
            const arr = ['a', 'b', 'c'];
            expect(toml.carpe(arr, 0)).toBe('a');
            expect(toml.carpe(arr, 2)).toBe('c');
            expect(toml.carpe(arr, 5)).toBe(null);
            expect(toml.carpe(arr, -1)).toBe(null);
        });

        test('returns null for non-arrays', () => {
            expect(toml.carpe({ 0: 'a' }, 0)).toBe(null);
            expect(toml.carpe(null, 0)).toBe(null);
        });
    });

    describe('inveni (find by path)', () => {
        test('retrieves nested values with dot notation', () => {
            const data = {
                database: {
                    connection: {
                        host: 'localhost',
                        port: 5432,
                    },
                },
            };
            expect(toml.inveni(data, 'database.connection.host')).toBe('localhost');
            expect(toml.inveni(data, 'database.connection.port')).toBe(5432);
            expect(toml.inveni(data, 'database.connection')).toEqual({ host: 'localhost', port: 5432 });
        });

        test('returns null for invalid paths', () => {
            const data = { a: { b: 1 } };
            expect(toml.inveni(data, 'a.b.c')).toBe(null);
            expect(toml.inveni(data, 'x.y.z')).toBe(null);
        });

        test('handles null/undefined input', () => {
            expect(toml.inveni(null, 'key')).toBe(null);
            expect(toml.inveni(undefined, 'key')).toBe(null);
        });

        test('handles array indices in path', () => {
            const data = { items: ['a', 'b', 'c'] };
            expect(toml.inveni(data, 'items.0')).toBe('a');
            expect(toml.inveni(data, 'items.2')).toBe('c');
        });
    });

    // =========================================================================
    // ROUNDTRIP
    // =========================================================================

    describe('roundtrip', () => {
        test('serialize then parse preserves data', () => {
            const original = {
                string: 'hello',
                integer: 42,
                float: 3.14,
                bool: true,
                array: [1, 2, 3],
                nested: { a: { b: { c: 'deep' } } },
            };

            const serialized = toml.pange(original);
            const parsed = toml.solve(serialized);

            expect(parsed).toEqual(original);
        });

        test('roundtrip with nested tables', () => {
            const original = {
                server: {
                    host: 'localhost',
                    port: 8080,
                },
                database: {
                    name: 'mydb',
                    credentials: {
                        user: 'admin',
                        password: 'secret',
                    },
                },
            };

            const serialized = toml.pange(original);
            const parsed = toml.solve(serialized);

            expect(parsed).toEqual(original);
        });
    });

    // =========================================================================
    // TOML-SPECIFIC FEATURES
    // =========================================================================

    describe('TOML-specific features', () => {
        test('parses TOML comments (ignored)', () => {
            const input = `
# This is a comment
name = "Alice"  # inline comment
age = 30
`;
            const result = toml.solve(input);
            expect(result).toEqual({ name: 'Alice', age: 30 });
        });

        test('parses inline tables', () => {
            const input = 'point = { x = 1, y = 2 }';
            const result = toml.solve(input);
            expect(result).toEqual({ point: { x: 1, y: 2 } });
        });

        test('parses multiline strings', () => {
            const input = `
description = """
This is a
multiline string"""
`;
            const result = toml.solve(input) as { description: string };
            expect(result.description).toContain('This is a');
            expect(result.description).toContain('multiline string');
        });

        test('parses literal strings', () => {
            const input = "path = 'C:\\Windows\\System32'";
            const result = toml.solve(input) as { path: string };
            expect(result.path).toBe('C:\\Windows\\System32');
        });
    });
});
