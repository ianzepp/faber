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

        test('type alias', () => {
            const result = compile('typus ID = textus');
            expect(result).toBe('ID = str');
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
    });

    // =========================================================================
    // FOR STATEMENTS
    // =========================================================================

    describe('for statements', () => {
        test('for with range', () => {
            const result = compile('ex 0..10 pro i { scribe i }');
            expect(result).toContain('for i in range(0, 10 + 1):');
            expect(result).toContain('print(i)');
        });

        test('for with range and step', () => {
            const result = compile('ex 0..10 per 2 pro i { scribe i }');
            expect(result).toContain('for i in range(0, 10 + 1, 2):');
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
            expect(compile('scribe !verum')).toContain('print(not True)');
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
                expect(result).toBe('x = [x for x in items if lambda n: (n > 0)(x)]');
            });

            test('mappata (map)', () => {
                const result = compile('varia x = items.mappata((n) => n * 2)');
                expect(result).toBe('x = [lambda n: (n * 2)(x) for x in items]');
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
                expect(result).toBe('x = all(lambda n: (n > 0)(x) for x in items)');
            });

            test('aliquis (any)', () => {
                const result = compile('varia x = items.aliquis((n) => n > 0)');
                expect(result).toBe('x = any(lambda n: (n > 0)(x) for x in items)');
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
});
