import { describe, test, expect } from 'bun:test';
import { codex } from './codex';

describe('codex', () => {
    describe('base64', () => {
        test('encodes bytes to base64', () => {
            const input = new TextEncoder().encode('Hello, World!');
            expect(codex.base64(input)).toBe('SGVsbG8sIFdvcmxkIQ==');
        });

        test('decodes base64 to bytes', () => {
            const result = codex.deBase64('SGVsbG8sIFdvcmxkIQ==');
            expect(new TextDecoder().decode(result)).toBe('Hello, World!');
        });

        test('temptaBase64 returns null on invalid input', () => {
            expect(codex.temptaBase64('not valid base64!!!')).toBeNull();
        });

        test('temptaBase64 returns bytes on valid input', () => {
            const result = codex.temptaBase64('SGVsbG8=');
            expect(result).not.toBeNull();
            expect(new TextDecoder().decode(result!)).toBe('Hello');
        });
    });

    describe('hex', () => {
        test('encodes bytes to hex', () => {
            const input = new Uint8Array([0xde, 0xad, 0xbe, 0xef]);
            expect(codex.hex(input)).toBe('deadbeef');
        });

        test('decodes hex to bytes', () => {
            const result = codex.deHex('deadbeef');
            expect(Array.from(result)).toEqual([0xde, 0xad, 0xbe, 0xef]);
        });

        test('deHex throws on odd length', () => {
            expect(() => codex.deHex('abc')).toThrow();
        });

        test('deHex throws on invalid characters', () => {
            expect(() => codex.deHex('ghij')).toThrow();
        });

        test('temptaHex returns null on invalid input', () => {
            expect(codex.temptaHex('abc')).toBeNull();
            expect(codex.temptaHex('ghij')).toBeNull();
        });

        test('temptaHex returns bytes on valid input', () => {
            const result = codex.temptaHex('cafe');
            expect(result).not.toBeNull();
            expect(Array.from(result!)).toEqual([0xca, 0xfe]);
        });
    });

    describe('url encoding', () => {
        test('url encodes special characters', () => {
            expect(codex.url('hello world')).toBe('hello%20world');
        });

        test('deUrl decodes percent-encoding', () => {
            expect(codex.deUrl('hello%20world')).toBe('hello world');
        });

        test('urlComponentum encodes query components', () => {
            expect(codex.urlComponentum('a=b&c=d')).toBe('a%3Db%26c%3Dd');
        });

        test('deUrlComponentum decodes query components', () => {
            expect(codex.deUrlComponentum('a%3Db%26c%3Dd')).toBe('a=b&c=d');
        });
    });
});
