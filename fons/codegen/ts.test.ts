import { test, expect, describe } from 'bun:test';
import { tokenize } from '../tokenizer';
import { parse } from '../parser';
import { generate } from './index';

function compile(code: string): string {
    const { tokens } = tokenize(code);
    const { program } = parse(tokens);

    if (!program) {
        throw new Error('Parse failed');
    }

    return generate(program);
}

describe('codegen', () => {
    describe('variable declarations', () => {
        test('esto -> let', () => {
            const js = compile('esto nomen = "Marcus"');

            expect(js).toBe('let nomen = "Marcus";');
        });

        test('fixum -> const', () => {
            const js = compile('fixum PI = 3.14159');

            expect(js).toBe('const PI = 3.14159;');
        });
    });

    describe('function declarations', () => {
        test('simple function', () => {
            const js = compile(`
        functio salve(nomen) {
          redde nomen
        }
      `);

            expect(js).toContain('function salve(nomen)');
            expect(js).toContain('return nomen;');
        });

        test('async function', () => {
            const js = compile(`
        futura functio fetch(url) {
          redde data
        }
      `);

            expect(js).toContain('async function fetch(url)');
        });
    });

    describe('if statements', () => {
        test('simple if', () => {
            const js = compile(`
        si verum {
          _scribe("yes")
        }
      `);

            expect(js).toContain('if (true)');
            expect(js).toContain('console.log("yes")');
        });

        test('if with else', () => {
            const js = compile(`
        si falsum {
          a()
        }
        aliter {
          b()
        }
      `);

            expect(js).toContain('if (false)');
            expect(js).toContain('} else {');
        });

        test('if with cape wraps in try', () => {
            const js = compile(`
        si riskyCall() {
          process()
        }
        cape erratum {
          handleError()
        }
      `);

            expect(js).toContain('try {');
            expect(js).toContain('catch (erratum)');
        });
    });

    describe('loops', () => {
        test('while loop', () => {
            const js = compile(`
        dum verum {
          _scribe("loop")
        }
      `);

            expect(js).toContain('while (true)');
        });

        test('for...in loop', () => {
            const js = compile(`
        in lista pro item {
          _scribe(item)
        }
      `);

            expect(js).toContain('for (const item in lista)');
        });

        test('for...of loop', () => {
            const js = compile(`
        ex numeros pro numero {
          _scribe(numero)
        }
      `);

            expect(js).toContain('for (const numero of numeros)');
        });
    });

    describe('expressions', () => {
        test('binary operators', () => {
            const js = compile('1 + 2');

            expect(js).toBe('(1 + 2);');
        });

        test('Latin logical operators become JS', () => {
            const js = compile('a et b');

            expect(js).toBe('(a && b);');
        });

        test('aut becomes ||', () => {
            const js = compile('a aut b');

            expect(js).toBe('(a || b);');
        });

        test('function call', () => {
            const js = compile('salve(nomen)');

            expect(js).toBe('salve(nomen);');
        });

        test('method call', () => {
            const js = compile('lista.filter(f)');

            expect(js).toBe('lista.filter(f);');
        });

        test('member access', () => {
            const js = compile('usuario.nomen');

            expect(js).toBe('usuario.nomen;');
        });
    });

    describe('arrow functions', () => {
        test('simple arrow', () => {
            const js = compile('(x) => x');

            expect(js).toBe('(x) => x;');
        });

        test('arrow with block', () => {
            const js = compile('(x) => { redde x }');

            expect(js).toContain('(x) =>');
            expect(js).toContain('return x;');
        });
    });

    describe('special expressions', () => {
        test('cede -> await', () => {
            const js = compile('cede fetch(url)');

            expect(js).toBe('await fetch(url);');
        });

        test('novum -> new', () => {
            const js = compile('novum Erratum(message)');

            expect(js).toBe('new Erratum(message);');
        });

        test('verum -> true', () => {
            const js = compile('verum');

            expect(js).toBe('true;');
        });

        test('falsum -> false', () => {
            const js = compile('falsum');

            expect(js).toBe('false;');
        });

        test('nihil -> null', () => {
            const js = compile('nihil');

            expect(js).toBe('null;');
        });
    });

    describe('try/catch/finally', () => {
        test('tempta/cape', () => {
            const js = compile(`
        tempta {
          riskyCode()
        }
        cape error {
          handleError()
        }
      `);

            expect(js).toContain('try {');
            expect(js).toContain('catch (error)');
        });

        test('with demum (finally)', () => {
            const js = compile(`
        tempta {
          riskyCode()
        }
        cape error {
          handleError()
        }
        demum {
          cleanup()
        }
      `);

            expect(js).toContain('finally {');
            expect(js).toContain('cleanup()');
        });
    });

    describe('type declarations', () => {
        test('type alias declaration', () => {
            const js = compile('typus ID = Textus');

            expect(js).toBe('type ID = string;');
        });

        test('type alias with generic', () => {
            const js = compile('typus StringList = Lista<Textus>');

            expect(js).toBe('type StringList = Array<string>;');
        });

        test('type with numeric parameter is ignored in TS', () => {
            const js = compile('typus SmallNum = Numerus<32>');

            expect(js).toBe('type SmallNum = number;');
        });

        test('type with modifier parameter is ignored in TS', () => {
            const js = compile('typus Natural = Numerus<Naturalis>');

            expect(js).toBe('type Natural = number;');
        });

        test('type with both numeric and modifier parameters', () => {
            const js = compile('typus UInt32 = Numerus<32, Naturalis>');

            expect(js).toBe('type UInt32 = number;');
        });
    });

    describe('complete programs', () => {
        test('hello world', () => {
            const js = compile(`
        functio salve(nomen) {
          redde "Salve, " + nomen
        }
        _scribe(salve("Mundus"))
      `);

            expect(js).toContain('function salve(nomen)');
            expect(js).toContain('return ("Salve, " + nomen);');
            expect(js).toContain('console.log(salve("Mundus"))');
        });
    });

    describe('guard statements', () => {
        test('custodi with single clause', () => {
            const js = compile(`
        custodi {
          si x == nihil { redde }
        }
      `);

            expect(js).toContain('if ((x == null))');
            expect(js).toContain('return;');
        });

        test('custodi with multiple clauses', () => {
            const js = compile(`
        custodi {
          si x < 0 { redde }
          si y == nihil { iace "error" }
        }
      `);

            expect(js).toContain('if ((x < 0))');
            expect(js).toContain('if ((y == null))');
            expect(js).toContain('throw "error"');
        });
    });

    describe('assert statements', () => {
        test('adfirma without message', () => {
            const js = compile('adfirma x > 0');

            expect(js).toContain('if (!(');
            expect(js).toContain('throw new Error(');
            expect(js).toContain('Assertion failed');
        });

        test('adfirma with message', () => {
            const js = compile('adfirma x > 0, "x must be positive"');

            expect(js).toContain('if (!(');
            expect(js).toContain('throw new Error("x must be positive")');
        });
    });

    describe('range expressions', () => {
        test('simple range in for loop', () => {
            const js = compile(`
        ex 0..10 pro i {
          _scribe(i)
        }
      `);

            expect(js).toContain('for (let i = 0; i <= 10; i++)');
        });

        test('range with step', () => {
            const js = compile(`
        ex 0..10 per 2 pro i {
          _scribe(i)
        }
      `);

            expect(js).toContain('for (let i = 0; i <= 10; i += 2)');
        });

        test('range with expressions', () => {
            const js = compile(`
        ex 1..n pro i {
          _scribe(i)
        }
      `);

            expect(js).toContain('for (let i = 1; i <= n; i++)');
        });
    });

    describe('switch statements', () => {
        test('elige with cases', () => {
            const js = compile(`
        elige x {
          si 1 { a() }
          si 2 { b() }
        }
      `);

            expect(js).toContain('switch (x)');
            expect(js).toContain('case 1:');
            expect(js).toContain('case 2:');
            expect(js).toContain('break;');
        });

        test('elige with default', () => {
            const js = compile(`
        elige x {
          si 1 { a() }
          aliter { c() }
        }
      `);

            expect(js).toContain('switch (x)');
            expect(js).toContain('case 1:');
            expect(js).toContain('default:');
        });
    });

    describe('with statements', () => {
        test('cum block transforms assignments', () => {
            const js = compile(`
        cum user {
          nomen = "Marcus"
          aetas = 30
        }
      `);

            expect(js).toContain('user.nomen = "Marcus"');
            expect(js).toContain('user.aetas = 30');
        });
    });

    describe('object destructuring', () => {
        test('simple destructuring', () => {
            const js = compile('fixum { nomen, aetas } = user');

            expect(js).toBe('const { nomen, aetas } = user;');
        });

        test('destructuring with rename', () => {
            const js = compile('fixum { nomen: localName } = user');

            expect(js).toBe('const { nomen: localName } = user;');
        });

        test('mutable destructuring', () => {
            const js = compile('esto { count } = data');

            expect(js).toBe('let { count } = data;');
        });
    });

    describe('object literals', () => {
        test('empty object', () => {
            const js = compile('fixum x = {}');

            expect(js).toBe('const x = {};');
        });

        test('object with properties', () => {
            const js = compile('fixum user = { nomen: "Marcus", aetas: 30 }');

            expect(js).toBe('const user = { nomen: "Marcus", aetas: 30 };');
        });

        test('object with expression values', () => {
            const js = compile('fixum data = { sum: 1 + 2, active: verum }');

            expect(js).toBe('const data = { sum: (1 + 2), active: true };');
        });
    });

    describe('array literals', () => {
        test('empty array', () => {
            const js = compile('fixum arr = []');

            expect(js).toBe('const arr = [];');
        });

        test('array with elements', () => {
            const js = compile('fixum nums = [1, 2, 3]');

            expect(js).toBe('const nums = [1, 2, 3];');
        });
    });
});
