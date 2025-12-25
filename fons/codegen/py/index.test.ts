/**
 * Python Code Generator Tests
 *
 * Tests Python 3.10+ code generation from Latin AST.
 * Covers type system, declarations, control flow, expressions, and OOP.
 */

import { describe, expect, test } from 'bun:test';
import { tokenize } from '../../tokenizer';
import { parse } from '../../parser/index';
import { generate } from '../index';

/**
 * Helper to compile Latin source to Python.
 */
function compile(code: string): string {
    const { tokens } = tokenize(code);
    const { program } = parse(tokens);
    if (!program) {
        throw new Error('Parse failed');
    }
    return generate(program, { target: 'py' });
}

// =============================================================================
// TYPE SYSTEM
// =============================================================================

describe('Python codegen', () => {
    describe('type system', () => {
        test('textus maps to str', () => {
            // Latin syntax: type before name
            const result = compile('varia textus x = "hello"');
            expect(result).toBe('x: str = "hello"');
        });

        test('numerus maps to int', () => {
            const result = compile('varia numerus x = 42');
            expect(result).toBe('x: int = 42');
        });

        test('bivalens maps to bool', () => {
            const result = compile('varia bivalens x = verum');
            expect(result).toBe('x: bool = True');
        });

        test('lista maps to list', () => {
            const result = compile('varia lista<numerus> x = []');
            expect(result).toBe('x: list[int] = []');
        });

        test('tabula maps to dict', () => {
            const result = compile('varia tabula<textus, numerus> x = {}');
            expect(result).toBe('x: dict[str, int] = {}');
        });

        test('nullable types use union with None', () => {
            const result = compile('varia textus? x = nihil');
            expect(result).toBe('x: str | None = None');
        });

        test('fractus maps to float', () => {
            const result = compile('varia fractus x = 3.14');
            expect(result).toBe('x: float = 3.14');
        });

        test('decimus maps to Decimal', () => {
            const result = compile('varia decimus x = 99.99');
            expect(result).toContain('x: Decimal = 99.99');
        });

        test('octeti maps to bytes', () => {
            const result = compile('varia octeti data = []');
            expect(result).toBe('data: bytes = []');
        });

        test('objectum maps to object', () => {
            const result = compile('varia objectum x = {}');
            expect(result).toBe('x: object = {}');
        });

        test('copia maps to set', () => {
            const result = compile('varia copia<numerus> x = []');
            expect(result).toBe('x: set[int] = []');
        });

        test('type alias', () => {
            const result = compile('typus ID = textus');
            expect(result).toBe('ID = str');
        });
    });

    // =========================================================================
    // ENUM DECLARATIONS
    // =========================================================================

    describe('enum declarations', () => {
        test('basic enum with auto values', () => {
            const result = compile('ordo Color { rubrum, viridis, caeruleum }');
            expect(result).toContain('class Color(Enum):');
            expect(result).toContain('rubrum = auto()');
            expect(result).toContain('viridis = auto()');
            expect(result).toContain('caeruleum = auto()');
        });

        test('enum with numeric values', () => {
            const result = compile('ordo Status { pendens = 0, actum = 1, finitum = 2 }');
            expect(result).toContain('class Status(Enum):');
            expect(result).toContain('pendens = 0');
            expect(result).toContain('actum = 1');
            expect(result).toContain('finitum = 2');
        });

        test('enum with string values', () => {
            const result = compile('ordo Direction { north = "N", south = "S" }');
            expect(result).toContain('class Direction(Enum):');
            expect(result).toContain('north = "N"');
            expect(result).toContain('south = "S"');
        });

        test('enum emits preamble import', () => {
            const result = compile('ordo Color { rubrum }');
            expect(result).toContain('from enum import Enum, auto');
        });
    });

    // =========================================================================
    // PREAMBLE
    // =========================================================================

    describe('preamble', () => {
        test('decimal type emits Decimal import', () => {
            const result = compile('varia decimus x = 99.99');
            expect(result).toContain('from decimal import Decimal');
        });

        test('no preamble when not needed', () => {
            const result = compile('varia numerus x = 5');
            expect(result).toBe('x: int = 5');
        });
    });

    // =========================================================================
    // VARIABLE DECLARATIONS
    // =========================================================================

    describe('variable declarations', () => {
        test('varia becomes assignment', () => {
            const result = compile('varia x = 5');
            expect(result).toBe('x = 5');
        });

        test('fixum becomes assignment (no const in Python)', () => {
            const result = compile('fixum x = 10');
            expect(result).toBe('x = 10');
        });

        test('with type annotation', () => {
            const result = compile('varia textus nomen = "Marcus"');
            expect(result).toBe('nomen: str = "Marcus"');
        });

        test('figendum adds await', () => {
            const result = compile('figendum data = fetchData()');
            expect(result).toBe('data = await fetchData()');
        });

        test('variandum adds await', () => {
            const result = compile('variandum result = getResult()');
            expect(result).toBe('result = await getResult()');
        });

        test('figendum with type annotation', () => {
            const result = compile('figendum textus data = fetch()');
            expect(result).toBe('data: str = await fetch()');
        });

        test('variandum with type annotation', () => {
            const result = compile('variandum numerus count = getCount()');
            expect(result).toBe('count: int = await getCount()');
        });
    });

    // =========================================================================
    // FUNCTION DECLARATIONS
    // =========================================================================

    describe('function declarations', () => {
        test('basic function', () => {
            const result = compile('functio salve() { }');
            expect(result).toContain('def salve():');
            expect(result).toContain('pass');
        });

        test('function with parameters', () => {
            const result = compile('functio salve(textus nomen) { scribe nomen }');
            expect(result).toContain('def salve(nomen: str):');
            expect(result).toContain('print(nomen)');
        });

        test('function with return type', () => {
            const result = compile('functio duplex(numerus x) fit numerus { redde x * 2 }');
            expect(result).toContain('def duplex(x: int) -> int:');
            expect(result).toContain('return (x * 2)');
        });

        test('async function', () => {
            // fiet = async single return (not fit which is sync)
            const result = compile('functio fetch() fiet textus { redde "data" }');
            expect(result).toContain('async def fetch() -> Awaitable[str]:');
        });

        test('generator function', () => {
            // fiunt = sync generator (not fit which is single return)
            const result = compile('functio gen() fiunt numerus { cede 1 }');
            expect(result).toContain('def gen() -> Iterator[int]:');
            expect(result).toContain('yield 1');
        });
    });

    // =========================================================================
    // IF STATEMENTS
    // =========================================================================

    describe('if statements', () => {
        test('simple if', () => {
            const result = compile('si verum { scribe "yes" }');
            expect(result).toContain('if True:');
            expect(result).toContain('print("yes")');
        });

        test('if-else', () => {
            const result = compile('si falsum { scribe "a" } aliter { scribe "b" }');
            expect(result).toContain('if False:');
            expect(result).toContain('else:');
        });

        test('if-elif-else chain', () => {
            const result = compile('si x == 1 { scribe "one" } aliter si x == 2 { scribe "two" } aliter { scribe "other" }');
            expect(result).toContain('if (x == 1):');
            expect(result).toContain('elif (x == 2):');
            expect(result).toContain('else:');
        });
    });

    // =========================================================================
    // WHILE STATEMENTS
    // =========================================================================

    describe('while statements', () => {
        test('simple while', () => {
            const result = compile('dum verum { scribe "loop" }');
            expect(result).toContain('while True:');
            expect(result).toContain('print("loop")');
        });

        test('break statement', () => {
            const result = compile('dum verum { rumpe }');
            expect(result).toContain('while True:');
            expect(result).toContain('break');
        });

        test('continue statement', () => {
            const result = compile('dum verum { perge }');
            expect(result).toContain('while True:');
            expect(result).toContain('continue');
        });
    });

    // =========================================================================
    // FOR STATEMENTS
    // =========================================================================

    describe('for statements', () => {
        test('for with range', () => {
            const result = compile('ex 0..10 pro i { scribe i }');
            expect(result).toContain('for i in range(0, 10):');
            expect(result).toContain('print(i)');
        });

        test('for with range and step', () => {
            const result = compile('ex 0..10 per 2 pro i { scribe i }');
            expect(result).toContain('for i in range(0, 10, 2):');
        });

        test('for over array', () => {
            const result = compile('ex items pro item { scribe item }');
            expect(result).toContain('for item in items:');
        });

        test('async for', () => {
            const result = compile('ex stream fiet chunk { scribe chunk }');
            expect(result).toContain('async for chunk in stream:');
        });
    });

    // =========================================================================
    // SWITCH/MATCH STATEMENTS
    // =========================================================================

    describe('switch/match statements', () => {
        test('basic match', () => {
            const result = compile('elige x { si 1 { scribe "one" } si 2 { scribe "two" } }');
            expect(result).toContain('match x:');
            expect(result).toContain('case 1:');
            expect(result).toContain('case 2:');
        });

        test('match with default', () => {
            const result = compile('elige x { si 1 { scribe "one" } aliter { scribe "other" } }');
            expect(result).toContain('match x:');
            expect(result).toContain('case _:');
        });
    });

    // =========================================================================
    // GUARD STATEMENTS
    // =========================================================================

    describe('guard statements', () => {
        test('simple guard', () => {
            const result = compile('custodi { si x == nihil { redde } }');
            expect(result).toContain('if (x == None):');
            expect(result).toContain('return');
        });

        test('multiple guards', () => {
            const result = compile('custodi { si a < 0 { redde -1 } si a > 100 { redde 100 } }');
            expect(result).toContain('if (a < 0):');
            expect(result).toContain('if (a > 100):');
        });
    });

    // =========================================================================
    // ASSERT STATEMENTS
    // =========================================================================

    describe('assert statements', () => {
        test('simple assert', () => {
            const result = compile('adfirma x > 0');
            expect(result).toBe('assert (x > 0)');
        });

        test('assert with message', () => {
            const result = compile('adfirma x > 0, "x must be positive"');
            expect(result).toBe('assert (x > 0), "x must be positive"');
        });
    });

    // =========================================================================
    // EXCEPTION HANDLING
    // =========================================================================

    describe('exception handling', () => {
        test('try-except', () => {
            const result = compile('tempta { scribe "try" } cape e { scribe "error" }');
            expect(result).toContain('try:');
            expect(result).toContain('except Exception as e:');
        });

        test('try-except-finally', () => {
            const result = compile('tempta { scribe "try" } cape e { scribe "error" } demum { scribe "finally" }');
            expect(result).toContain('try:');
            expect(result).toContain('except Exception as e:');
            expect(result).toContain('finally:');
        });

        test('throw/raise', () => {
            const result = compile('iace "error message"');
            expect(result).toBe('raise Exception("error message")');
        });
    });

    // =========================================================================
    // EXPRESSIONS
    // =========================================================================

    describe('expressions', () => {
        test('boolean literals', () => {
            expect(compile('scribe verum')).toContain('print(True)');
            expect(compile('scribe falsum')).toContain('print(False)');
        });

        test('null/None literal', () => {
            expect(compile('scribe nihil')).toContain('print(None)');
        });

        test('string literals', () => {
            expect(compile('scribe "hello"')).toContain('print("hello")');
        });

        test('number literals', () => {
            expect(compile('scribe 42')).toContain('print(42)');
            expect(compile('scribe 3.14')).toContain('print(3.14)');
        });

        test('array literals', () => {
            expect(compile('varia x = [1, 2, 3]')).toBe('x = [1, 2, 3]');
        });

        test('object/dict literals', () => {
            expect(compile('varia x = { nomen: "Marcus" }')).toBe('x = {"nomen": "Marcus"}');
        });

        test('binary operators', () => {
            expect(compile('scribe 1 + 2')).toContain('print((1 + 2))');
            expect(compile('scribe 3 - 1')).toContain('print((3 - 1))');
            expect(compile('scribe 2 * 3')).toContain('print((2 * 3))');
            expect(compile('scribe 6 / 2')).toContain('print((6 / 2))');
        });

        test('logical operators map to and/or', () => {
            expect(compile('scribe verum et falsum')).toContain('(True and False)');
            expect(compile('scribe verum aut falsum')).toContain('(True or False)');
        });

        test('comparison operators', () => {
            expect(compile('scribe x == y')).toContain('(x == y)');
            expect(compile('scribe x != y')).toContain('(x != y)');
            expect(compile('scribe x < y')).toContain('(x < y)');
            expect(compile('scribe x > y')).toContain('(x > y)');
        });

        test('unary not operator', () => {
            // WHY: Use 'non' for logical negation (prefix ! removed for !. non-null assertion)
            expect(compile('scribe non verum')).toContain('print(not True)');
        });

        test('ternary with ? :', () => {
            const result = compile('verum ? 1 : 0');
            expect(result).toBe('1 if True else 0');
        });

        test('ternary with sic secus (Latin)', () => {
            const result = compile('verum sic 1 secus 0');
            expect(result).toBe('1 if True else 0');
        });

        test('ternary in variable', () => {
            const result = compile('varia x = verum ? 1 : 0');
            expect(result).toBe('x = 1 if True else 0');
        });

        test('ternary with condition', () => {
            const result = compile('x > 5 ? "big" : "small"');
            expect(result).toBe('"big" if (x > 5) else "small"');
        });

        test('nested ternary', () => {
            const result = compile('a ? b ? c : d : e');
            expect(result).toBe('c if b else d if a else e');
        });

        test('member access', () => {
            expect(compile('scribe obj.prop')).toContain('print(obj.prop)');
        });

        test('computed member access', () => {
            expect(compile('scribe arr[0]')).toContain('print(arr[0])');
        });

        test('function call', () => {
            expect(compile('salve("Marcus")')).toBe('salve("Marcus")');
        });
    });

    // =========================================================================
    // SPECIAL OPERATORS
    // =========================================================================

    describe('special operators', () => {
        test('negativum checks less than zero', () => {
            const result = compile('scribe negativum x');
            expect(result).toContain('(x < 0)');
        });

        test('positivum checks greater than zero', () => {
            const result = compile('scribe positivum x');
            expect(result).toContain('(x > 0)');
        });
    });

    // =========================================================================
    // ARROW FUNCTIONS
    // =========================================================================

    describe('arrow functions', () => {
        test('simple lambda', () => {
            const result = compile('varia f = (x) => x * 2');
            expect(result).toBe('f = lambda x: (x * 2)');
        });

        test('lambda with multiple params', () => {
            const result = compile('varia add = (a, b) => a + b');
            expect(result).toBe('add = lambda a, b: (a + b)');
        });
    });

    // =========================================================================
    // NEW EXPRESSIONS
    // =========================================================================

    describe('new expressions', () => {
        test('new without arguments', () => {
            const result = compile('varia p = novum Persona()');
            expect(result).toBe('p = Persona()');
        });

        test('new with arguments', () => {
            const result = compile('varia p = novum Persona("Marcus", 30)');
            expect(result).toBe('p = Persona("Marcus", 30)');
        });
    });

    // =========================================================================
    // THIS/SELF
    // =========================================================================

    describe('this/self', () => {
        test('ego maps to self', () => {
            const result = compile('scribe ego.nomen');
            expect(result).toContain('print(self.nomen)');
        });
    });

    // =========================================================================
    // IMPORTS
    // =========================================================================

    describe('imports', () => {
        test('wildcard import', () => {
            const result = compile('ex norma importa *');
            expect(result).toBe('import norma');
        });

        test('single named import', () => {
            const result = compile('ex norma importa scribe');
            expect(result).toBe('from norma import scribe');
        });

        // Note: May have parser issue with comma-separated imports
        test.skip('multiple named imports', () => {
            const result = compile('ex norma importa scribe, lege');
            expect(result).toBe('from norma import scribe, lege');
        });
    });

    // =========================================================================
    // SCRIBE (PRINT)
    // =========================================================================

    describe('scribe statements', () => {
        test('single argument', () => {
            expect(compile('scribe "hello"')).toBe('print("hello")');
        });

        test('multiple arguments', () => {
            expect(compile('scribe "a", "b", "c"')).toBe('print("a", "b", "c")');
        });

        // Note: Parser may not handle scribe with no args
        test.skip('no arguments', () => {
            expect(compile('scribe')).toBe('print()');
        });
    });

    // =========================================================================
    // LISTA METHODS
    // =========================================================================

    describe('lista methods', () => {
        describe('adding elements', () => {
            test('adde (append)', () => {
                const result = compile('items.adde(5)');
                expect(result).toBe('items.append(5)');
            });

            test('addita (spread copy)', () => {
                const result = compile('varia x = items.addita(5)');
                expect(result).toBe('x = [*items, 5]');
            });
        });

        describe('accessing elements', () => {
            test('primus (first)', () => {
                const result = compile('varia x = items.primus()');
                expect(result).toBe('x = items[0]');
            });

            test('ultimus (last)', () => {
                const result = compile('varia x = items.ultimus()');
                expect(result).toBe('x = items[-1]');
            });

            test('longitudo (length)', () => {
                const result = compile('varia x = items.longitudo()');
                expect(result).toBe('x = len(items)');
            });
        });

        describe('searching', () => {
            test('continet (includes)', () => {
                const result = compile('varia x = items.continet(5)');
                expect(result).toBe('x = (5 in items)');
            });
        });

        describe('transformations', () => {
            test('filtrata (filter)', () => {
                const result = compile('varia x = items.filtrata((n) => n > 0)');
                expect(result).toBe('x = list(filter(lambda n: (n > 0), items))');
            });

            test('mappata (map)', () => {
                const result = compile('varia x = items.mappata((n) => n * 2)');
                expect(result).toBe('x = list(map(lambda n: (n * 2), items))');
            });

            test('inversa (reverse)', () => {
                const result = compile('varia x = items.inversa()');
                expect(result).toBe('x = items[::-1]');
            });

            test('ordinata (sorted)', () => {
                const result = compile('varia x = items.ordinata()');
                expect(result).toBe('x = sorted(items)');
            });
        });

        describe('slicing', () => {
            test('prima (first n)', () => {
                const result = compile('varia x = items.prima(3)');
                expect(result).toBe('x = items[:3]');
            });

            test('ultima (last n)', () => {
                const result = compile('varia x = items.ultima(3)');
                expect(result).toBe('x = items[-3:]');
            });

            test('omitte (skip n)', () => {
                const result = compile('varia x = items.omitte(2)');
                expect(result).toBe('x = items[2:]');
            });
        });

        describe('predicates', () => {
            test('omnes (all)', () => {
                const result = compile('varia x = items.omnes((n) => n > 0)');
                expect(result).toBe('x = all(map(lambda n: (n > 0), items))');
            });

            test('aliquis (any)', () => {
                const result = compile('varia x = items.aliquis((n) => n > 0)');
                expect(result).toBe('x = any(map(lambda n: (n > 0), items))');
            });
        });

        describe('string operations', () => {
            test('coniunge (join)', () => {
                const result = compile('varia x = items.coniunge(", ")');
                expect(result).toBe('x = ", ".join(items)');
            });
        });
    });

    // =========================================================================
    // GENUS (CLASS) DECLARATIONS
    // =========================================================================

    describe('genus declarations', () => {
        test('empty class', () => {
            const result = compile('genus Persona { }');
            expect(result).toContain('class Persona:');
            expect(result).toContain('pass');
        });

        test('class with fields', () => {
            const result = compile(`
                genus Persona {
                    textus nomen
                    numerus aetas
                }
            `);
            expect(result).toContain('class Persona:');
            expect(result).toContain('nomen: str');
            expect(result).toContain('aetas: int');
        });

        test('class with field defaults', () => {
            const result = compile(`
                genus Config {
                    numerus timeout: 30
                    bivalens debug: falsum
                }
            `);
            expect(result).toContain('timeout: int = 30');
            expect(result).toContain('debug: bool = False');
        });

        test('class with method', () => {
            const result = compile(`
                genus Persona {
                    textus nomen
                    functio saluta() fit textus {
                        redde "Salve, " + ego.nomen
                    }
                }
            `);
            expect(result).toContain('def saluta(self) -> str:');
            expect(result).toContain('return ("Salve, " + self.nomen)');
        });

        test('class with async method', () => {
            const result = compile(`
                genus Service {
                    functio fetch() fiet textus {
                        redde "data"
                    }
                }
            `);
            expect(result).toContain('async def fetch(self) -> Awaitable[str]:');
        });

        test('class with generator method', () => {
            const result = compile(`
                genus Range {
                    functio iter() fiunt numerus {
                        cede 1
                    }
                }
            `);
            expect(result).toContain('def iter(self) -> Iterator[int]:');
            expect(result).toContain('yield 1');
        });

        test('class with creo constructor', () => {
            const result = compile(`
                genus Persona {
                    textus nomen
                    functio creo() {
                        scribe "Created"
                    }
                }
            `);
            expect(result).toContain('def __init__(self');
            expect(result).toContain('def _creo(self):');
            expect(result).toContain('print("Created")');
        });

        test('class implements interface', () => {
            const result = compile(`
                genus Worker implet Runnable {
                    functio run() { }
                }
            `);
            expect(result).toContain('class Worker(Runnable):');
        });

        test('class with type parameters', () => {
            const result = compile(`
                genus Container<T> {
                    T value
                }
            `);
            expect(result).toContain('class Container[T]:');
        });

        test('class with nexum reactive field', () => {
            const result = compile(`
                genus counter {
                    nexum numerus count: 0
                }
            `);
            expect(result).toContain('_count: int = 0');
            expect(result).toContain('@property');
            expect(result).toContain('def count(self) -> int:');
            expect(result).toContain('@count.setter');
            expect(result).toContain('self._pingo()');
        });

        test('class with mixed regular and nexum fields', () => {
            const result = compile(`
                genus widget {
                    textus name
                    nexum numerus value: 0
                }
            `);
            expect(result).toContain('name: str');
            expect(result).toContain('_value: int = 0');
            expect(result).toContain('@property');
            expect(result).toContain('def value(self) -> int:');
        });
    });

    // =========================================================================
    // PACTUM (INTERFACE/PROTOCOL) DECLARATIONS
    // =========================================================================

    describe('pactum declarations', () => {
        test('empty protocol', () => {
            const result = compile('pactum Runnable { }');
            expect(result).toContain('class Runnable(Protocol):');
            expect(result).toContain('pass');
        });

        test('protocol with method signature', () => {
            const result = compile(`
                pactum Runnable {
                    functio run() fit vacuum
                }
            `);
            expect(result).toContain('class Runnable(Protocol):');
            expect(result).toContain('def run(self) -> None: ...');
        });

        test('protocol with async method', () => {
            const result = compile(`
                pactum Fetcher {
                    functio fetch() fiet textus
                }
            `);
            expect(result).toContain('async def fetch(self) -> Awaitable[str]: ...');
        });

        test('protocol with generator method', () => {
            const result = compile(`
                pactum Iterable {
                    functio iter() fiunt numerus
                }
            `);
            expect(result).toContain('def iter(self) -> Iterator[int]: ...');
        });

        test('protocol with type parameters', () => {
            const result = compile(`
                pactum Container<T> {
                    functio get() fit T
                }
            `);
            expect(result).toContain('class Container[T](Protocol):');
        });
    });

    // =========================================================================
    // IN (WITH) STATEMENTS
    // =========================================================================

    describe('in (with) statements', () => {
        test('simple property assignment', () => {
            const result = compile(`
                in user {
                    nomen = "Marcus"
                }
            `);
            expect(result).toContain('user.nomen = "Marcus"');
        });

        test('multiple property assignments', () => {
            const result = compile(`
                in config {
                    host = "localhost"
                    port = 8080
                }
            `);
            expect(result).toContain('config.host = "localhost"');
            expect(result).toContain('config.port = 8080');
        });
    });

    // =========================================================================
    // THROW VARIATIONS
    // =========================================================================

    describe('throw variations', () => {
        test('throw string literal', () => {
            const result = compile('iace "error"');
            expect(result).toBe('raise Exception("error")');
        });

        test('throw new Error', () => {
            const result = compile('iace novum Error("something went wrong")');
            expect(result).toBe('raise Exception("something went wrong")');
        });

        test('throw new erratum', () => {
            const result = compile('iace novum erratum("failed")');
            expect(result).toBe('raise Exception("failed")');
        });

        test('mori (fatal/panic)', () => {
            const result = compile('mori "fatal error"');
            expect(result).toBe('raise SystemExit("fatal error")');
        });
    });

    // =========================================================================
    // VIDE/MONE (DEBUG/WARN) STATEMENTS
    // =========================================================================

    describe('vide/mone statements', () => {
        test('vide adds debug prefix', () => {
            const result = compile('vide "checking value"');
            expect(result).toBe('print("[DEBUG]", "checking value")');
        });

        test('mone adds warn prefix', () => {
            const result = compile('mone "deprecated"');
            expect(result).toBe('print("[WARN]", "deprecated")');
        });

        test('vide with multiple args', () => {
            const result = compile('vide "x =", x');
            expect(result).toBe('print("[DEBUG]", "x =", x)');
        });
    });

    // =========================================================================
    // FAC BLOCK STATEMENTS
    // =========================================================================

    describe('fac block statements', () => {
        test('simple fac block', () => {
            const result = compile('fac { scribe "hello" }');
            // fac block just outputs the body directly in Python
            expect(result).toContain('print("hello")');
        });
    });

    // =========================================================================
    // RANGE EXPRESSIONS
    // =========================================================================

    describe('range expressions', () => {
        test('simple range', () => {
            const result = compile('varia x = 0..10');
            expect(result).toBe('x = list(range(0, 10))');
        });

        test('range with step', () => {
            const result = compile('varia x = 0..10 per 2');
            expect(result).toBe('x = list(range(0, 10, 2))');
        });
    });

    // =========================================================================
    // STRICT EQUALITY
    // =========================================================================

    describe('strict equality', () => {
        test('est maps to ==', () => {
            const result = compile(`
                varia x = 1
                varia y = 2
                scribe x est y
            `);
            expect(result).toContain('(x == y)');
        });
    });

    // =========================================================================
    // NULLA/NONNULLA OPERATORS
    // =========================================================================

    describe('nulla/nonnulla operators', () => {
        test('nulla checks empty', () => {
            const result = compile('scribe nulla x');
            expect(result).toContain("not x or len(x) == 0 if hasattr(x, '__len__') else not x");
        });

        test('nonnulla checks has content', () => {
            const result = compile('scribe nonnulla x');
            expect(result).toContain("x and (len(x) > 0 if hasattr(x, '__len__') else bool(x))");
        });
    });

    // =========================================================================
    // AWAIT/YIELD EXPRESSIONS
    // =========================================================================

    describe('await/yield expressions', () => {
        test('cede as await in async context', () => {
            const result = compile(`
                functio fetch() fiet textus {
                    varia data = cede getData()
                    redde data
                }
            `);
            expect(result).toContain('data = await getData()');
        });

        test('cede as yield in generator context', () => {
            const result = compile(`
                functio gen() fiunt numerus {
                    cede 1
                    cede 2
                }
            `);
            expect(result).toContain('yield 1');
            expect(result).toContain('yield 2');
        });
    });

    // =========================================================================
    // SWITCH WITH CATCH
    // =========================================================================

    describe('switch with catch', () => {
        test('switch with catch clause', () => {
            const result = compile(`
                elige x {
                    si 1 { scribe "one" }
                } cape e {
                    scribe "error"
                }
            `);
            expect(result).toContain('try:');
            expect(result).toContain('match x:');
            expect(result).toContain('except Exception as e:');
        });
    });

    // =========================================================================
    // COMPLETE PROGRAMS
    // =========================================================================

    describe('complete programs', () => {
        test('multi-statement program', () => {
            const result = compile(`
                varia numerus x = 10
                varia numerus y = 20
                scribe x + y
            `);
            expect(result).toContain('x: int = 10');
            expect(result).toContain('y: int = 20');
            expect(result).toContain('print((x + y))');
        });

        test('function with control flow', () => {
            const result = compile(`
                functio max(numerus a, numerus b) fit numerus {
                    si a > b {
                        redde a
                    } aliter {
                        redde b
                    }
                }
            `);
            expect(result).toContain('def max(a: int, b: int) -> int:');
            expect(result).toContain('if (a > b):');
            expect(result).toContain('return a');
            expect(result).toContain('else:');
            expect(result).toContain('return b');
        });
    });

    describe('optional chaining and non-null assertion', () => {
        test('optional member access ?. expands to conditional', () => {
            const result = compile('user?.name');

            expect(result).toContain('(user.name if user is not None else None)');
        });

        test('optional computed access ?[ expands to conditional', () => {
            const result = compile('items?[0]');

            expect(result).toContain('(items[0] if items is not None else None)');
        });

        test('optional call ?( expands to conditional', () => {
            const result = compile('callback?(x)');

            expect(result).toContain('(callback(x) if callback is not None else None)');
        });

        test('non-null member access !. passes through', () => {
            const result = compile('user!.name');

            expect(result).toContain('user.name');
        });

        test('non-null computed access ![ passes through', () => {
            const result = compile('items![0]');

            expect(result).toContain('items[0]');
        });

        test('non-null call !( passes through', () => {
            const result = compile('callback!(x)');

            expect(result).toContain('callback(x)');
        });
    });
});
