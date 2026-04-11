import { describe, expect, test } from 'bun:test';

import { compile } from './index';

describe('nanus-ts compile', () => {
    test('compiles a simple variable declaration', () => {
        const result = compile('fixum x = 1');

        expect(result.success).toBe(true);
        expect(result.output).toBe('const x = 1;');
    });

    test('compiles a simple function body', () => {
        const result = compile('functio salve() { scribe "hi" }');

        expect(result.success).toBe(true);
        expect(result.output).toContain('function salve()');
        expect(result.output).toContain('console.log("hi");');
    });

    test('reports parse failures without throwing', () => {
        const result = compile('fixum = 1');

        expect(result.success).toBe(false);
        expect(result.error).toContain("expected identifier, got '='");
    });
});
