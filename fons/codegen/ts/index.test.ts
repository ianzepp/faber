import { test, expect, describe } from 'bun:test';
import { tokenize } from '../../tokenizer';
import { parse } from '../../parser';
import { generate } from '../index';

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
        test('varia -> let', () => {
            const js = compile('varia nomen = "Marcus"');

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

        test('fit returns sync function', () => {
            const js = compile('functio greet() fit textus { redde "hello" }');

            expect(js).toContain('function greet(): string');
            expect(js).not.toContain('async');
            expect(js).not.toContain('*');
        });

        test('fiet returns async function with Promise<T>', () => {
            const js = compile('functio fetch() fiet textus { redde "data" }');

            expect(js).toContain('async function fetch(): Promise<string>');
        });

        test('fiunt returns generator function with Generator<T>', () => {
            const js = compile('functio range() fiunt numerus { redde 1 }');

            expect(js).toContain('function* range(): Generator<number>');
        });

        test('fient returns async generator function with AsyncGenerator<T>', () => {
            const js = compile('functio stream() fient textus { redde "chunk" }');

            expect(js).toContain('async function* stream(): AsyncGenerator<string>');
        });

        test('cursor prefix with fiunt is redundant but valid', () => {
            const js = compile('cursor functio range() fiunt numerus { redde 1 }');

            expect(js).toContain('function* range(): Generator<number>');
        });

        test('futura prefix with fiet is redundant but valid', () => {
            const js = compile('futura functio fetch() fiet textus { redde "data" }');

            expect(js).toContain('async function fetch(): Promise<string>');
        });

        test('cursor prefix makes generator without verb', () => {
            const js = compile('cursor functio range() -> numerus { redde 1 }');

            expect(js).toContain('function* range(): Generator<number>');
        });

        test('futura cursor makes async generator', () => {
            const js = compile('futura cursor functio stream() -> textus { redde "chunk" }');

            expect(js).toContain('async function* stream(): AsyncGenerator<string>');
        });

        test('cede emits yield in generator functions', () => {
            const js = compile('functio range() fiunt numerus { cede 1 }');

            expect(js).toContain('yield 1');
            expect(js).not.toContain('await');
        });

        test('cede emits await in async functions', () => {
            const js = compile('functio fetch() fiet textus { cede getData() }');

            expect(js).toContain('await getData()');
            expect(js).not.toContain('yield');
        });

        test('cede emits yield in async generators', () => {
            const js = compile('functio stream() fient textus { cede "chunk" }');

            expect(js).toContain('yield "chunk"');
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

        test('ex...fit generates sync for...of', () => {
            const js = compile('ex items fit item { scribe item }');

            expect(js).toContain('for (const item of items)');
            expect(js).not.toContain('await');
        });

        test('ex...fiet generates for await...of', () => {
            const js = compile('ex stream fiet chunk { scribe chunk }');

            expect(js).toContain('for await (const chunk of stream)');
        });

        test('in...fit generates sync for...in', () => {
            const js = compile('in obj fit key { scribe key }');

            expect(js).toContain('for (const key in obj)');
            expect(js).not.toContain('await');
        });

        test('in...fiet generates for await...in', () => {
            const js = compile('in asyncObj fiet key { scribe key }');

            expect(js).toContain('for await (const key in asyncObj)');
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

        test('ternary with ? :', () => {
            const js = compile('verum ? 1 : 0');

            expect(js).toBe('true ? 1 : 0;');
        });

        test('ternary with sic secus', () => {
            const js = compile('verum sic 1 secus 0');

            expect(js).toBe('true ? 1 : 0;');
        });

        test('ternary with condition', () => {
            const js = compile('x > 5 ? "big" : "small"');

            expect(js).toBe('(x > 5) ? "big" : "small";');
        });

        test('nested ternary', () => {
            const js = compile('a ? b ? c : d : e');

            expect(js).toBe('a ? b ? c : d : e;');
        });

        test('ternary in variable', () => {
            const js = compile('varia x = verum ? 1 : 0');

            expect(js).toBe('let x = true ? 1 : 0;');
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
            const js = compile('novum erratum(message)');

            expect(js).toBe('new erratum(message);');
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
            const js = compile('typus ID = textus');

            expect(js).toBe('type ID = string;');
        });

        test('type alias with generic', () => {
            const js = compile('typus StringList = lista<textus>');

            expect(js).toBe('type StringList = Array<string>;');
        });

        test('type with numeric parameter is ignored in TS', () => {
            const js = compile('typus SmallNum = numerus<32>');

            expect(js).toBe('type SmallNum = number;');
        });

        test('type with size type parameter', () => {
            // i32, u64 etc. are parsed as type annotations but ignored in TS
            const js = compile('typus Int32 = numerus<i32>');

            expect(js).toBe('type Int32 = number<i32>;');
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
            const js = compile('varia { count } = data');

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

    describe('genus declarations', () => {
        test('genus generates auto-merge constructor', () => {
            const js = compile('genus persona { textus nomen: "X" }');

            expect(js).toContain('class persona');
            expect(js).toContain('nomen: string = "X"');
            expect(js).toContain('constructor(overrides: { nomen?: string } = {})');
            expect(js).toContain('if (overrides.nomen !== undefined) this.nomen = overrides.nomen');
        });

        test('genus with creo emits private no-args method', () => {
            const js = compile(`
                genus persona {
                    numerus aetas: 0
                    functio creo() {
                        si ego.aetas < 0 { ego.aetas = 0 }
                    }
                }
            `);

            expect(js).toContain('this.creo();');
            expect(js).toContain('private creo()');
        });

        test('genus without creo does not call creo', () => {
            const js = compile('genus persona { textus nomen: "X" }');

            expect(js).not.toContain('this.creo()');
            expect(js).not.toContain('private creo()');
        });

        test('novum without cum passes empty object', () => {
            const js = compile('fixum p = novum persona');

            expect(js).toBe('const p = new persona();');
        });

        test('novum with cum passes overrides', () => {
            const js = compile('fixum p = novum persona cum { nomen: "Claudia" }');

            expect(js).toBe('const p = new persona({ nomen: "Claudia" });');
        });

        test('ego becomes this', () => {
            const js = compile(`
                genus persona {
                    textus nomen: "X"
                    functio saluta() { redde ego.nomen }
                }
            `);

            expect(js).toContain('return this.nomen');
        });
    });

    describe('missing features - nulla/nonnulla operators', () => {
        test('nulla generates null check', () => {
            const js = compile('si nulla x { scribe "empty" }');

            expect(js).toContain('if');
            expect(js).toContain('null');
        });

        test('nonnulla generates non-null check', () => {
            const js = compile('si nonnulla items { scribe "has items" }');

            expect(js).toContain('if');
        });

        test('nulla on array', () => {
            const js = compile('fixum check = nulla []');

            expect(js).toContain('const check');
        });

        test('nonnulla on object', () => {
            const js = compile('fixum check = nonnulla { a: 1 }');

            expect(js).toContain('const check');
        });

        test('nulla in conditional', () => {
            const js = compile('fixum result = nulla x && verum');

            expect(js).toContain('&&');
            expect(js).toContain('true');
        });
    });

    describe('missing features - iace (throw)', () => {
        test('iace string becomes throw', () => {
            const js = compile('iace "error message"');

            expect(js).toContain('throw');
            expect(js).toContain('error message');
        });

        test('iace with Error constructor', () => {
            const js = compile('iace novum Error("msg")');

            expect(js).toContain('throw');
            expect(js).toContain('new Error');
        });

        test('iace variable', () => {
            const js = compile('iace err');

            expect(js).toContain('throw');
            expect(js).toContain('err');
        });

        test('iace in function', () => {
            const js = compile(`
                functio validate() {
                    iace "invalid"
                }
            `);

            expect(js).toContain('function validate');
            expect(js).toContain('throw');
        });

        test('iace in switch case', () => {
            const js = compile(`
                elige x {
                    si 1 { iace "error" }
                }
            `);

            expect(js).toContain('switch');
            expect(js).toContain('throw');
        });
    });

    describe('missing features - ergo one-liners', () => {
        test('si with ergo generates inline if', () => {
            const js = compile('si x > 5 ergo scribe "big"');

            expect(js).toContain('if');
            expect(js).toContain('x > 5');
        });

        test('dum with ergo generates while', () => {
            const js = compile('dum x > 0 ergo x = x - 1');

            expect(js).toContain('while');
            expect(js).toContain('x > 0');
        });

        test('ex...pro with ergo', () => {
            const js = compile('ex items pro item ergo scribe item');

            expect(js).toContain('for');
            expect(js).toContain('of items');
        });
    });

    describe('missing features - scribe with multiple arguments', () => {
        test('scribe two args becomes console.log', () => {
            const js = compile('scribe "Name:", name');

            expect(js).toContain('console.log');
            expect(js).toContain('Name:');
        });

        test('scribe multiple args', () => {
            const js = compile('scribe "a", "b", "c"');

            expect(js).toContain('console.log');
        });

        test('scribe with expressions', () => {
            const js = compile('scribe x + y, a * b');

            expect(js).toContain('console.log');
        });

        test('scribe with function calls', () => {
            const js = compile('scribe getName(), getAge()');

            expect(js).toContain('console.log');
            expect(js).toContain('getName()');
        });
    });

    describe('lista methods - Latin array API', () => {
        describe('adding elements', () => {
            test('adde -> push (mutating)', () => {
                const js = compile('items.adde(x)');
                expect(js).toBe('items.push(x);');
            });

            test('addita -> spread (copying)', () => {
                const js = compile('items.addita(x)');
                expect(js).toBe('[...items, x];');
            });

            test('praepone -> unshift (mutating)', () => {
                const js = compile('items.praepone(x)');
                expect(js).toBe('items.unshift(x);');
            });

            test('praeposita -> spread (copying)', () => {
                const js = compile('items.praeposita(x)');
                expect(js).toBe('[x, ...items];');
            });
        });

        describe('removing elements', () => {
            test('remove -> pop (mutating)', () => {
                const js = compile('items.remove()');
                expect(js).toBe('items.pop();');
            });

            test('remota -> slice (copying)', () => {
                const js = compile('items.remota()');
                expect(js).toBe('items.slice(0, -1);');
            });

            test('decapita -> shift (mutating)', () => {
                const js = compile('items.decapita()');
                expect(js).toBe('items.shift();');
            });

            test('decapitata -> slice (copying)', () => {
                const js = compile('items.decapitata()');
                expect(js).toBe('items.slice(1);');
            });

            test('purga -> length = 0 (mutating)', () => {
                const js = compile('items.purga()');
                expect(js).toBe('items.length = 0;');
            });
        });

        describe('accessing elements', () => {
            test('primus -> [0]', () => {
                const js = compile('items.primus()');
                expect(js).toBe('items[0];');
            });

            test('ultimus -> at(-1)', () => {
                const js = compile('items.ultimus()');
                expect(js).toBe('items.at(-1);');
            });

            test('accipe -> indexed access', () => {
                const js = compile('items.accipe(2)');
                expect(js).toBe('items[2];');
            });
        });

        describe('properties', () => {
            test('longitudo -> length', () => {
                const js = compile('items.longitudo()');
                expect(js).toBe('items.length;');
            });

            test('vacua -> length === 0', () => {
                const js = compile('items.vacua()');
                expect(js).toBe('items.length === 0;');
            });
        });

        describe('searching', () => {
            test('continet -> includes', () => {
                const js = compile('items.continet(x)');
                expect(js).toBe('items.includes(x);');
            });

            test('indiceDe -> indexOf', () => {
                const js = compile('items.indiceDe(x)');
                expect(js).toBe('items.indexOf(x);');
            });

            test('inveni -> find', () => {
                const js = compile('items.inveni(fn)');
                expect(js).toBe('items.find(fn);');
            });

            test('inveniIndicem -> findIndex', () => {
                const js = compile('items.inveniIndicem(fn)');
                expect(js).toBe('items.findIndex(fn);');
            });
        });

        describe('transformations', () => {
            test('filtrata -> filter', () => {
                const js = compile('items.filtrata(fn)');
                expect(js).toBe('items.filter(fn);');
            });

            test('mappata -> map', () => {
                const js = compile('items.mappata(fn)');
                expect(js).toBe('items.map(fn);');
            });

            test('reducta -> reduce (args swapped)', () => {
                const js = compile('items.reducta(0, (acc, n) => acc + n)');
                expect(js).toBe('items.reduce((acc, n) => (acc + n), 0);');
            });

            test('inversa -> spread + reverse (copying)', () => {
                const js = compile('items.inversa()');
                expect(js).toBe('[...items].reverse();');
            });

            test('ordinata -> spread + sort (copying)', () => {
                const js = compile('items.ordinata()');
                expect(js).toBe('[...items].sort();');
            });

            test('ordinata with comparator', () => {
                const js = compile('items.ordinata(fn)');
                expect(js).toBe('[...items].sort(fn);');
            });

            test('sectio -> slice', () => {
                const js = compile('items.sectio(1, 3)');
                expect(js).toBe('items.slice(1, 3);');
            });

            test('prima -> slice(0, n)', () => {
                const js = compile('items.prima(5)');
                expect(js).toBe('items.slice(0, 5);');
            });

            test('ultima -> slice(-n)', () => {
                const js = compile('items.ultima(3)');
                expect(js).toBe('items.slice(-3);');
            });

            test('omitte -> slice(n)', () => {
                const js = compile('items.omitte(2)');
                expect(js).toBe('items.slice(2);');
            });
        });

        describe('predicates', () => {
            test('omnes -> every', () => {
                const js = compile('items.omnes(fn)');
                expect(js).toBe('items.every(fn);');
            });

            test('aliquis -> some', () => {
                const js = compile('items.aliquis(fn)');
                expect(js).toBe('items.some(fn);');
            });
        });

        describe('other methods', () => {
            test('coniunge -> join', () => {
                const js = compile('items.coniunge(", ")');
                expect(js).toBe('items.join(", ");');
            });

            test('perambula -> forEach', () => {
                const js = compile('items.perambula(fn)');
                expect(js).toBe('items.forEach(fn);');
            });

            test('plana -> flat', () => {
                const js = compile('items.plana()');
                expect(js).toBe('items.flat();');
            });

            test('explanata -> flatMap', () => {
                const js = compile('items.explanata(fn)');
                expect(js).toBe('items.flatMap(fn);');
            });
        });

        describe('chaining', () => {
            test('method chain is preserved', () => {
                const js = compile('items.filtrata(f).mappata(g)');
                expect(js).toBe('items.filter(f).map(g);');
            });

            test('complex chain', () => {
                const js = compile('items.filtrata(f).mappata(g).prima(10)');
                expect(js).toBe('items.filter(f).map(g).slice(0, 10);');
            });
        });

        describe('with arrow functions', () => {
            test('filtrata with arrow', () => {
                const js = compile('items.filtrata((x) => x > 0)');
                expect(js).toBe('items.filter((x) => (x > 0));');
            });

            test('mappata with arrow', () => {
                const js = compile('items.mappata((x) => x * 2)');
                expect(js).toBe('items.map((x) => (x * 2));');
            });
        });
    });
});
