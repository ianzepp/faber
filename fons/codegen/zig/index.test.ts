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

    return generate(program, { target: 'zig' });
}

describe('zig codegen', () => {
    describe('variable declarations', () => {
        test('varia -> var', () => {
            const zig = compile('varia x = 5');

            expect(zig).toContain('var x');
            expect(zig).toContain('= 5');
        });

        test('fixum -> const', () => {
            const zig = compile('fixum PI = 3.14');

            expect(zig).toContain('const PI');
            expect(zig).toContain('= 3.14');
        });

        test('type annotation', () => {
            const zig = compile('varia numerus x = 5');

            expect(zig).toContain('var x: i64 = 5');
        });
    });

    describe('function declarations', () => {
        test('simple function', () => {
            const zig = compile(`
        functio salve(textus nomen) -> nihil {
          redde
        }
      `);

            expect(zig).toContain('fn salve(nomen: []const u8) void');
            expect(zig).toContain('return;');
        });

        test('async function becomes error union', () => {
            const zig = compile(`
        futura functio fetch(textus url) -> numerus {
          redde 0
        }
      `);

            expect(zig).toContain('fn fetch(url: []const u8) !i64');
        });

        test('async with void return', () => {
            const zig = compile(`
                futura functio doWork() {
                    redde
                }
            `);

            expect(zig).toContain('fn doWork() !void');
        });

        test('cede becomes try', () => {
            const zig = compile(`
                futura functio getData() -> numerus {
                    fixum result = cede fetchData()
                    redde result
                }
            `);

            expect(zig).toContain('try fetchData()');
        });
    });

    describe('if statements', () => {
        test('simple if', () => {
            const zig = compile(`
        si verum {
          x = 1
        }
      `);

            expect(zig).toContain('if (true)');
        });

        test('if with else', () => {
            const zig = compile(`
        si falsum {
          a()
        }
        aliter {
          b()
        }
      `);

            expect(zig).toContain('if (false)');
            expect(zig).toContain('} else {');
        });
    });

    describe('loops', () => {
        test('while loop', () => {
            const zig = compile(`
        dum verum {
          x = 1
        }
      `);

            expect(zig).toContain('while (true)');
        });

        test('for loop over slice', () => {
            const zig = compile(`
        ex items pro item {
          process(item)
        }
      `);

            expect(zig).toContain('for (items) |item|');
        });

        test('for loop with range', () => {
            const zig = compile(`
        ex 0..10 pro i {
          process(i)
        }
      `);

            expect(zig).toContain('var i: usize = 0');
            expect(zig).toContain('while (i <= 10)');
            expect(zig).toContain('(i += 1)');
        });

        test('for loop with range and step', () => {
            const zig = compile(`
        ex 0..10 per 2 pro i {
          process(i)
        }
      `);

            expect(zig).toContain('while (i <= 10)');
            expect(zig).toContain('(i += 2)');
        });
    });

    describe('expressions', () => {
        test('binary operators', () => {
            const zig = compile('1 + 2');

            expect(zig).toContain('(1 + 2)');
        });

        test('logical and becomes and', () => {
            const zig = compile('a et b');

            expect(zig).toContain('(a and b)');
        });

        test('logical or becomes or', () => {
            const zig = compile('a aut b');

            expect(zig).toContain('(a or b)');
        });

        test('function call', () => {
            const zig = compile('salve(nomen)');

            expect(zig).toContain('salve(nomen)');
        });

        test('member access', () => {
            const zig = compile('usuario.nomen');

            expect(zig).toContain('usuario.nomen');
        });
    });

    describe('special expressions', () => {
        test('verum -> true', () => {
            const zig = compile('verum');

            expect(zig).toContain('true');
        });

        test('falsum -> false', () => {
            const zig = compile('falsum');

            expect(zig).toContain('false');
        });

        test('nihil -> null', () => {
            const zig = compile('nihil');

            expect(zig).toContain('null');
        });

        test('novum -> Type.init()', () => {
            const zig = compile('novum erratum(message)');

            expect(zig).toContain('erratum.init(message)');
        });
    });

    describe('guard statements', () => {
        test('custodi generates if statements', () => {
            const zig = compile(`
        custodi {
          si x == nihil { redde }
        }
      `);

            expect(zig).toContain('if ((x == null))');
            expect(zig).toContain('return;');
        });
    });

    describe('assert statements', () => {
        test('adfirma without message', () => {
            const zig = compile('adfirma x > 0');

            expect(zig).toContain('std.debug.assert((x > 0))');
        });

        test('adfirma with message uses panic', () => {
            const zig = compile('adfirma x > 0, "x must be positive"');

            expect(zig).toContain('if (!((x > 0)))');
            expect(zig).toContain('@panic("x must be positive")');
        });
    });

    describe('switch statements', () => {
        test('elige generates switch', () => {
            const zig = compile(`
        elige x {
          si 1 { a() }
          si 2 { b() }
        }
      `);

            expect(zig).toContain('switch (x)');
            expect(zig).toContain('1 =>');
            expect(zig).toContain('2 =>');
        });

        test('elige with default', () => {
            const zig = compile(`
        elige x {
          si 1 { a() }
          aliter { c() }
        }
      `);

            expect(zig).toContain('switch (x)');
            expect(zig).toContain('else =>');
        });
    });

    describe('with statements', () => {
        test('cum block expands to member assignments', () => {
            const zig = compile(`
        cum user {
          nomen = "Marcus"
          aetas = 30
        }
      `);

            expect(zig).toContain('user.nomen = "Marcus"');
            expect(zig).toContain('user.aetas = 30');
        });
    });

    describe('object destructuring', () => {
        test('destructuring expands to temp + member access', () => {
            const zig = compile('fixum { nomen, aetas } = user');

            expect(zig).toContain('const _tmp = user');
            expect(zig).toContain('const nomen = _tmp.nomen');
            expect(zig).toContain('const aetas = _tmp.aetas');
        });
    });

    describe('object literals', () => {
        test('empty object', () => {
            const zig = compile('fixum x = {}');

            expect(zig).toContain('const x = .{}');
        });

        test('object with properties', () => {
            const zig = compile('fixum user = { nomen: "Marcus", aetas: 30 }');

            expect(zig).toContain('.nomen = "Marcus"');
            expect(zig).toContain('.aetas = 30');
        });
    });

    describe('array literals', () => {
        test('empty array', () => {
            const zig = compile('fixum arr = []');

            expect(zig).toContain('const arr = .{}');
        });

        test('array with elements', () => {
            const zig = compile('fixum nums = [1, 2, 3]');

            expect(zig).toContain('.{ 1, 2, 3 }');
        });
    });

    describe('type declarations', () => {
        test('type alias', () => {
            const zig = compile('typus ID = textus');

            expect(zig).toContain('const ID = []const u8');
        });
    });

    describe('operator mapping', () => {
        test('est maps to ==', () => {
            const zig = compile(`
                varia x = 1
                varia y = 2
                si x est y { scribe "equal" }
            `);

            expect(zig).toContain('(x == y)');
        });

        test('&& maps to and', () => {
            const zig = compile('si verum et falsum { scribe "both" }');

            expect(zig).toContain('and');
            expect(zig).not.toContain('&&');
        });

        test('|| maps to or', () => {
            const zig = compile('si verum aut falsum { scribe "either" }');

            expect(zig).toContain('or');
            expect(zig).not.toContain('||');
        });
    });

    describe('missing features - nulla/nonnulla operators', () => {
        test('nulla generates null check', () => {
            const zig = compile('si nulla x { scribe "empty" }');

            expect(zig).toContain('if');
        });

        test('nonnulla generates non-null check', () => {
            const zig = compile('si nonnulla items { scribe "has" }');

            expect(zig).toContain('if');
        });

        test('nulla in assignment', () => {
            const zig = compile('fixum check = nulla x');

            expect(zig).toContain('const check');
        });
    });

    describe('missing features - iace (throw)', () => {
        test('iace with string', () => {
            const zig = compile('iace "error"');

            expect(zig).toContain('error');
        });

        test('iace with Error generates panic', () => {
            const zig = compile('iace novum Error("msg")');

            expect(zig).toContain('@panic');
        });

        test('iace in function', () => {
            const zig = compile(`
                functio validate() {
                    iace "error"
                }
            `);

            expect(zig).toContain('fn validate');
        });
    });

    describe('missing features - ergo one-liners', () => {
        test('si with ergo', () => {
            const zig = compile('si x > 5 ergo scribe "big"');

            expect(zig).toContain('if');
            expect(zig).toContain('(x > 5)');
        });

        test('dum with ergo', () => {
            const zig = compile('dum x > 0 ergo x = x - 1');

            expect(zig).toContain('while');
        });

        test('ex...pro with ergo', () => {
            const zig = compile('ex items pro item ergo process(item)');

            expect(zig).toContain('for');
        });
    });

    describe('missing features - scribe with multiple arguments', () => {
        test('scribe with multiple args', () => {
            const zig = compile('scribe "Name:", name');

            expect(zig).toContain('std.debug.print');
        });

        test('scribe with expressions', () => {
            const zig = compile('scribe x + y, a');

            expect(zig).toContain('std.debug.print');
        });
    });

    describe('genus declarations', () => {
        test('genus generates struct', () => {
            const zig = compile('genus persona { textus nomen: "X" }');

            expect(zig).toContain('const persona = struct {');
            expect(zig).toContain('nomen: []const u8 = "X"');
            expect(zig).toContain('};');
        });

        test('genus with multiple fields', () => {
            const zig = compile(`
                genus persona {
                    textus nomen: "anon"
                    numerus aetas: 0
                }
            `);

            expect(zig).toContain('nomen: []const u8 = "anon"');
            expect(zig).toContain('aetas: i64 = 0');
        });

        test('genus generates init with @hasField', () => {
            const zig = compile('genus persona { textus nomen: "X" }');

            expect(zig).toContain('pub fn init(overrides: anytype) Self');
            expect(zig).toContain('@hasField(@TypeOf(overrides), "nomen")');
            expect(zig).toContain('return self;');
        });

        test('genus with creo calls creo in init', () => {
            const zig = compile(`
                genus persona {
                    numerus aetas: 0
                    functio creo() {
                        si ego.aetas < 0 { ego.aetas = 0 }
                    }
                }
            `);

            expect(zig).toContain('self.creo();');
            expect(zig).toContain('fn creo(self: *Self) void');
        });

        test('genus without creo does not call creo', () => {
            const zig = compile('genus persona { textus nomen: "X" }');

            expect(zig).not.toContain('self.creo()');
        });

        test('genus with method generates pub fn', () => {
            const zig = compile(`
                genus persona {
                    textus nomen: "X"
                    functio saluta() -> textus { redde ego.nomen }
                }
            `);

            expect(zig).toContain('pub fn saluta(self: *const Self) []const u8');
            expect(zig).toContain('return self.nomen');
        });

        test('genus generates Self = @This()', () => {
            const zig = compile(`
                genus persona {
                    textus nomen: "X"
                    functio saluta() -> textus { redde ego.nomen }
                }
            `);

            expect(zig).toContain('const Self = @This();');
        });

        test('ego becomes self in methods', () => {
            const zig = compile(`
                genus persona {
                    textus nomen: "X"
                    functio saluta() { redde ego.nomen }
                }
            `);

            expect(zig).toContain('return self.nomen');
            expect(zig).not.toContain('ego');
        });

        test('novum becomes Type.init()', () => {
            const zig = compile('fixum p = novum persona');

            expect(zig).toContain('persona.init()');
        });

        test('novum with cum passes overrides', () => {
            const zig = compile('fixum p = novum persona cum { nomen: "Claudia" }');

            expect(zig).toContain('persona.init(.{ .nomen = "Claudia" })');
        });
    });

    describe('pactum declarations', () => {
        test('pactum generates comment', () => {
            const zig = compile(`
                pactum iterabilis {
                    functio sequens() -> textus?
                }
            `);

            expect(zig).toContain('// pactum iterabilis');
            expect(zig).toContain('interface contract');
        });

        test('pactum documents required methods', () => {
            const zig = compile(`
                pactum iterabilis {
                    functio sequens() -> textus?
                    functio reset()
                }
            `);

            expect(zig).toContain('requires fn sequens');
            expect(zig).toContain('requires fn reset');
        });
    });
});
