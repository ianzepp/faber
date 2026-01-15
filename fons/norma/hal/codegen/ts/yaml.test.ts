import { test, expect, describe } from 'bun:test';
import { yaml } from './yaml';

describe('yaml HAL', () => {
    describe('serialization', () => {
        test('solve serializes to YAML', () => {
            const result = yaml.solve({ name: 'Alice', age: 30 });
            expect(result).toContain('name:');
            expect(result).toContain('Alice');
            expect(result).toContain('age:');
            expect(result).toContain('30');
        });

        test('solve handles arrays', () => {
            const result = yaml.solve(['a', 'b', 'c']);
            // Bun YAML may use compact [a,b,c] or expanded - a format
            expect(result).toContain('a');
            expect(result).toContain('b');
            expect(result).toContain('c');
        });

        test('solve handles primitives', () => {
            expect(yaml.solve('hello').trim()).toBe('hello');
            expect(yaml.solve(42).trim()).toBe('42');
            expect(yaml.solve(true).trim()).toBe('true');
            expect(yaml.solve(null).trim()).toBe('null');
        });

        test('solveMulti creates multi-document YAML', () => {
            const result = yaml.solveMulti([{ a: 1 }, { b: 2 }]);
            expect(result).toContain('a:');
            expect(result).toContain('---');
            expect(result).toContain('b:');
        });
    });

    describe('deserialization', () => {
        test('pange parses YAML objects', () => {
            const result = yaml.pange('name: Alice\nage: 30');
            expect(result).toEqual({ name: 'Alice', age: 30 });
        });

        test('pange parses YAML arrays', () => {
            const result = yaml.pange('- a\n- b\n- c');
            expect(result).toEqual(['a', 'b', 'c']);
        });

        test('pange parses primitives', () => {
            expect(yaml.pange('hello')).toBe('hello');
            expect(yaml.pange('42')).toBe(42);
            expect(yaml.pange('true')).toBe(true);
            expect(yaml.pange('null')).toBe(null);
        });

        test('pange handles nested structures', () => {
            const input = `
user:
  name: Alice
  address:
    city: Paris
    zip: 75001
`;
            const result = yaml.pange(input);
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

        test('pangeTuto returns null on invalid YAML', () => {
            expect(yaml.pangeTuto('valid: yaml')).toEqual({ valid: 'yaml' });
            // Most things are valid YAML, but unbalanced brackets aren't
            expect(yaml.pangeTuto('[unclosed')).toBe(null);
        });

        test('pangeMulti parses multi-document YAML', () => {
            const input = `
a: 1
---
b: 2
---
c: 3
`;
            const result = yaml.pangeMulti(input);
            expect(result).toHaveLength(3);
            expect(result[0]).toEqual({ a: 1 });
            expect(result[1]).toEqual({ b: 2 });
            expect(result[2]).toEqual({ c: 3 });
        });
    });

    describe('type checking', () => {
        test('estNihil', () => {
            expect(yaml.estNihil(null)).toBe(true);
            expect(yaml.estNihil(undefined)).toBe(true);
            expect(yaml.estNihil(0)).toBe(false);
            expect(yaml.estNihil('')).toBe(false);
        });

        test('estTextus', () => {
            expect(yaml.estTextus('hello')).toBe(true);
            expect(yaml.estTextus('')).toBe(true);
            expect(yaml.estTextus(42)).toBe(false);
        });

        test('estNumerus', () => {
            expect(yaml.estNumerus(42)).toBe(true);
            expect(yaml.estNumerus(3.14)).toBe(true);
            expect(yaml.estNumerus('42')).toBe(false);
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

            const serialized = yaml.solve(original);
            const parsed = yaml.pange(serialized);

            expect(parsed).toEqual(original);
        });
    });

    describe('YAML-specific features', () => {
        test('parses YAML comments (ignored)', () => {
            const input = `
# This is a comment
name: Alice  # inline comment
age: 30
`;
            const result = yaml.pange(input);
            expect(result).toEqual({ name: 'Alice', age: 30 });
        });

        test('parses multiline strings', () => {
            const input = `
description: |
  This is a
  multiline string
`;
            const result = yaml.pange(input) as { description: string };
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
            const result = yaml.pange(input) as Record<string, Record<string, string>>;
            expect(result.development?.adapter).toBe('postgres');
            expect(result.development?.database).toBe('dev_db');
            expect(result.production?.adapter).toBe('postgres');
            expect(result.production?.database).toBe('prod_db');
        });
    });
});
