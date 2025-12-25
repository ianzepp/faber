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

    return generate(program, { target: 'cpp' });
}

describe('cpp codegen', () => {
    describe('variable declarations', () => {
        test('varia -> auto', () => {
            const cpp = compile('varia x = 5');

            expect(cpp).toContain('auto x = 5');
        });

        test('fixum -> const auto', () => {
            const cpp = compile('fixum PI = 3.14');

            expect(cpp).toContain('const auto PI = 3.14');
        });

        test('typed variable', () => {
            const cpp = compile('varia numerus x = 5');

            expect(cpp).toContain('int64_t x = 5');
        });

        test('typed const', () => {
            const cpp = compile('fixum textus name = "Marcus"');

            expect(cpp).toContain('const std::string name');
        });
    });

    describe('function declarations', () => {
        test('simple function', () => {
            const cpp = compile(`
                functio salve(textus nomen) -> nihil {
                    redde
                }
            `);

            expect(cpp).toContain('void salve(const std::string& nomen)');
            expect(cpp).toContain('return;');
        });

        test('function with return type', () => {
            const cpp = compile(`
                functio add(numerus a, numerus b) -> numerus {
                    redde a + b
                }
            `);

            expect(cpp).toContain('int64_t add(int64_t a, int64_t b)');
            expect(cpp).toContain('return (a + b)');
        });
    });

    describe('control flow', () => {
        test('if statement', () => {
            const cpp = compile(`
                si verum {
                    x = 1
                }
            `);

            expect(cpp).toContain('if (true)');
        });

        test('if-else statement', () => {
            const cpp = compile(`
                si falsum {
                    a()
                }
                aliter {
                    b()
                }
            `);

            expect(cpp).toContain('if (false)');
            expect(cpp).toContain('} else {');
        });

        test('while loop', () => {
            const cpp = compile(`
                dum verum {
                    x = 1
                }
            `);

            expect(cpp).toContain('while (true)');
        });

        test('for loop with range', () => {
            const cpp = compile(`
                ex 0..10 pro i {
                    scribe i
                }
            `);

            expect(cpp).toContain('for (int64_t i = 0; i < 10; i += 1)');
        });

        test('for loop over collection', () => {
            const cpp = compile(`
                ex items pro item {
                    scribe item
                }
            `);

            expect(cpp).toContain('for (auto& item : items)');
        });
    });

    describe('expressions', () => {
        test('binary operators', () => {
            const cpp = compile('1 + 2');

            expect(cpp).toContain('(1 + 2)');
        });

        test('strict equality maps to ==', () => {
            const cpp = compile('a === b');

            expect(cpp).toContain('(a == b)');
        });

        test('strict inequality maps to !=', () => {
            const cpp = compile('a !== b');

            expect(cpp).toContain('(a != b)');
        });

        test('logical and', () => {
            const cpp = compile('a et b');

            expect(cpp).toContain('(a && b)');
        });

        test('logical or', () => {
            const cpp = compile('a aut b');

            expect(cpp).toContain('(a || b)');
        });

        test('ternary', () => {
            const cpp = compile('x ? 1 : 0');

            expect(cpp).toContain('(x ? 1 : 0)');
        });
    });

    describe('special expressions', () => {
        test('verum -> true', () => {
            const cpp = compile('verum');

            expect(cpp).toContain('true');
        });

        test('falsum -> false', () => {
            const cpp = compile('falsum');

            expect(cpp).toContain('false');
        });

        test('nihil -> nullptr', () => {
            const cpp = compile('nihil');

            expect(cpp).toContain('nullptr');
        });
    });

    describe('scribe statement', () => {
        test('scribe with string', () => {
            const cpp = compile('scribe "hello"');

            expect(cpp).toContain('std::print');
            expect(cpp).toContain('hello');
        });

        test('scribe with multiple args', () => {
            const cpp = compile('scribe "Name:", name');

            expect(cpp).toContain('std::print("{} {}');
        });
    });

    describe('genus declarations', () => {
        test('simple struct', () => {
            const cpp = compile(`
                genus Persona {
                    textus nomen: "anon"
                }
            `);

            expect(cpp).toContain('struct Persona {');
            expect(cpp).toContain('std::string nomen = ');
            expect(cpp).toContain('};');
        });

        test('struct with method', () => {
            const cpp = compile(`
                genus Persona {
                    textus nomen: "anon"
                    functio saluta() -> textus {
                        redde ego.nomen
                    }
                }
            `);

            expect(cpp).toContain('std::string saluta()');
            expect(cpp).toContain('this->nomen');
        });
    });

    describe('pactum declarations', () => {
        test('generates C++20 concept', () => {
            const cpp = compile(`
                pactum Printable {
                    functio print() -> nihil
                }
            `);

            expect(cpp).toContain('template<typename T>');
            expect(cpp).toContain('concept Printable = requires');
        });
    });

    describe('error handling', () => {
        test('throw statement', () => {
            const cpp = compile('iace "error"');

            expect(cpp).toContain('throw std::runtime_error');
        });

        test('try-catch', () => {
            const cpp = compile(`
                tempta {
                    risky()
                } cape err {
                    handle(err)
                }
            `);

            expect(cpp).toContain('try {');
            expect(cpp).toContain('catch (const std::exception& err)');
        });
    });

    describe('lambda expressions', () => {
        test('arrow function', () => {
            const cpp = compile('fixum f = (x) => x + 1');

            expect(cpp).toContain('[&](auto x) { return (x + 1); }');
        });
    });

    describe('new expressions', () => {
        test('novum with default', () => {
            const cpp = compile('novum Persona');

            expect(cpp).toContain('Persona{}');
        });

        test('novum with property overrides', () => {
            const cpp = compile('novum Persona { nomen: "Marcus" }');

            expect(cpp).toContain('Persona{.nomen = ');
        });
    });

    describe('includes', () => {
        test('generates required includes', () => {
            const cpp = compile('scribe "hello"');

            expect(cpp).toContain('#include <print>');
            expect(cpp).toContain('#include <string>');
        });

        test('optional includes added when used', () => {
            const cpp = compile('adfirma verum');

            expect(cpp).toContain('#include <cassert>');
        });
    });

    describe('main function', () => {
        test('runtime code wrapped in main', () => {
            const cpp = compile(`
                scribe "hello"
            `);

            expect(cpp).toContain('int main() {');
            expect(cpp).toContain('return 0;');
        });

        test('declarations outside main', () => {
            const cpp = compile(`
                functio greet() {
                    scribe "hi"
                }
                greet()
            `);

            // Function should be before main
            expect(cpp.indexOf('void greet')).toBeLessThan(cpp.indexOf('int main()'));
        });
    });
});
