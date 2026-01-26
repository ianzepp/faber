import { test, expect, describe } from 'bun:test';
import { yaml } from './yaml';

describe('yaml HAL', () => {
    // =========================================================================
    // SERIALIZATION (pange, necto)
    // =========================================================================

    describe('pange (serialize)', () => {
        test('serializes to YAML', () => {
            const result = yaml.pange({ name: 'Alice', age: 30 });
            expect(result).toContain('name:');
            expect(result).toContain('Alice');
            expect(result).toContain('age:');
            expect(result).toContain('30');
        });

        test('handles arrays', () => {
            const result = yaml.pange(['a', 'b', 'c']);
            expect(result).toContain('a');
            expect(result).toContain('b');
            expect(result).toContain('c');
        });

        test('handles primitives', () => {
            expect(yaml.pange('hello').trim()).toBe('hello');
            expect(yaml.pange(42).trim()).toBe('42');
            expect(yaml.pange(true).trim()).toBe('true');
            expect(yaml.pange(null).trim()).toBe('null');
        });
    });

    describe('necto (bind multi-doc)', () => {
        test('creates multi-document YAML', () => {
            const result = yaml.necto([{ a: 1 }, { b: 2 }]);
            expect(result).toContain('a:');
            expect(result).toContain('---');
            expect(result).toContain('b:');
        });
    });

    // =========================================================================
    // PARSING (solve, tempta, collige)
    // =========================================================================

    describe('solve (parse)', () => {
        test('parses YAML objects', () => {
            const result = yaml.solve('name: Alice\nage: 30');
            expect(result).toEqual({ name: 'Alice', age: 30 });
        });

        test('parses YAML arrays', () => {
            const result = yaml.solve('- a\n- b\n- c');
            expect(result).toEqual(['a', 'b', 'c']);
        });

        test('parses primitives', () => {
            expect(yaml.solve('hello')).toBe('hello');
            expect(yaml.solve('42')).toBe(42);
            expect(yaml.solve('true')).toBe(true);
            expect(yaml.solve('null')).toBe(null);
        });

        test('handles nested structures', () => {
            const input = `
user:
  name: Alice
  address:
    city: Paris
    zip: 75001
`;
            const result = yaml.solve(input);
            expect(result).toEqual({
                user: {
                    name: 'Alice',
                    address: {
                        city: 'Paris',
                        zip: 75001,
                    },
                },
            });
        });
    });

    describe('tempta (try parse)', () => {
        test('returns parsed value on valid YAML', () => {
            expect(yaml.tempta('valid: yaml')).toEqual({ valid: 'yaml' });
        });

        test('returns null on invalid YAML', () => {
            expect(yaml.tempta('[unclosed')).toBe(null);
        });
    });

    describe('collige (gather multi-doc)', () => {
        test('parses multi-document YAML', () => {
            const input = `
a: 1
---
b: 2
---
c: 3
`;
            const result = yaml.collige(input);
            expect(result).toHaveLength(3);
            expect(result[0]).toEqual({ a: 1 });
            expect(result[1]).toEqual({ b: 2 });
            expect(result[2]).toEqual({ c: 3 });
        });
    });

    // =========================================================================
    // TYPE CHECKING
    // =========================================================================

    describe('type checking', () => {
        test('estNihil', () => {
            expect(yaml.estNihil(null)).toBe(true);
            expect(yaml.estNihil(undefined)).toBe(true);
            expect(yaml.estNihil(0)).toBe(false);
            expect(yaml.estNihil('')).toBe(false);
        });

        test('estBivalens', () => {
            expect(yaml.estBivalens(true)).toBe(true);
            expect(yaml.estBivalens(false)).toBe(true);
            expect(yaml.estBivalens(0)).toBe(false);
        });

        test('estNumerus', () => {
            expect(yaml.estNumerus(42)).toBe(true);
            expect(yaml.estNumerus(3.14)).toBe(true);
            expect(yaml.estNumerus('42')).toBe(false);
        });

        test('estTextus', () => {
            expect(yaml.estTextus('hello')).toBe(true);
            expect(yaml.estTextus('')).toBe(true);
            expect(yaml.estTextus(42)).toBe(false);
        });

        test('estLista', () => {
            expect(yaml.estLista([])).toBe(true);
            expect(yaml.estLista([1, 2, 3])).toBe(true);
            expect(yaml.estLista({})).toBe(false);
        });

        test('estTabula', () => {
            expect(yaml.estTabula({})).toBe(true);
            expect(yaml.estTabula({ a: 1 })).toBe(true);
            expect(yaml.estTabula([])).toBe(false);
            expect(yaml.estTabula(null)).toBe(false);
        });
    });

    // =========================================================================
    // VALUE EXTRACTION
    // =========================================================================

    describe('value extraction', () => {
        test('utTextus', () => {
            expect(yaml.utTextus('hello', 'default')).toBe('hello');
            expect(yaml.utTextus(42, 'default')).toBe('default');
        });

        test('utNumerus', () => {
            expect(yaml.utNumerus(42, 0)).toBe(42);
            expect(yaml.utNumerus('42', 0)).toBe(0);
        });

        test('utBivalens', () => {
            expect(yaml.utBivalens(true, false)).toBe(true);
            expect(yaml.utBivalens('true', false)).toBe(false);
        });
    });

    // =========================================================================
    // VALUE ACCESS
    // =========================================================================

    describe('cape (get by key)', () => {
        test('gets object property', () => {
            const obj = { name: 'Alice', age: 30 };
            expect(yaml.cape(obj, 'name')).toBe('Alice');
            expect(yaml.cape(obj, 'missing')).toBe(null);
        });

        test('returns null for non-objects', () => {
            expect(yaml.cape([1, 2], '0')).toBe(null);
            expect(yaml.cape(null, 'x')).toBe(null);
        });
    });

    describe('carpe (pluck by index)', () => {
        test('gets array element', () => {
            const arr = ['a', 'b', 'c'];
            expect(yaml.carpe(arr, 0)).toBe('a');
            expect(yaml.carpe(arr, 2)).toBe('c');
            expect(yaml.carpe(arr, 5)).toBe(null);
        });

        test('returns null for non-arrays', () => {
            expect(yaml.carpe({ 0: 'a' }, 0)).toBe(null);
        });
    });

    describe('inveni (find by path)', () => {
        test('gets nested value', () => {
            const obj = {
                user: { name: 'Alice', tags: ['admin', 'user'] },
            };
            expect(yaml.inveni(obj, 'user.name')).toBe('Alice');
            expect(yaml.inveni(obj, 'user.tags.0')).toBe('admin');
            expect(yaml.inveni(obj, 'user.missing')).toBe(null);
        });

        test('handles missing paths', () => {
            expect(yaml.inveni(null, 'x')).toBe(null);
            expect(yaml.inveni({}, 'a.b')).toBe(null);
        });
    });

    // =========================================================================
    // ROUNDTRIP
    // =========================================================================

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

            const serialized = yaml.pange(original);
            const parsed = yaml.solve(serialized);

            expect(parsed).toEqual(original);
        });
    });

    // =========================================================================
    // YAML-SPECIFIC FEATURES
    // =========================================================================

    describe('YAML-specific features', () => {
        test('parses YAML comments (ignored)', () => {
            const input = `
# This is a comment
name: Alice  # inline comment
age: 30
`;
            const result = yaml.solve(input);
            expect(result).toEqual({ name: 'Alice', age: 30 });
        });

        test('parses multiline strings', () => {
            const input = `
description: |
  This is a
  multiline string
`;
            const result = yaml.solve(input) as { description: string };
            expect(result.description).toContain('This is a');
            expect(result.description).toContain('multiline string');
        });

        test('parses anchors and aliases', () => {
            const input = `
defaults: &defaults
  adapter: postgres
  host: localhost

development:
  <<: *defaults
  database: dev_db

production:
  <<: *defaults
  database: prod_db
`;
            const result = yaml.solve(input) as Record<string, Record<string, string>>;
            expect(result.development?.adapter).toBe('postgres');
            expect(result.development?.database).toBe('dev_db');
            expect(result.production?.adapter).toBe('postgres');
            expect(result.production?.database).toBe('prod_db');
        });
    });
});
