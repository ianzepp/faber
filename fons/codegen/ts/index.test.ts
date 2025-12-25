import { test, expect, describe } from 'bun:test';
import { tokenize } from '../../tokenizer';
import { parse } from '../../parser';
import { analyze } from '../../semantic';
import { generate } from '../index';

function compile(code: string): string {
    const { tokens } = tokenize(code);
    const { program } = parse(tokens);

    if (!program) {
        throw new Error('Parse failed');
    }

    // WHY: Run semantic analysis to populate resolvedType on AST nodes.
    // This is required for correct collection method dispatch.
    const { program: analyzedProgram } = analyze(program);

    return generate(analyzedProgram);
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

        test('figendum -> const with await', () => {
            const js = compile(`
                futura functio fetchData() -> textus { redde "data" }
                figendum data = fetchData()
            `);

            expect(js).toContain('const data = await fetchData()');
        });

        test('variandum -> let with await', () => {
            const js = compile(`
                futura functio getResult() -> numerus { redde 42 }
                variandum result = getResult()
            `);

            expect(js).toContain('let result = await getResult()');
        });

        test('figendum with type annotation', () => {
            const js = compile(`
                futura functio fetch() -> textus { redde "x" }
                figendum textus data = fetch()
            `);

            expect(js).toContain('const data: string = await fetch()');
        });

        test('variandum with type annotation', () => {
            const js = compile(`
                futura functio getCount() -> numerus { redde 1 }
                variandum numerus count = getCount()
            `);

            expect(js).toContain('let count: number = await getCount()');
        });

        test('multiple async bindings', () => {
            const js = compile(`
                futura functio fetchA() -> textus { redde "a" }
                futura functio fetchB() -> textus { redde "b" }
                figendum a = fetchA()
                figendum b = fetchB()
            `);

            expect(js).toContain('const a = await fetchA()');
            expect(js).toContain('const b = await fetchB()');
        });

        test('mixed async and sync bindings', () => {
            const js = compile(`
                futura functio fetch() -> textus { redde "x" }
                fixum sync = "static"
                figendum async = fetch()
            `);

            expect(js).toContain('const sync = "static"');
            expect(js).toContain('const async = await fetch()');
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

        test('rumpe generates break', () => {
            const js = compile(`
                dum verum {
                    rumpe
                }
            `);

            expect(js).toContain('while (true)');
            expect(js).toContain('break;');
        });

        test('perge generates continue', () => {
            const js = compile(`
                dum verum {
                    perge
                }
            `);

            expect(js).toContain('while (true)');
            expect(js).toContain('continue;');
        });

        test('rumpe in for loop', () => {
            const js = compile(`
                ex items pro item {
                    si item == nihil { rumpe }
                }
            `);

            expect(js).toContain('for (const item of items)');
            expect(js).toContain('if ((item == null))');
            expect(js).toContain('break;');
        });

        test('perge in for loop', () => {
            const js = compile(`
                ex items pro item {
                    si item == nihil { perge }
                    scribe item
                }
            `);

            expect(js).toContain('for (const item of items)');
            expect(js).toContain('continue;');
            expect(js).toContain('console.log(item)');
        });

        test('rumpe and perge together', () => {
            const js = compile(`
                dum verum {
                    si skip { perge }
                    si done { rumpe }
                }
            `);

            expect(js).toContain('continue;');
            expect(js).toContain('break;');
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

    describe('enum declarations (ordo)', () => {
        test('simple enum', () => {
            const js = compile('ordo color { rubrum, viridis, caeruleum }');

            expect(js).toBe('enum color { rubrum, viridis, caeruleum }');
        });

        test('enum with numeric values', () => {
            const js = compile('ordo status { pendens = 0, actum = 1, finitum = 2 }');

            expect(js).toBe('enum status { pendens = 0, actum = 1, finitum = 2 }');
        });

        test('enum with string values', () => {
            const js = compile('ordo direction { north = "N", south = "S" }');

            expect(js).toBe('enum direction { north = "N", south = "S" }');
        });

        test('enum with mixed values', () => {
            const js = compile('ordo mixed { a, b = 5, c }');

            expect(js).toBe('enum mixed { a, b = 5, c }');
        });

        test('enum member access', () => {
            const js = compile(`
                ordo color { rubrum, viridis }
                scribe color.rubrum
            `);

            expect(js).toContain('enum color { rubrum, viridis }');
            expect(js).toContain('console.log(color.rubrum)');
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

    describe('switch statements (elige)', () => {
        test('elige with cases emits if/else chain', () => {
            const js = compile(`
                elige x {
                    si 1 { a() }
                    si 2 { b() }
                }
            `);

            expect(js).toContain('if (x === 1)');
            expect(js).toContain('else if (x === 2)');
            expect(js).toContain('a()');
            expect(js).toContain('b()');
        });

        test('elige with default emits else', () => {
            const js = compile(`
                elige x {
                    si 1 { a() }
                    aliter { c() }
                }
            `);

            expect(js).toContain('if (x === 1)');
            expect(js).toContain('else {');
            expect(js).toContain('c()');
        });

        test('elige with multiple cases and default', () => {
            const js = compile(`
                elige status {
                    si 0 { pending() }
                    si 1 { active() }
                    si 2 { done() }
                    aliter { unknown() }
                }
            `);

            expect(js).toContain('if (status === 0)');
            expect(js).toContain('else if (status === 1)');
            expect(js).toContain('else if (status === 2)');
            expect(js).toContain('else {');
            expect(js).toContain('unknown()');
        });

        test('elige with string cases', () => {
            const js = compile(`
                elige name {
                    si "alice" { greetAlice() }
                    si "bob" { greetBob() }
                }
            `);

            expect(js).toContain('if (name === "alice")');
            expect(js).toContain('else if (name === "bob")');
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
            expect(js).toContain('if (overrides.nomen !== undefined) { this.nomen = overrides.nomen; }');
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

    describe('nexum (reactive fields)', () => {
        test('nexum field emits private backing field', () => {
            const js = compile('genus counter { nexum numerus count: 0 }');

            expect(js).toContain('#count = 0;');
        });

        test('nexum field emits getter', () => {
            const js = compile('genus counter { nexum numerus count: 0 }');

            expect(js).toContain('get count(): number { return this.#count; }');
        });

        test('nexum field emits setter with invalidation', () => {
            const js = compile('genus counter { nexum numerus count: 0 }');

            expect(js).toContain("set count(v: number) { this.#count = v; this.__invalidate?.('count'); }");
        });

        test('nexum works with different types', () => {
            const js = compile('genus user { nexum textus name: "anon" }');

            expect(js).toContain('#name = "anon";');
            expect(js).toContain('get name(): string { return this.#name; }');
            expect(js).toContain("set name(v: string) { this.#name = v; this.__invalidate?.('name'); }");
        });

        test('nexum without initial value', () => {
            const js = compile('genus timer { nexum numerus elapsed }');

            expect(js).toContain('#elapsed;');
            expect(js).toContain('get elapsed(): number');
            expect(js).toContain('set elapsed(v: number)');
        });

        test('mixed nexum and regular fields', () => {
            const js = compile(`
                genus widget {
                    textus id: "x"
                    nexum numerus count: 0
                    bivalens active: verum
                }
            `);

            // Regular fields - public by default (struct semantics)
            expect(js).toContain('id: string = "x";');
            expect(js).toContain('active: boolean = true;');

            // Reactive field - getter/setter pattern
            expect(js).toContain('#count = 0;');
            expect(js).toContain('get count(): number');
            expect(js).toContain('set count(v: number)');
        });

        test('nexum with publicus modifier', () => {
            const js = compile('genus counter { nexum numerus count: 0 }');

            // Even without publicus, reactive fields are accessible via getter/setter
            expect(js).toContain('get count()');
            expect(js).toContain('set count(');
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

        test('iace in elige case', () => {
            const js = compile(`
                elige x {
                    si 1 { iace "error" }
                }
            `);

            expect(js).toContain('if (x === 1)');
            expect(js).toContain('throw "error"');
        });
    });

    describe('mori (panic)', () => {
        test('mori emits Panic class in preamble', () => {
            const js = compile('mori "something went wrong"');

            expect(js).toContain('class Panic extends Error { name = "Panic"; }');
        });

        test('mori string becomes throw new Panic', () => {
            const js = compile('mori "something went wrong"');

            expect(js).toContain('throw new Panic("something went wrong")');
        });

        test('mori with variable wraps in Panic', () => {
            const js = compile('mori err');

            expect(js).toContain('throw new Panic(String(err))');
        });

        test('mori always wraps in Panic (unlike iace)', () => {
            const iaceJs = compile('iace "message"');
            const moriJs = compile('mori "message"');

            // iace just throws the value directly (no preamble)
            expect(iaceJs).toBe('throw "message";');

            // mori wraps in Panic class
            expect(moriJs).toContain('throw new Panic("message")');
        });

        test('mori in conditional', () => {
            const js = compile('si invalid { mori "impossible state" }');

            expect(js).toContain('if (invalid)');
            expect(js).toContain('throw new Panic("impossible state")');
        });

        test('mori with expression', () => {
            const js = compile('mori computeError()');

            expect(js).toContain('throw new Panic(String(computeError()))');
        });

        test('mori preserves quotes in message', () => {
            const js = compile('mori "can\'t do this"');

            expect(js).toContain("can't do this");
        });

        test('preamble only emitted when mori used', () => {
            const withoutMori = compile('scribe "hello"');
            const withMori = compile('mori "error"');

            expect(withoutMori).not.toContain('class Panic');
            expect(withMori).toContain('class Panic');
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

        describe('mutating variants', () => {
            test('filtra -> in-place filter', () => {
                const js = compile('items.filtra(fn)');
                expect(js).toContain('splice');
                expect(js).toContain('fn');
            });

            test('ordina -> sort (mutating)', () => {
                const js = compile('items.ordina()');
                expect(js).toBe('items.sort();');
            });

            test('ordina with comparator', () => {
                const js = compile('items.ordina(fn)');
                expect(js).toBe('items.sort(fn);');
            });

            test('inverte -> reverse (mutating)', () => {
                const js = compile('items.inverte()');
                expect(js).toBe('items.reverse();');
            });
        });

        describe('lodash-inspired', () => {
            test('congrega -> Object.groupBy', () => {
                const js = compile('items.congrega(fn)');
                expect(js).toBe('Object.groupBy(items, fn);');
            });

            test('unica -> Set spread', () => {
                const js = compile('items.unica()');
                expect(js).toBe('[...new Set(items)];');
            });

            test('planaOmnia -> flat(Infinity)', () => {
                const js = compile('items.planaOmnia()');
                expect(js).toBe('items.flat(Infinity);');
            });

            test('fragmenta -> chunk implementation', () => {
                const js = compile('items.fragmenta(3)');
                expect(js).toContain('Array.from');
                expect(js).toContain('Math.ceil');
                expect(js).toContain('slice');
            });

            test('densa -> filter(Boolean)', () => {
                const js = compile('items.densa()');
                expect(js).toBe('items.filter(Boolean);');
            });

            test('partire -> partition via reduce', () => {
                const js = compile('items.partire(fn)');
                expect(js).toContain('reduce');
                expect(js).toContain('fn');
            });

            test('misce -> Fisher-Yates shuffle', () => {
                const js = compile('items.misce()');
                expect(js).toContain('Math.random');
                expect(js).toContain('[...items]');
            });

            test('specimen -> random element', () => {
                const js = compile('items.specimen()');
                expect(js).toContain('Math.floor');
                expect(js).toContain('Math.random');
                expect(js).toContain('items.length');
            });

            test('specimina -> random n elements', () => {
                const js = compile('items.specimina(5)');
                expect(js).toContain('Math.random');
                expect(js).toContain('slice(0, 5)');
            });
        });

        describe('aggregation', () => {
            test('summa -> reduce sum', () => {
                const js = compile('nums.summa()');
                expect(js).toBe('nums.reduce((a, b) => a + b, 0);');
            });

            test('medium -> average', () => {
                const js = compile('nums.medium()');
                expect(js).toContain('reduce');
                expect(js).toContain('nums.length');
            });

            test('minimus -> Math.min', () => {
                const js = compile('nums.minimus()');
                expect(js).toBe('Math.min(...nums);');
            });

            test('maximus -> Math.max', () => {
                const js = compile('nums.maximus()');
                expect(js).toBe('Math.max(...nums);');
            });

            test('minimusPer -> min by key', () => {
                const js = compile('items.minimusPer(fn)');
                expect(js).toContain('reduce');
                expect(js).toContain('fn');
            });

            test('maximusPer -> max by key', () => {
                const js = compile('items.maximusPer(fn)');
                expect(js).toContain('reduce');
                expect(js).toContain('fn');
            });

            test('numera -> count matching', () => {
                const js = compile('items.numera(fn)');
                expect(js).toBe('items.filter(fn).length;');
            });
        });
    });

    // =========================================================================
    // TABULA METHODS - Latin Map API
    // =========================================================================
    // Tests use typed declarations so semantic analyzer populates resolvedType.
    describe('tabula methods - Latin Map API', () => {
        // Helper to compile with a tabula variable in scope
        const tabulaCompile = (expr: string) => compile(`fixum tabula<textus, numerus> m = novum tabula()\n${expr}`);

        describe('core operations', () => {
            test('pone -> set', () => {
                const js = tabulaCompile('m.pone(k, v)');
                expect(js).toContain('m.set(k, v)');
            });

            test('accipe -> get', () => {
                const js = tabulaCompile('m.accipe(k)');
                expect(js).toContain('m.get(k)');
            });

            test('habet -> has', () => {
                const js = tabulaCompile('m.habet(k)');
                expect(js).toContain('m.has(k)');
            });

            test('dele -> delete', () => {
                const js = tabulaCompile('m.dele(k)');
                expect(js).toContain('m.delete(k)');
            });

            test('longitudo -> size', () => {
                const js = tabulaCompile('m.longitudo()');
                expect(js).toContain('m.size');
            });

            test('vacua -> size === 0', () => {
                const js = tabulaCompile('m.vacua()');
                expect(js).toContain('m.size === 0');
            });

            test('purga -> clear', () => {
                const js = tabulaCompile('m.purga()');
                expect(js).toContain('m.clear()');
            });
        });

        describe('iteration', () => {
            test('claves -> keys', () => {
                const js = tabulaCompile('m.claves()');
                expect(js).toContain('m.keys()');
            });

            test('valores -> values', () => {
                const js = tabulaCompile('m.valores()');
                expect(js).toContain('m.values()');
            });

            test('paria -> entries', () => {
                const js = tabulaCompile('m.paria()');
                expect(js).toContain('m.entries()');
            });
        });

        describe('lodash-inspired', () => {
            test('accipeAut -> get with default', () => {
                const js = tabulaCompile('m.accipeAut(k, def)');
                expect(js).toContain('m.get(k) ?? def');
            });

            test('confla -> merge maps', () => {
                const js = tabulaCompile('m.confla(other)');
                expect(js).toContain('new Map([...m, ...other])');
            });

            test('inversa -> swap keys/values', () => {
                const js = tabulaCompile('m.inversa()');
                expect(js).toContain('new Map');
                expect(js).toContain('[v, k]');
            });

            test('mappaValores -> transform values', () => {
                const js = tabulaCompile('m.mappaValores(fn)');
                expect(js).toContain('new Map');
            });

            test('mappaClaves -> transform keys', () => {
                const js = tabulaCompile('m.mappaClaves(fn)');
                expect(js).toContain('new Map');
            });

            test('selige -> pick keys', () => {
                const js = tabulaCompile('m.selige(a, b)');
                expect(js).toContain('new Map');
                expect(js).toContain('filter');
            });

            test('omitte -> omit keys', () => {
                const js = tabulaCompile('m.omitte(a, b)');
                expect(js).toContain('new Map');
                expect(js).toContain('filter');
            });
        });

        describe('conversions', () => {
            test('inLista -> spread to array', () => {
                const js = tabulaCompile('m.inLista()');
                expect(js).toContain('[...m]');
            });

            test('inObjectum -> Object.fromEntries', () => {
                const js = tabulaCompile('m.inObjectum()');
                expect(js).toContain('Object.fromEntries(m)');
            });
        });
    });

    // =========================================================================
    // COPIA METHODS - Latin Set API
    // =========================================================================
    describe('copia methods - Latin Set API', () => {
        // Helper to compile with copia variables in scope
        const copiaCompile = (expr: string) => compile(`fixum copia<numerus> a = novum copia()\nfixum copia<numerus> b = novum copia()\n${expr}`);

        describe('core operations', () => {
            test('adde -> add', () => {
                const js = copiaCompile('a.adde(v)');
                expect(js).toContain('a.add(v)');
            });

            test('habet -> has', () => {
                const js = copiaCompile('a.habet(v)');
                expect(js).toContain('a.has(v)');
            });

            test('dele -> delete', () => {
                const js = copiaCompile('a.dele(v)');
                expect(js).toContain('a.delete(v)');
            });

            test('longitudo -> size', () => {
                const js = copiaCompile('a.longitudo()');
                expect(js).toContain('a.size');
            });

            test('vacua -> size === 0', () => {
                const js = copiaCompile('a.vacua()');
                expect(js).toContain('a.size === 0');
            });

            test('purga -> clear', () => {
                const js = copiaCompile('a.purga()');
                expect(js).toContain('a.clear()');
            });
        });

        describe('set operations', () => {
            test('unio -> union', () => {
                const js = copiaCompile('a.unio(b)');
                expect(js).toContain('new Set([...a, ...b])');
            });

            test('intersectio -> intersection', () => {
                const js = copiaCompile('a.intersectio(b)');
                expect(js).toContain('filter');
                expect(js).toContain('b.has');
            });

            test('differentia -> difference', () => {
                const js = copiaCompile('a.differentia(b)');
                expect(js).toContain('filter');
                expect(js).toContain('!b.has');
            });

            test('symmetrica -> symmetric difference', () => {
                const js = copiaCompile('a.symmetrica(b)');
                expect(js).toContain('new Set');
                expect(js).toContain('filter');
            });
        });

        describe('predicates', () => {
            test('subcopia -> is subset', () => {
                const js = copiaCompile('a.subcopia(b)');
                expect(js).toContain('every');
                expect(js).toContain('b.has');
            });

            test('supercopia -> is superset', () => {
                const js = copiaCompile('a.supercopia(b)');
                expect(js).toContain('every');
                expect(js).toContain('a.has');
            });
        });

        describe('conversions and iteration', () => {
            test('inLista -> spread to array', () => {
                const js = copiaCompile('a.inLista()');
                expect(js).toContain('[...a]');
            });

            test('valores -> values', () => {
                const js = copiaCompile('a.valores()');
                expect(js).toContain('a.values()');
            });

            test('perambula -> forEach', () => {
                const js = copiaCompile('a.perambula(fn)');
                expect(js).toContain('a.forEach(fn)');
            });
        });
    });

    // =========================================================================
    // FAC - Block Scope and Lambda Expressions
    // =========================================================================
    describe('fac - block scope and lambda', () => {
        describe('fac block statement', () => {
            test('fac block -> bare block', () => {
                const js = compile('fac { varia x = 1 }');
                expect(js).toContain('{');
                expect(js).toContain('let x = 1');
            });

            test('fac with cape -> try-catch', () => {
                const js = compile('fac { x() } cape e { y() }');
                expect(js).toContain('try {');
                expect(js).toContain('x()');
                expect(js).toContain('catch (e)');
                expect(js).toContain('y()');
            });
        });

        describe('pro expression (lambda)', () => {
            test('single param lambda', () => {
                const js = compile('pro x redde x * 2');
                expect(js).toBe('(x) => (x * 2);');
            });

            test('multi param lambda', () => {
                const js = compile('pro x, y redde x + y');
                expect(js).toBe('(x, y) => (x + y);');
            });

            test('zero param lambda', () => {
                const js = compile('pro redde 42');
                expect(js).toBe('() => 42;');
            });

            test('lambda used in variable declaration', () => {
                const js = compile('fixum double = pro x redde x * 2');
                expect(js).toBe('const double = (x) => (x * 2);');
            });

            test('lambda as method argument', () => {
                const js = compile('items.filtrata(pro x redde x > 0)');
                expect(js).toBe('items.filter((x) => (x > 0));');
            });
        });
    });

    describe('imports', () => {
        test('norma imports are suppressed (handled via intrinsics)', () => {
            const js = compile('ex "norma/tempus" importa nunc');

            // norma imports don't emit import statements
            expect(js).not.toContain('import');
            expect(js).toBe('');
        });

        test('external package imports pass through', () => {
            const js = compile('ex "@hono/hono" importa Hono, Context');

            expect(js).toBe('import { Hono, Context } from "@hono/hono";');
        });

        test('nunc() compiles to Date.now()', () => {
            const js = compile(`
                ex "norma/tempus" importa nunc
                fixum now = nunc()
            `);

            expect(js).not.toContain('import');
            expect(js).toContain('const now = Date.now()');
        });

        test('SECUNDUM compiles to literal 1000', () => {
            const js = compile(`
                ex "norma/tempus" importa SECUNDUM
                fixum sec = SECUNDUM
            `);

            expect(js).not.toContain('import');
            expect(js).toContain('const sec = 1000');
        });

        test('dormi compiles to Promise with setTimeout', () => {
            const js = compile(`
                ex "norma/tempus" importa dormi, SECUNDUM
                futura functio wait() {
                    cede dormi(5 * SECUNDUM)
                }
            `);

            expect(js).not.toContain('import');
            expect(js).toContain('await new Promise(r => setTimeout(r, (5 * 1000)))');
        });

        test('all duration constants compile to literals', () => {
            const js = compile(`
                ex "norma/tempus" importa MILLISECUNDUM, SECUNDUM, MINUTUM, HORA, DIES
                fixum a = MILLISECUNDUM
                fixum b = SECUNDUM
                fixum c = MINUTUM
                fixum d = HORA
                fixum e = DIES
            `);

            expect(js).toContain('const a = 1');
            expect(js).toContain('const b = 1000');
            expect(js).toContain('const c = 60000');
            expect(js).toContain('const d = 3600000');
            expect(js).toContain('const e = 86400000');
        });
    });
});
