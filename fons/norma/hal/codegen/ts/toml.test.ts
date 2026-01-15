import { test, expect, describe } from 'bun:test';
import { toml } from './toml';

describe('toml HAL', () => {
    describe('serialization', () => {
        test('solve serializes to TOML', () => {
            const result = toml.solve({ name: 'Alice', age: 30 });
            expect(result).toContain('name');
            expect(result).toContain('Alice');
            expect(result).toContain('age');
            expect(result).toContain('30');
        });

        test('solve handles nested tables', () => {
            const result = toml.solve({
                database: {
                    host: 'localhost',
                    port: 5432,
                },
            });
            expect(result).toContain('[database]');
            expect(result).toContain('host');
            expect(result).toContain('localhost');
        });

        test('solve throws on non-object root', () => {
            expect(() => toml.solve('string')).toThrow('TOML root must be a table');
            expect(() => toml.solve(['array'])).toThrow('TOML root must be a table');
            expect(() => toml.solve(42)).toThrow('TOML root must be a table');
            expect(() => toml.solve(null)).toThrow('TOML root must be a table');
        });

        test('solvePulchre produces formatted output', () => {
            const result = toml.solvePulchre({ name: 'Alice', age: 30 });
            expect(result).toContain('name');
            expect(result).toContain('Alice');
        });
    });

    describe('deserialization', () => {
        test('pange parses TOML tables', () => {
            const result = toml.pange('name = "Alice"\nage = 30');
            expect(result).toEqual({ name: 'Alice', age: 30 });
        });

        test('pange parses nested tables', () => {
            const input = `
[database]
host = "localhost"
port = 5432
`;
            const result = toml.pange(input);
            expect(result).toEqual({
                database: {
                    host: 'localhost',
                    port: 5432,
                },
            });
        });

        test('pange parses arrays', () => {
            const result = toml.pange('items = [1, 2, 3]');
            expect(result).toEqual({ items: [1, 2, 3] });
        });

        test('pange parses array of tables', () => {
            const input = `
[[products]]
name = "Hammer"
price = 9.99

[[products]]
name = "Nail"
price = 0.05
`;
            const result = toml.pange(input) as { products: Array<{ name: string; price: number }> };
            expect(result.products).toHaveLength(2);
            expect(result.products[0]!.name).toBe('Hammer');
            expect(result.products[1]!.name).toBe('Nail');
        });

        test('pange throws on invalid TOML', () => {
            expect(() => toml.pange('invalid = = value')).toThrow();
            expect(() => toml.pange('key = "unclosed string')).toThrow();
        });

        test('pangeTuto returns null on invalid TOML', () => {
            expect(toml.pangeTuto('valid = "toml"')).toEqual({ valid: 'toml' });
            expect(toml.pangeTuto('invalid = = value')).toBe(null);
            expect(toml.pangeTuto('key = "unclosed string')).toBe(null);
        });
    });

    describe('type checking', () => {
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

        test('estBivalens', () => {
            expect(toml.estBivalens(true)).toBe(true);
            expect(toml.estBivalens(false)).toBe(true);
            expect(toml.estBivalens(1)).toBe(false);
            expect(toml.estBivalens('true')).toBe(false);
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

    describe('table access', () => {
        test('cape retrieves top-level values', () => {
            const data = { name: 'Alice', age: 30 };
            expect(toml.cape(data, 'name')).toBe('Alice');
            expect(toml.cape(data, 'age')).toBe(30);
            expect(toml.cape(data, 'missing')).toBe(null);
        });

        test('cape retrieves nested values with dot notation', () => {
            const data = {
                database: {
                    connection: {
                        host: 'localhost',
                        port: 5432,
                    },
                },
            };
            expect(toml.cape(data, 'database.connection.host')).toBe('localhost');
            expect(toml.cape(data, 'database.connection.port')).toBe(5432);
            expect(toml.cape(data, 'database.connection')).toEqual({ host: 'localhost', port: 5432 });
        });

        test('cape returns null for invalid paths', () => {
            const data = { a: { b: 1 } };
            expect(toml.cape(data, 'a.b.c')).toBe(null);
            expect(toml.cape(data, 'x.y.z')).toBe(null);
        });

        test('cape handles null/undefined input', () => {
            expect(toml.cape(null, 'key')).toBe(null);
            expect(toml.cape(undefined, 'key')).toBe(null);
        });
    });

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

            const serialized = toml.solve(original);
            const parsed = toml.pange(serialized);

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

            const serialized = toml.solve(original);
            const parsed = toml.pange(serialized);

            expect(parsed).toEqual(original);
        });
    });

    describe('TOML-specific features', () => {
        test('parses TOML comments (ignored)', () => {
            const input = `
# This is a comment
name = "Alice"  # inline comment
age = 30
`;
            const result = toml.pange(input);
            expect(result).toEqual({ name: 'Alice', age: 30 });
        });

        test('parses inline tables', () => {
            const input = 'point = { x = 1, y = 2 }';
            const result = toml.pange(input);
            expect(result).toEqual({ point: { x: 1, y: 2 } });
        });

        test('parses multiline strings', () => {
            const input = `
description = """
This is a
multiline string"""
`;
            const result = toml.pange(input) as { description: string };
            expect(result.description).toContain('This is a');
            expect(result.description).toContain('multiline string');
        });

        test('parses literal strings', () => {
            const input = "path = 'C:\\Windows\\System32'";
            const result = toml.pange(input) as { path: string };
            expect(result.path).toBe('C:\\Windows\\System32');
        });
    });
});
