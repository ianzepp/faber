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

    // WHY: Semantic analysis populates resolvedType on AST nodes,
    // which is needed for collection method dispatch (tabula vs copia vs lista)
    const { program: analyzedProgram } = analyze(program);

    return generate(analyzedProgram, { target: 'cpp' });
}

describe('cpp codegen', () => {
    // MIGRATED to fundamenta.yaml
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

    // MIGRATED to functiones.yaml
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

    // MIGRATED to regimen.yaml
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

        test('ante keyword (explicit exclusive)', () => {
            const cpp = compile(`
                ex 0 ante 10 pro i {
                    scribe i
                }
            `);

            expect(cpp).toContain('for (int64_t i = 0; i < 10; i += 1)');
        });

        test('usque keyword (inclusive)', () => {
            const cpp = compile(`
                ex 0 usque 10 pro i {
                    scribe i
                }
            `);

            expect(cpp).toContain('for (int64_t i = 0; i <= 10; i += 1)');
        });
    });

    // MIGRATED to operatores.yaml
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

    // MIGRATED to fundamenta.yaml
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

    // MIGRATED to fundamenta.yaml
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

    // MIGRATED to structurae.yaml
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

        test('auto-merge constructor - default constructor', () => {
            const cpp = compile(`
                genus Persona {
                    textus nomen: "anon"
                    numerus aetas: 0
                }
            `);

            expect(cpp).toContain('Persona() = default;');
        });

        test('auto-merge constructor - template with requires', () => {
            const cpp = compile(`
                genus Persona {
                    textus nomen: "anon"
                    numerus aetas: 0
                }
            `);

            expect(cpp).toContain('template<typename Overrides>');
            expect(cpp).toContain('requires std::is_aggregate_v<Overrides>');
            expect(cpp).toContain('Persona(const Overrides& o)');
        });

        test('auto-merge constructor - if constexpr field checks', () => {
            const cpp = compile(`
                genus Persona {
                    textus nomen: "anon"
                    numerus aetas: 0
                }
            `);

            expect(cpp).toContain('if constexpr (requires { o.nomen; }) nomen = o.nomen;');
            expect(cpp).toContain('if constexpr (requires { o.aetas; }) aetas = o.aetas;');
        });

        test('auto-merge constructor - includes type_traits', () => {
            const cpp = compile(`
                genus Persona {
                    textus nomen: "anon"
                }
            `);

            expect(cpp).toContain('#include <type_traits>');
        });

        test('genus with creo emits _creo method', () => {
            const cpp = compile(`
                genus Persona {
                    textus nomen: "anon"
                    functio creo() {
                        ego.nomen = "initialized"
                    }
                }
            `);

            expect(cpp).toContain('void _creo()');
            expect(cpp).toContain('this->nomen = std::string("initialized")');
        });

        test('genus with creo - constructor calls _creo', () => {
            const cpp = compile(`
                genus Persona {
                    textus nomen: "anon"
                    functio creo() {
                        ego.nomen = "initialized"
                    }
                }
            `);

            expect(cpp).toContain('_creo();');
        });

        test('genus without creo - no _creo call', () => {
            const cpp = compile(`
                genus Persona {
                    textus nomen: "anon"
                }
            `);

            expect(cpp).not.toContain('_creo();');
            expect(cpp).not.toContain('void _creo()');
        });
    });

    // MIGRATED to structurae.yaml
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

    // MIGRATED to errores.yaml
    describe('error handling', () => {
        test('iace (throw) statement', () => {
            const cpp = compile('iace "error"');

            expect(cpp).toContain('throw std::runtime_error');
        });

        test('mori (panic) with string', () => {
            const cpp = compile('mori "fatal error"');

            expect(cpp).toContain('#include <cstdlib>');
            expect(cpp).toContain('std::print(stderr');
            expect(cpp).toContain('FATAL:');
            expect(cpp).toContain('std::abort()');
        });

        test('mori (panic) with expression', () => {
            const cpp = compile('mori errMsg');

            expect(cpp).toContain('std::print(stderr');
            expect(cpp).toContain('std::abort()');
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

        test('demum generates scope guard', () => {
            const cpp = compile(`
                tempta {
                    risky()
                } demum {
                    cleanup()
                }
            `);

            // Should have scope guard helper
            expect(cpp).toContain('struct _ScopeGuard');
            expect(cpp).toContain('~_ScopeGuard()');

            // Should create scope guard before try
            expect(cpp).toContain('auto _demum_');
            expect(cpp).toContain('_ScopeGuard([&]{');
            expect(cpp).toContain('cleanup()');
        });

        test('demum with cape', () => {
            const cpp = compile(`
                tempta {
                    risky()
                } cape err {
                    handle(err)
                } demum {
                    cleanup()
                }
            `);

            expect(cpp).toContain('try {');
            expect(cpp).toContain('catch (const std::exception& err)');
            expect(cpp).toContain('_ScopeGuard([&]{');
            expect(cpp).toContain('cleanup()');
        });
    });

    describe('praefixum (constexpr)', () => {
        test('praefixum expression form', () => {
            const cpp = compile('fixum size = praefixum(256 * 4)');

            expect(cpp).toContain('(256 * 4)');
        });

        test('praefixum block form', () => {
            const cpp = compile(`
                fixum result = praefixum {
                    varia x = 10
                    redde x * 2
                }
            `);

            expect(cpp).toContain('[&]');
            expect(cpp).toContain('auto x = 10');
            expect(cpp).toContain('return (x * 2)');
            expect(cpp).toContain('}()');
        });
    });

    // MIGRATED to functiones.yaml
    describe('lambda expressions', () => {
        test('arrow function', () => {
            const cpp = compile('fixum f = (x) => x + 1');

            expect(cpp).toContain('[&](auto x) { return (x + 1); }');
        });
    });

    // MIGRATED to structurae.yaml
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

    describe('lista methods', () => {
        describe('adding elements', () => {
            test('adde -> push_back', () => {
                const cpp = compile('items.adde(1)');

                expect(cpp).toContain('.push_back(1)');
            });

            test('addita -> copy and push_back', () => {
                const cpp = compile('items.addita(1)');

                expect(cpp).toContain('auto v = items');
                expect(cpp).toContain('v.push_back(1)');
                expect(cpp).toContain('return v');
            });

            test('praepone -> insert at begin', () => {
                const cpp = compile('items.praepone(1)');

                expect(cpp).toContain('.insert(');
                expect(cpp).toContain('.begin()');
            });
        });

        describe('removing elements', () => {
            test('remove -> pop_back with return', () => {
                const cpp = compile('items.remove()');

                expect(cpp).toContain('.back()');
                expect(cpp).toContain('.pop_back()');
            });

            test('decapita -> erase at begin', () => {
                const cpp = compile('items.decapita()');

                expect(cpp).toContain('.front()');
                expect(cpp).toContain('.erase(');
            });

            test('purga -> clear', () => {
                const cpp = compile('items.purga()');

                expect(cpp).toContain('.clear()');
            });
        });

        describe('accessing elements', () => {
            test('primus -> front()', () => {
                const cpp = compile('items.primus()');

                expect(cpp).toContain('.front()');
            });

            test('ultimus -> back()', () => {
                const cpp = compile('items.ultimus()');

                expect(cpp).toContain('.back()');
            });

            test('accipe -> at()', () => {
                const cpp = compile('items.accipe(0)');

                expect(cpp).toContain('.at(0)');
            });

            test('longitudo -> size()', () => {
                const cpp = compile('items.longitudo()');

                expect(cpp).toContain('.size()');
            });

            test('vacua -> empty()', () => {
                const cpp = compile('items.vacua()');

                expect(cpp).toContain('.empty()');
            });
        });

        describe('searching', () => {
            test('continet -> std::find', () => {
                const cpp = compile('items.continet(5)');

                expect(cpp).toContain('std::find(');
                expect(cpp).toContain('!= items.end()');
            });

            test('indiceDe -> std::find with distance', () => {
                const cpp = compile('items.indiceDe(5)');

                expect(cpp).toContain('std::find(');
                expect(cpp).toContain('std::distance(');
            });
        });

        describe('transformations (C++23 ranges)', () => {
            test('filtrata -> views::filter | ranges::to', () => {
                const cpp = compile('items.filtrata(pro x: x > 0)');

                expect(cpp).toContain('std::views::filter');
                expect(cpp).toContain('std::ranges::to<std::vector>');
            });

            test('mappata -> views::transform | ranges::to', () => {
                const cpp = compile('items.mappata(pro x: x * 2)');

                expect(cpp).toContain('std::views::transform');
                expect(cpp).toContain('std::ranges::to<std::vector>');
            });

            test('inversa -> views::reverse | ranges::to', () => {
                const cpp = compile('items.inversa()');

                expect(cpp).toContain('std::views::reverse');
                expect(cpp).toContain('std::ranges::to<std::vector>');
            });

            test('prima -> views::take | ranges::to', () => {
                const cpp = compile('items.prima(5)');

                expect(cpp).toContain('std::views::take(5)');
                expect(cpp).toContain('std::ranges::to<std::vector>');
            });

            test('omitte -> views::drop | ranges::to', () => {
                const cpp = compile('items.omitte(2)');

                expect(cpp).toContain('std::views::drop(2)');
                expect(cpp).toContain('std::ranges::to<std::vector>');
            });
        });

        describe('predicates', () => {
            test('omnes -> ranges::all_of', () => {
                const cpp = compile('items.omnes(pro x: x > 0)');

                expect(cpp).toContain('std::ranges::all_of(');
            });

            test('aliquis -> ranges::any_of', () => {
                const cpp = compile('items.aliquis(pro x: x > 0)');

                expect(cpp).toContain('std::ranges::any_of(');
            });
        });

        describe('mutating operations', () => {
            test('ordina -> ranges::sort (in-place)', () => {
                const cpp = compile('items.ordina()');

                expect(cpp).toContain('std::ranges::sort(items)');
            });

            test('inverte -> ranges::reverse (in-place)', () => {
                const cpp = compile('items.inverte()');

                expect(cpp).toContain('std::ranges::reverse(items)');
            });
        });

        describe('numeric aggregation', () => {
            test('summa -> std::accumulate', () => {
                const cpp = compile('nums.summa()');

                expect(cpp).toContain('std::accumulate(');
            });

            test('minimus -> ranges::min_element', () => {
                const cpp = compile('nums.minimus()');

                expect(cpp).toContain('std::ranges::min_element(');
            });

            test('maximus -> ranges::max_element', () => {
                const cpp = compile('nums.maximus()');

                expect(cpp).toContain('std::ranges::max_element(');
            });
        });

        describe('iteration', () => {
            test('perambula -> ranges::for_each', () => {
                const cpp = compile('items.perambula(pro x { scribe x })');

                expect(cpp).toContain('std::ranges::for_each(');
            });
        });

        describe('includes headers', () => {
            test('algorithm methods add <algorithm> header', () => {
                const cpp = compile('items.continet(5)');

                expect(cpp).toContain('#include <algorithm>');
            });

            test('ranges methods add <ranges> header', () => {
                const cpp = compile('items.filtrata(pro x: x > 0)');

                expect(cpp).toContain('#include <ranges>');
            });

            test('numeric methods add <numeric> header', () => {
                const cpp = compile('nums.summa()');

                expect(cpp).toContain('#include <numeric>');
            });
        });
    });

    describe('tabula methods', () => {
        describe('core operations', () => {
            test('pone -> insert_or_assign', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    map.pone("key", 42)
                `);

                expect(cpp).toContain('.insert_or_assign(');
            });

            test('accipe -> at()', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    map.accipe("key")
                `);

                expect(cpp).toContain('.at(');
            });

            test('habet -> contains()', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    map.habet("key")
                `);

                expect(cpp).toContain('.contains(');
            });

            test('dele -> erase()', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    map.dele("key")
                `);

                expect(cpp).toContain('.erase(');
            });

            test('longitudo -> size()', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    map.longitudo()
                `);

                expect(cpp).toContain('.size()');
            });

            test('vacua -> empty()', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    map.vacua()
                `);

                expect(cpp).toContain('.empty()');
            });

            test('purga -> clear()', () => {
                const cpp = compile(`
                    varia tabula<textus, numerus> map = {}
                    map.purga()
                `);

                expect(cpp).toContain('.clear()');
            });
        });

        describe('iteration', () => {
            test('claves -> views::keys', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    map.claves()
                `);

                expect(cpp).toContain('std::views::keys');
            });

            test('valores -> views::values', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    map.valores()
                `);

                expect(cpp).toContain('std::views::values');
            });
        });

        describe('extended methods', () => {
            test('accipeAut -> contains + at with default', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    map.accipeAut("key", 0)
                `);

                expect(cpp).toContain('.contains(');
                expect(cpp).toContain('.at(');
            });

            test('confla -> merge maps', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    fixum tabula<textus, numerus> other = {}
                    map.confla(other)
                `);

                expect(cpp).toContain('for (auto& [k, v]');
            });

            test('inLista -> vector from map', () => {
                const cpp = compile(`
                    fixum tabula<textus, numerus> map = {}
                    map.inLista()
                `);

                expect(cpp).toContain('std::vector(');
            });
        });
    });

    describe('copia methods', () => {
        describe('core operations', () => {
            test('adde -> insert()', () => {
                const cpp = compile(`
                    varia copia<numerus> set = {}
                    set.adde(5)
                `);

                expect(cpp).toContain('.insert(5)');
            });

            test('habet -> contains()', () => {
                const cpp = compile(`
                    fixum copia<numerus> set = {}
                    set.habet(5)
                `);

                expect(cpp).toContain('.contains(5)');
            });

            test('dele -> erase()', () => {
                const cpp = compile(`
                    varia copia<numerus> set = {}
                    set.dele(5)
                `);

                expect(cpp).toContain('.erase(5)');
            });

            test('longitudo -> size()', () => {
                const cpp = compile(`
                    fixum copia<numerus> set = {}
                    set.longitudo()
                `);

                expect(cpp).toContain('.size()');
            });

            test('vacua -> empty()', () => {
                const cpp = compile(`
                    fixum copia<numerus> set = {}
                    set.vacua()
                `);

                expect(cpp).toContain('.empty()');
            });

            test('purga -> clear()', () => {
                const cpp = compile(`
                    varia copia<numerus> set = {}
                    set.purga()
                `);

                expect(cpp).toContain('.clear()');
            });
        });

        describe('set operations', () => {
            test('unio -> set union', () => {
                const cpp = compile(`
                    fixum copia<numerus> setA = {}
                    fixum copia<numerus> setB = {}
                    setA.unio(setB)
                `);

                expect(cpp).toContain('std::unordered_set');
                expect(cpp).toContain('.insert(');
            });

            test('intersectio -> set intersection', () => {
                const cpp = compile(`
                    fixum copia<numerus> setA = {}
                    fixum copia<numerus> setB = {}
                    setA.intersectio(setB)
                `);

                expect(cpp).toContain('.contains(');
                expect(cpp).toContain('.insert(');
            });

            test('differentia -> set difference', () => {
                const cpp = compile(`
                    fixum copia<numerus> setA = {}
                    fixum copia<numerus> setB = {}
                    setA.differentia(setB)
                `);

                expect(cpp).toContain('!');
                expect(cpp).toContain('.contains(');
            });

            test('symmetrica -> symmetric difference', () => {
                const cpp = compile(`
                    fixum copia<numerus> setA = {}
                    fixum copia<numerus> setB = {}
                    setA.symmetrica(setB)
                `);

                expect(cpp).toContain('!');
                // Should check both directions
                expect(cpp.match(/\.contains\(/g)?.length).toBeGreaterThanOrEqual(2);
            });
        });

        describe('predicates', () => {
            test('subcopia -> all_of with contains', () => {
                const cpp = compile(`
                    fixum copia<numerus> setA = {}
                    fixum copia<numerus> setB = {}
                    setA.subcopia(setB)
                `);

                expect(cpp).toContain('std::ranges::all_of');
                expect(cpp).toContain('.contains(');
            });

            test('supercopia -> all_of reversed', () => {
                const cpp = compile(`
                    fixum copia<numerus> setA = {}
                    fixum copia<numerus> setB = {}
                    setA.supercopia(setB)
                `);

                expect(cpp).toContain('std::ranges::all_of');
            });
        });

        describe('conversions', () => {
            test('inLista -> vector from set', () => {
                const cpp = compile(`
                    fixum copia<numerus> set = {}
                    set.inLista()
                `);

                expect(cpp).toContain('std::vector(');
            });
        });

        describe('iteration', () => {
            test('perambula -> ranges::for_each', () => {
                const cpp = compile(`
                    fixum copia<numerus> set = {}
                    set.perambula(pro x { scribe x })
                `);

                expect(cpp).toContain('std::ranges::for_each(');
            });
        });
    });
});
