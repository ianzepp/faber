import { test, expect, describe } from 'bun:test';
import * as prettier from 'prettier';
import plugin from './index.ts';

async function format(code: string): Promise<string> {
    return prettier.format(code, {
        parser: 'faber',
        plugins: [plugin],
        printWidth: 100,
    });
}

describe('prettier plugin', () => {
    describe('loads and parses', () => {
        test('empty file', async () => {
            const result = await format('');
            expect(result).toBe('');
        });

        test('simple variable declaration', async () => {
            const result = await format('fixum x = 42');
            expect(result).toContain('fixum');
            expect(result).toContain('x');
            expect(result).toContain('42');
        });

        test('function declaration', async () => {
            const result = await format('functio salve() { scribe "hello" }');
            expect(result).toContain('functio salve()');
            expect(result).toContain('scribe');
        });
    });

    describe('variable declarations', () => {
        test('varia with initializer', async () => {
            const result = await format('varia nomen = "Marcus"');
            expect(result).toBe('varia nomen = "Marcus"\n');
        });

        test('fixum with type annotation', async () => {
            const result = await format('fixum numerus x = 42');
            expect(result).toBe('fixum numerus x = 42\n');
        });

        test('figendum async binding', async () => {
            const result = await format('figendum data = fetchData()');
            expect(result).toBe('figendum data = fetchData()\n');
        });

        test('array destructuring pattern', async () => {
            const result = await format('fixum [a, b] = arr');
            expect(result).toBe('fixum [a, b] = arr\n');
        });
    });

    describe('control flow', () => {
        test('if statement', async () => {
            const result = await format('si x > 0 { scribe x }');
            expect(result).toContain('si x > 0');
        });

        test('if-else statement', async () => {
            const result = await format('si x > 0 { scribe "pos" } secus { scribe "neg" }');
            expect(result).toContain('si x > 0');
            expect(result).toContain('secus');
        });

        test('while loop', async () => {
            const result = await format('dum x > 0 { x = x - 1 }');
            expect(result).toContain('dum x > 0');
        });

        test('for loop', async () => {
            const result = await format('ex items pro item { scribe item }');
            expect(result).toContain('ex items pro item');
        });

        test('switch statement', async () => {
            const result = await format('elige x { si 1 { scribe "one" } secus { scribe "other" } }');
            expect(result).toContain('elige x');
            expect(result).toContain('si 1');
        });

        test('guard statement', async () => {
            const result = await format('custodi { si x == nihil { redde } }');
            expect(result).toContain('custodi');
        });

        test('break and continue', async () => {
            const result = await format('dum verum { si done { rumpe } perge }');
            expect(result).toContain('rumpe');
            expect(result).toContain('perge');
        });
    });

    describe('expressions', () => {
        test('binary expression', async () => {
            const result = await format('fixum x = 1 + 2 * 3');
            expect(result).toContain('1 + 2 * 3');
        });

        test('member expression', async () => {
            const result = await format('fixum x = obj.prop');
            expect(result).toBe('fixum x = obj.prop\n');
        });

        test('call expression', async () => {
            const result = await format('fixum x = func(a, b, c)');
            expect(result).toContain('func(a, b, c)');
        });

        test('array literal', async () => {
            const result = await format('fixum arr = [1, 2, 3]');
            expect(result).toContain('[1, 2, 3]');
        });

        test('object literal', async () => {
            const result = await format('fixum obj = { a: 1, b: 2 }');
            expect(result).toContain('{ a: 1, b: 2 }');
        });

        test('template literal', async () => {
            const result = await format('fixum s = `hello ${name}`');
            expect(result).toContain('`hello ${name}`');
        });

        test('new expression', async () => {
            const result = await format('fixum p = novum Persona()');
            expect(result).toContain('novum Persona()');
        });

        test('await expression', async () => {
            const result = await format('fixum x = cede fetchData()');
            expect(result).toContain('cede fetchData()');
        });

        test('range expression', async () => {
            const result = await format('ex 0..10 pro i { scribe i }');
            expect(result).toContain('0..10');
        });

        test('this expression', async () => {
            const result = await format('fixum x = ego.nomen');
            expect(result).toContain('ego.nomen');
        });
    });

    describe('declarations', () => {
        test('function with parameters', async () => {
            const result = await format('functio add(numerus a, numerus b) -> numerus { redde a + b }');
            expect(result).toContain('functio add');
            expect(result).toContain('numerus a');
            expect(result).toContain('-> numerus');
        });

        test('async function', async () => {
            const result = await format('functio fetch() futura { redde cede getData() }');
            expect(result).toContain('functio fetch()');
        });

        test('import declaration', async () => {
            const result = await format('ex norma importa scribe');
            expect(result).toBe('ex norma importa scribe\n');
        });

        test('import with multiple specifiers', async () => {
            const result = await format('ex norma importa scribe, lege');
            expect(result).toBe('ex norma importa scribe, lege\n');
        });

        test('import with alias', async () => {
            const result = await format('ex norma importa scribe ut s');
            expect(result).toBe('ex norma importa scribe ut s\n');
        });

        test('wildcard import', async () => {
            const result = await format('ex norma importa *');
            expect(result).toBe('ex norma importa *\n');
        });

        test('destructure declaration', async () => {
            const result = await format('ex persona fixum nomen, aetas');
            expect(result).toBe('ex persona fixum nomen, aetas\n');
        });

        test('destructure with alias', async () => {
            const result = await format('ex persona fixum nomen ut n, aetas ut a');
            expect(result).toBe('ex persona fixum nomen ut n, aetas ut a\n');
        });

        test('type alias', async () => {
            const result = await format('typus ID = textus');
            expect(result).toBe('typus ID = textus\n');
        });
    });

    describe('OOP features', () => {
        test('enum declaration', async () => {
            const result = await format('ordo Color { rubrum, viridis, caeruleum }');
            expect(result).toContain('ordo Color');
            expect(result).toContain('rubrum');
        });

        test('enum with values', async () => {
            const result = await format('ordo Status { pendens = 0, actum = 1 }');
            expect(result).toContain('pendens = 0');
        });

        test('genus declaration', async () => {
            const result = await format('genus Persona { textus nomen }');
            expect(result).toContain('genus Persona');
            expect(result).toContain('textus nomen');
        });

        test('genus with methods', async () => {
            const result = await format('genus Persona { textus nomen functio salve() { scribe ego.nomen } }');
            expect(result).toContain('genus Persona');
            expect(result).toContain('functio salve()');
        });

        test('pactum declaration', async () => {
            const result = await format('pactum Iterabilis { functio sequens() -> textus? }');
            expect(result).toContain('pactum Iterabilis');
            expect(result).toContain('functio sequens()');
        });
    });

    describe('error handling', () => {
        test('try-catch', async () => {
            const result = await format('tempta { riskyOp() } cape err { scribe err }');
            expect(result).toContain('tempta');
            expect(result).toContain('cape err');
        });

        test('try-catch-finally', async () => {
            const result = await format('tempta { op() } cape e { handle(e) } demum { cleanup() }');
            expect(result).toContain('tempta');
            expect(result).toContain('cape');
            expect(result).toContain('demum');
        });

        test('throw statement', async () => {
            const result = await format('iace "error occurred"');
            expect(result).toBe('iace "error occurred"\n');
        });

        test('assert statement', async () => {
            const result = await format('adfirma x > 0, "x must be positive"');
            expect(result).toContain('adfirma x > 0');
        });
    });

    describe('fac blocks and lambdas', () => {
        test('fac block', async () => {
            const result = await format('fac { doSomething() }');
            expect(result).toContain('fac');
        });

        test('fac with catch', async () => {
            const result = await format('fac { riskyOp() } cape err { handle(err) }');
            expect(result).toContain('fac');
            expect(result).toContain('cape err');
        });

        test('pro lambda expression', async () => {
            const result = await format('fixum double = pro x: x * 2');
            expect(result).toContain('pro x: x * 2');
        });
    });

    describe('formatting', () => {
        test('preserves blank lines between statements', async () => {
            const input = `fixum a = 1

fixum b = 2`;
            const result = await format(input);
            expect(result).toContain('\n\n');
        });

        test('breaks long parameter lists', async () => {
            const result = await format('functio longFunc(numerus aaa, numerus bbb, numerus ccc) { redde aaa }');
            expect(result).toContain('functio longFunc');
        });
    });
});
