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

    return generate(analyzedProgram, { target: 'zig' });
}

describe('zig codegen', () => {
    describe('variable declarations', () => {
        test('varia -> var', () => {
            const zig = compile('varia x = 5');

            expect(zig).toContain('var x');
            expect(zig).toContain('= 5');
        });

        test('fixum -> const with m_ prefix at module level', () => {
            const zig = compile('fixum PI = 3.14');

            // WHY: Module-level constants use m_ prefix to avoid shadowing with function params
            expect(zig).toContain('const m_PI');
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

    describe('ownership prepositions (de/in)', () => {
        describe('de (borrowed, read-only)', () => {
            test('de textus remains slice', () => {
                const zig = compile(`
                    functio length(de textus source) -> numerus {
                        redde 0
                    }
                `);

                // WHY: textus is already []const u8, de doesn't change it
                expect(zig).toContain('fn length(source: []const u8) i64');
            });

            test('de numerus becomes const pointer', () => {
                const zig = compile(`
                    functio read(de numerus value) -> numerus {
                        redde 0
                    }
                `);

                // WHY: Primitives need pointer for borrow semantics
                expect(zig).toContain('fn read(value: *const i64) i64');
            });

            test('de lista becomes const slice', () => {
                const zig = compile(`
                    functio sum(de lista<numerus> items) -> numerus {
                        redde 0
                    }
                `);

                // WHY: Borrowed list exposes items as const slice, not ArrayList
                expect(zig).toContain('fn sum(items: []const i64) i64');
            });

            test('de with user-defined type becomes const pointer', () => {
                const zig = compile(`
                    functio inspect(de persona p) -> numerus {
                        redde 0
                    }
                `);

                expect(zig).toContain('fn inspect(p: *const persona) i64');
            });
        });

        describe('in (mutable borrow)', () => {
            test('in textus becomes mutable pointer to slice', () => {
                const zig = compile(`
                    functio modify(in textus target) {
                        redde
                    }
                `);

                // WHY: Mutable string needs pointer to mutable slice
                expect(zig).toContain('fn modify(target: *[]u8) void');
            });

            test('in numerus becomes mutable pointer', () => {
                const zig = compile(`
                    functio increment(in numerus value) {
                        redde
                    }
                `);

                expect(zig).toContain('fn increment(value: *i64) void');
            });

            test('in lista becomes mutable pointer to ArrayList', () => {
                const zig = compile(`
                    functio append(in lista<numerus> items, numerus value) {
                        redde
                    }
                `);

                // WHY: Mutable list access needs pointer to ArrayList
                expect(zig).toContain('fn append(items: *std.ArrayList(i64), value: i64) void');
            });

            test('in with user-defined type becomes mutable pointer', () => {
                const zig = compile(`
                    functio update(in persona p) {
                        redde
                    }
                `);

                expect(zig).toContain('fn update(p: *persona) void');
            });
        });

        describe('mixed prepositions', () => {
            test('function with de and in parameters', () => {
                const zig = compile(`
                    functio copy(de textus source, in textus target) {
                        redde
                    }
                `);

                expect(zig).toContain('source: []const u8');
                expect(zig).toContain('target: *[]u8');
            });

            test('function with owned and borrowed parameters', () => {
                const zig = compile(`
                    functio process(de lista<numerus> input, numerus multiplier) -> numerus {
                        redde 0
                    }
                `);

                // de lista = borrowed slice, no preposition numerus = owned value
                expect(zig).toContain('input: []const i64');
                expect(zig).toContain('multiplier: i64');
            });
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
            expect(zig).toContain('while (i < 10)');
            expect(zig).toContain('(i += 1)');
        });

        test('for loop with range and step', () => {
            const zig = compile(`
        ex 0..10 per 2 pro i {
          process(i)
        }
      `);

            expect(zig).toContain('while (i < 10)');
            expect(zig).toContain('(i += 2)');
        });

        test('ante keyword (explicit exclusive)', () => {
            const zig = compile(`
        ex 0 ante 10 pro i {
          process(i)
        }
      `);

            expect(zig).toContain('while (i < 10)');
        });

        test('usque keyword (inclusive)', () => {
            const zig = compile(`
        ex 0 usque 10 pro i {
          process(i)
        }
      `);

            expect(zig).toContain('while (i <= 10)');
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
        test('elige generates if-else chain', () => {
            const zig = compile(`
        elige x {
          si 1 { a() }
          si 2 { b() }
        }
      `);

            expect(zig).toContain('if ((x == 1))');
            expect(zig).toContain('else if ((x == 2))');
        });

        test('elige with default', () => {
            const zig = compile(`
        elige x {
          si 1 { a() }
          aliter { c() }
        }
      `);

            expect(zig).toContain('if ((x == 1))');
            expect(zig).toContain('else {');
        });

        test('elige with strings uses std.mem.eql', () => {
            const zig = compile(`
        elige status {
          si "active" { a() }
          si "pending" { b() }
        }
      `);

            expect(zig).toContain('std.mem.eql(u8, status, "active")');
            expect(zig).toContain('std.mem.eql(u8, status, "pending")');
        });
    });

    describe('with statements', () => {
        test('in block expands to member assignments', () => {
            const zig = compile(`
        in user {
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

        test('union type falls back to anytype', () => {
            // WHY: Zig doesn't support untagged unions like TS's A | B.
            // For unio<A, B>, we emit anytype as a fallback.
            // Use discretio (tagged unions) for proper Zig support.
            const zig = compile('fixum unio<textus, numerus> value = "hello"');

            expect(zig).toContain('anytype');
        });

        test('type alias with union falls back to anytype', () => {
            const zig = compile('typus Json = unio<textus, numerus, bivalens, nihil>');

            expect(zig).toContain('const Json = anytype');
        });
    });

    describe('operator mapping', () => {
        test('=== maps to ==', () => {
            const zig = compile(`
                varia x = 1
                varia y = 2
                si x === y { scribe "equal" }
            `);

            expect(zig).toContain('(x == y)');
        });

        test('est with type uses @TypeOf comparison', () => {
            const zig = compile(`
                varia x = 1
                si x est numerus { scribe "is number" }
            `);

            expect(zig).toContain('@TypeOf(x) == i64');
        });

        test('nihil unary uses null comparison', () => {
            const zig = compile(`
                varia x: numerus? = nihil
                si nihil x { scribe "null" }
            `);

            expect(zig).toContain('(x == null)');
        });

        test('nonnihil unary uses != null', () => {
            const zig = compile(`
                varia x: numerus? = 42
                si nonnihil x { scribe "not null" }
            `);

            expect(zig).toContain('(x != null)');
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

        test('bitwise AND', () => {
            const zig = compile('fixum x = 5 & 3');

            expect(zig).toContain('(5 & 3)');
        });

        test('bitwise OR', () => {
            const zig = compile('fixum x = 5 | 3');

            expect(zig).toContain('(5 | 3)');
        });

        test('bitwise XOR', () => {
            const zig = compile('fixum x = 5 ^ 3');

            expect(zig).toContain('(5 ^ 3)');
        });

        test('bitwise NOT', () => {
            const zig = compile('fixum x = ~5');

            expect(zig).toContain('~5');
        });

        test('left shift', () => {
            const zig = compile('fixum x = 1 << 4');

            expect(zig).toContain('(1 << 4)');
        });

        test('right shift', () => {
            const zig = compile('fixum x = 16 >> 2');

            expect(zig).toContain('(16 >> 2)');
        });
    });

    describe('string comparison', () => {
        test('string === uses std.mem.eql', () => {
            // WHY: === for value equality, Zig requires std.mem.eql for strings
            const zig = compile('si status === "active" { scribe "ok" }');

            expect(zig).toContain('std.mem.eql(u8, status, "active")');
            expect(zig).not.toContain('status == "active"');
        });

        test('string !== uses !std.mem.eql', () => {
            // WHY: !== for value inequality
            const zig = compile('si status !== "pending" { scribe "done" }');

            expect(zig).toContain('!std.mem.eql(u8, status, "pending")');
            expect(zig).not.toContain('status != "pending"');
        });

        test('two string literals use std.mem.eql', () => {
            const zig = compile('si "hello" === "world" { scribe "match" }');

            expect(zig).toContain('std.mem.eql(u8, "hello", "world")');
        });

        test('string concatenation uses ++', () => {
            const zig = compile('fixum greeting = "Hello, " + "World"');

            expect(zig).toContain('("Hello, " ++ "World")');
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

    describe('error handling - iace and mori', () => {
        test('iace with string returns error', () => {
            const zig = compile('iace "timeout"');

            // WHY: iace is recoverable, generates return error.X
            expect(zig).toContain('return error.Timeout');
        });

        test('iace with Error returns error', () => {
            const zig = compile('iace novum Error("invalid input")');

            // WHY: Error message converted to PascalCase error name
            expect(zig).toContain('return error.InvalidInput');
        });

        test('mori generates panic', () => {
            const zig = compile('mori "fatal crash"');

            // WHY: mori is fatal/unrecoverable, generates @panic
            expect(zig).toContain('@panic("fatal crash")');
        });

        test('mori with Error generates panic', () => {
            const zig = compile('mori novum Error("out of memory")');

            expect(zig).toContain('@panic("out of memory")');
        });

        test('function with iace has error union return type', () => {
            const zig = compile(`
                functio validate(numerus x) -> numerus {
                    si x < 0 { iace "negative value" }
                    redde x
                }
            `);

            // WHY: Function containing iace needs !T return type
            expect(zig).toContain('fn validate(x: i64) !i64');
            expect(zig).toContain('return error.NegativeValue');
        });

        test('function with mori does NOT get error union', () => {
            const zig = compile(`
                functio crash() {
                    mori "goodbye"
                }
            `);

            // WHY: mori is panic, not recoverable error - no !T needed
            expect(zig).toContain('fn crash() void');
            expect(zig).toContain('@panic("goodbye")');
        });

        test('async function with iace keeps error union', () => {
            const zig = compile(`
                futura functio fetch() -> numerus {
                    iace "network error"
                }
            `);

            // WHY: Both async and iace need error union - should still be !T
            expect(zig).toContain('fn fetch() !i64');
            expect(zig).toContain('return error.NetworkError');
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

        test('novum without overrides passes .{}', () => {
            const zig = compile('fixum p = novum persona');

            // WHY: init() uses @hasField pattern, needs empty struct when no overrides
            expect(zig).toContain('persona.init(.{})');
        });

        test('novum with property overrides', () => {
            const zig = compile('fixum p = novum persona { nomen: "Claudia" }');

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

    describe('prae (compile-time)', () => {
        describe('prae typus (type parameters)', () => {
            test('single type parameter becomes comptime T: type', () => {
                const zig = compile(`
                    functio identity(prae typus T, T value) -> T {
                        redde value
                    }
                `);

                expect(zig).toContain('fn identity(comptime T: type, value: T) T');
                expect(zig).toContain('return value');
            });

            test('multiple type parameters', () => {
                const zig = compile(`
                    functio pair(prae typus K, prae typus V, K key, V value) {
                        redde
                    }
                `);

                expect(zig).toContain('comptime K: type');
                expect(zig).toContain('comptime V: type');
                expect(zig).toContain('key: K');
                expect(zig).toContain('value: V');
            });

            test('type parameter only (no regular params)', () => {
                const zig = compile(`
                    functio sizeof(prae typus T) -> numerus {
                        redde 0
                    }
                `);

                expect(zig).toContain('fn sizeof(comptime T: type) i64');
            });
        });

        describe('praefixum (compile-time expressions)', () => {
            test('praefixum expression becomes comptime', () => {
                const zig = compile('fixum size = praefixum(256 * 4)');

                expect(zig).toContain('comptime ((256 * 4))');
            });

            test('praefixum block becomes comptime blk', () => {
                const zig = compile(`
                    fixum table = praefixum {
                        varia x = 10
                        redde x
                    }
                `);

                expect(zig).toContain('comptime blk: {');
                expect(zig).toContain('var x: i64 = 10');
                expect(zig).toContain('break :blk x');
            });

            test('praefixum with binary expression', () => {
                const zig = compile('fixum mask = praefixum(0xFF - 0xAA)');

                expect(zig).toContain('comptime ((0xFF - 0xAA))');
            });
        });
    });

    describe('hex literals', () => {
        test('hex literal passes through', () => {
            const zig = compile('fixum mask = 0xFF');

            expect(zig).toContain('const m_mask');
            expect(zig).toContain('= 0xFF');
        });

        test('hex literal in expression', () => {
            const zig = compile('fixum result = 0xFF - 0xAA');

            expect(zig).toContain('const m_result');
            expect(zig).toContain('0xFF');
            expect(zig).toContain('0xAA');
        });

        test('hex bigint strips n suffix', () => {
            const zig = compile('fixum big = 0xFFFFFFFFFFn');

            // Zig comptime_int is arbitrary precision, no n suffix needed
            expect(zig).toContain('const m_big');
            expect(zig).toContain('= 0xFFFFFFFFFF');
            expect(zig).not.toContain('n;');
        });

        test('lowercase hex', () => {
            const zig = compile('fixum val = 0xabcdef');

            expect(zig).toContain('const m_val');
            expect(zig).toContain('= 0xabcdef');
        });
    });

    // =========================================================================
    // LISTA METHODS - Latin Array API for Zig
    // =========================================================================
    describe('lista methods - Latin ArrayList API', () => {
        // Helper to compile with lista variable in scope
        const listaCompile = (expr: string) => compile(`fixum lista<numerus> items = []\n${expr}`);

        describe('arena preamble', () => {
            test('emits arena allocator when lista is used', () => {
                const zig = listaCompile('items.adde(1)');

                expect(zig).toContain('var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator)');
                expect(zig).toContain('defer arena.deinit()');
                expect(zig).toContain('const alloc = arena.allocator()');
            });
        });

        describe('adding elements', () => {
            test('adde -> append with allocator', () => {
                const zig = listaCompile('items.adde(42)');

                expect(zig).toContain('items.append(alloc, 42)');
                expect(zig).toContain('catch @panic("OOM")');
            });

            test('praepone -> insert at 0', () => {
                const zig = listaCompile('items.praepone(1)');

                expect(zig).toContain('items.insert(alloc, 0, 1)');
            });

            test('addita -> compileError (not implemented)', () => {
                const zig = listaCompile('items.addita(1)');

                expect(zig).toContain('@compileError');
            });
        });

        describe('removing elements', () => {
            test('remove -> pop', () => {
                const zig = listaCompile('items.remove()');

                expect(zig).toContain('items.pop()');
            });

            test('decapita -> orderedRemove(0)', () => {
                const zig = listaCompile('items.decapita()');

                expect(zig).toContain('items.orderedRemove(0)');
            });

            test('purga -> clearRetainingCapacity', () => {
                const zig = listaCompile('items.purga()');

                expect(zig).toContain('items.clearRetainingCapacity()');
            });
        });

        describe('accessing elements', () => {
            test('primus -> items[0]', () => {
                const zig = listaCompile('items.primus()');

                expect(zig).toContain('items.items[0]');
            });

            test('ultimus -> items[items.len - 1]', () => {
                const zig = listaCompile('items.ultimus()');

                expect(zig).toContain('items.items[items.items.len - 1]');
            });

            test('accipe -> items[index]', () => {
                const zig = listaCompile('items.accipe(2)');

                expect(zig).toContain('items.items[2]');
            });
        });

        describe('properties', () => {
            test('longitudo -> items.len', () => {
                const zig = listaCompile('items.longitudo()');

                expect(zig).toContain('items.items.len');
            });

            test('vacua -> items.len == 0', () => {
                const zig = listaCompile('items.vacua()');

                expect(zig).toContain('items.items.len == 0');
            });
        });

        describe('unimplemented methods', () => {
            test('filtrata -> compileError', () => {
                const zig = listaCompile('items.filtrata(fn)');

                expect(zig).toContain('@compileError');
                expect(zig).toContain('not implemented');
            });

            test('mappata -> compileError', () => {
                const zig = listaCompile('items.mappata(fn)');

                expect(zig).toContain('@compileError');
            });

            test('reducta -> compileError', () => {
                const zig = listaCompile('items.reducta(0, fn)');

                expect(zig).toContain('@compileError');
            });
        });
    });

    // =========================================================================
    // TABULA METHODS - Latin HashMap API for Zig
    // =========================================================================
    describe('tabula methods - Latin HashMap API', () => {
        const tabulaCompile = (expr: string) => compile(`fixum tabula<textus, numerus> map = novum tabula()\n${expr}`);

        describe('core operations', () => {
            test('pone -> put with allocator', () => {
                const zig = tabulaCompile('map.pone(key, 42)');

                expect(zig).toContain('map.put(alloc, key, 42)');
                expect(zig).toContain('catch @panic("OOM")');
            });

            test('accipe -> get', () => {
                const zig = tabulaCompile('map.accipe(key)');

                expect(zig).toContain('map.get(key)');
            });

            test('habet -> contains', () => {
                const zig = tabulaCompile('map.habet(key)');

                expect(zig).toContain('map.contains(key)');
            });

            test('dele -> remove', () => {
                const zig = tabulaCompile('map.dele(key)');

                expect(zig).toContain('map.remove(key)');
            });

            test('longitudo -> count', () => {
                const zig = tabulaCompile('map.longitudo()');

                expect(zig).toContain('map.count()');
            });

            test('vacua -> count == 0', () => {
                const zig = tabulaCompile('map.vacua()');

                expect(zig).toContain('map.count() == 0');
            });

            test('purga -> clearRetainingCapacity', () => {
                const zig = tabulaCompile('map.purga()');

                expect(zig).toContain('map.clearRetainingCapacity()');
            });
        });

        describe('iteration', () => {
            test('claves -> keyIterator', () => {
                const zig = tabulaCompile('map.claves()');

                expect(zig).toContain('map.keyIterator()');
            });

            test('valores -> valueIterator', () => {
                const zig = tabulaCompile('map.valores()');

                expect(zig).toContain('map.valueIterator()');
            });

            test('paria -> iterator', () => {
                const zig = tabulaCompile('map.paria()');

                expect(zig).toContain('map.iterator()');
            });
        });

        describe('extended', () => {
            test('accipeAut -> get with orelse', () => {
                const zig = tabulaCompile('map.accipeAut(key, 0)');

                expect(zig).toContain('map.get(key) orelse 0');
            });
        });
    });

    // =========================================================================
    // COPIA METHODS - Latin HashSet API for Zig
    // =========================================================================
    describe('copia methods - Latin HashSet API', () => {
        const copiaCompile = (expr: string) => compile(`fixum copia<numerus> set = novum copia()\n${expr}`);

        describe('core operations', () => {
            test('adde -> put with void value', () => {
                const zig = copiaCompile('set.adde(42)');

                expect(zig).toContain('set.put(alloc, 42, {})');
                expect(zig).toContain('catch @panic("OOM")');
            });

            test('habet -> contains', () => {
                const zig = copiaCompile('set.habet(42)');

                expect(zig).toContain('set.contains(42)');
            });

            test('dele -> remove', () => {
                const zig = copiaCompile('set.dele(42)');

                expect(zig).toContain('set.remove(42)');
            });

            test('longitudo -> count', () => {
                const zig = copiaCompile('set.longitudo()');

                expect(zig).toContain('set.count()');
            });

            test('vacua -> count == 0', () => {
                const zig = copiaCompile('set.vacua()');

                expect(zig).toContain('set.count() == 0');
            });

            test('purga -> clearRetainingCapacity', () => {
                const zig = copiaCompile('set.purga()');

                expect(zig).toContain('set.clearRetainingCapacity()');
            });
        });

        describe('iteration', () => {
            test('valores -> keyIterator (set values are keys)', () => {
                const zig = copiaCompile('set.valores()');

                expect(zig).toContain('set.keyIterator()');
            });
        });

        describe('set operations - unimplemented', () => {
            test('unio -> compileError', () => {
                const zig = copiaCompile('set.unio(other)');

                expect(zig).toContain('@compileError');
            });

            test('intersectio -> compileError', () => {
                const zig = copiaCompile('set.intersectio(other)');

                expect(zig).toContain('@compileError');
            });
        });
    });
});
