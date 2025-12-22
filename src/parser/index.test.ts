import { test, expect, describe } from 'bun:test';
import { tokenize } from '../tokenizer';
import { parse } from './index';

function parseCode(code: string) {
    const { tokens } = tokenize(code);

    return parse(tokens);
}

describe('parser', () => {
    describe('variable declarations', () => {
        test('esto without type', () => {
            const { program } = parseCode('esto nomen = "Marcus"');

            expect(program).not.toBeNull();
            expect(program!.body[0].type).toBe('VariableDeclaration');
            const decl = program!.body[0] as any;

            expect(decl.kind).toBe('esto');
            expect(decl.name.name).toBe('nomen');
            expect(decl.init.value).toBe('Marcus');
        });

        test('fixum with type annotation', () => {
            const { program } = parseCode('fixum Numerus numerus = 42');
            const decl = program!.body[0] as any;

            expect(decl.kind).toBe('fixum');
            expect(decl.typeAnnotation.name).toBe('Numerus');
            expect(decl.init.value).toBe(42);
        });

        test('generic type annotation', () => {
            const { program } = parseCode('fixum Lista<Numerus> lista = nihil');
            const decl = program!.body[0] as any;

            expect(decl.typeAnnotation.name).toBe('Lista');
            expect(decl.typeAnnotation.typeParameters[0].name).toBe('Numerus');
        });
    });

    describe('function declarations', () => {
        test('simple function with arrow return type', () => {
            const { program } = parseCode(`
        functio salve(Textus nomen) -> Textus {
          redde nomen
        }
      `);

            expect(program!.body[0].type).toBe('FunctionDeclaration');
            const fn = program!.body[0] as any;

            expect(fn.name.name).toBe('salve');
            expect(fn.params).toHaveLength(1);
            expect(fn.params[0].name.name).toBe('nomen');
            expect(fn.returnType.name).toBe('Textus');
            expect(fn.async).toBe(false);
        });

        test('async function with futura', () => {
            const { program } = parseCode(`
        futura functio fetch(Textus url) -> Textus {
          redde data
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.async).toBe(true);
            expect(fn.returnType.name).toBe('Textus');
        });

        test('function with preposition parameter', () => {
            const { program } = parseCode(`
        functio mitte(Textus nuntium, ad Textus recipientem) {
          scribe(nuntium)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[1].preposition).toBe('ad');
            expect(fn.params[1].name.name).toBe('recipientem');
        });

        test('function without return type', () => {
            const { program } = parseCode(`
        functio greet(nomen) {
          scribe(nomen)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.name.name).toBe('greet');
            expect(fn.returnType).toBeUndefined();
        });
    });

    describe('if statements', () => {
        test('simple if', () => {
            const { program } = parseCode(`
        si verum {
          scribe("yes")
        }
      `);

            expect(program!.body[0].type).toBe('IfStatement');
            const stmt = program!.body[0] as any;

            expect(stmt.test.value).toBe(true);
        });

        test('if with aliter', () => {
            const { program } = parseCode(`
        si falsum {
          scribe("yes")
        }
        aliter {
          scribe("no")
        }
      `);
            const stmt = program!.body[0] as any;

            expect(stmt.alternate).not.toBeUndefined();
            expect(stmt.alternate.type).toBe('BlockStatement');
        });

        test('if with cape (catch)', () => {
            const { program } = parseCode(`
        si riskyCall() {
          process()
        }
        cape erratum {
          handleError()
        }
      `);
            const stmt = program!.body[0] as any;

            expect(stmt.catchClause).not.toBeUndefined();
            expect(stmt.catchClause.param.name).toBe('erratum');
        });
    });

    describe('loops', () => {
        test('while loop', () => {
            const { program } = parseCode(`
        dum verum {
          scribe("loop")
        }
      `);

            expect(program!.body[0].type).toBe('WhileStatement');
        });

        test('for...in loop', () => {
            const { program } = parseCode(`
        in lista pro item {
          scribe(item)
        }
      `);
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ForStatement');
            expect(stmt.kind).toBe('in');
            expect(stmt.variable.name).toBe('item');
        });

        test('for...ex loop', () => {
            const { program } = parseCode(`
        ex numeros pro numero {
          scribe(numero)
        }
      `);
            const stmt = program!.body[0] as any;

            expect(stmt.kind).toBe('ex');
        });
    });

    describe('expressions', () => {
        test('binary operators', () => {
            const { program } = parseCode('1 + 2 * 3');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.operator).toBe('+');
            expect(expr.right.operator).toBe('*');
        });

        test('comparison operators', () => {
            const { program } = parseCode('a > b');
            const expr = (program!.body[0] as any).expression;

            expect(expr.operator).toBe('>');
        });

        test('logical operators with Latin keywords', () => {
            const { program } = parseCode('a et b');
            const expr = (program!.body[0] as any).expression;

            expect(expr.operator).toBe('&&');
        });

        test('aut operator', () => {
            const { program } = parseCode('a aut b');
            const expr = (program!.body[0] as any).expression;

            expect(expr.operator).toBe('||');
        });

        test('negation with !', () => {
            const { program } = parseCode('!active');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('UnaryExpression');
            expect(expr.operator).toBe('!');
        });

        test('function call', () => {
            const { program } = parseCode('salve(nomen)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('CallExpression');
            expect(expr.callee.name).toBe('salve');
            expect(expr.arguments).toHaveLength(1);
        });

        test('member access', () => {
            const { program } = parseCode('usuario.nomen');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('MemberExpression');
            expect(expr.object.name).toBe('usuario');
            expect(expr.property.name).toBe('nomen');
        });

        test('chained member access', () => {
            const { program } = parseCode('a.b.c');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('MemberExpression');
            expect(expr.object.type).toBe('MemberExpression');
        });

        test('method call', () => {
            const { program } = parseCode('lista.filter(f)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('CallExpression');
            expect(expr.callee.type).toBe('MemberExpression');
        });

        test('empty array literal', () => {
            const { program } = parseCode('[]');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ArrayExpression');
            expect(expr.elements).toHaveLength(0);
        });

        test('array literal with elements', () => {
            const { program } = parseCode('[1, 2, 3]');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ArrayExpression');
            expect(expr.elements).toHaveLength(3);
            expect(expr.elements[0].value).toBe(1);
            expect(expr.elements[2].value).toBe(3);
        });

        test('nested array literal', () => {
            const { program } = parseCode('[[1, 2], [3, 4]]');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ArrayExpression');
            expect(expr.elements).toHaveLength(2);
            expect(expr.elements[0].type).toBe('ArrayExpression');
            expect(expr.elements[0].elements).toHaveLength(2);
        });
    });

    describe('arrow functions', () => {
        test('simple arrow function', () => {
            const { program } = parseCode('(x) => x');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ArrowFunctionExpression');
            expect(expr.params).toHaveLength(1);
            expect(expr.body.name).toBe('x');
        });

        test('arrow function with block', () => {
            const { program } = parseCode('(x) => { redde x }');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ArrowFunctionExpression');
            expect(expr.body.type).toBe('BlockStatement');
        });
    });

    describe('await expression', () => {
        test('exspecta', () => {
            const { program } = parseCode('exspecta fetch(url)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('AwaitExpression');
            expect(expr.argument.type).toBe('CallExpression');
        });
    });

    describe('new expression', () => {
        test('novum', () => {
            const { program } = parseCode('novum Erratum(message)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('NewExpression');
            expect(expr.callee.name).toBe('Erratum');
        });
    });

    describe('try/catch/finally', () => {
        test('tempta/cape', () => {
            const { program } = parseCode(`
        tempta {
          riskyCode()
        }
        cape error {
          handleError()
        }
      `);
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('TryStatement');
            expect(stmt.handler.param.name).toBe('error');
        });

        test('tempta/cape/demum', () => {
            const { program } = parseCode(`
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
            const stmt = program!.body[0] as any;

            expect(stmt.finalizer).not.toBeUndefined();
        });
    });

    describe('type annotations', () => {
        test('nullable type', () => {
            const { program } = parseCode('fixum Textus? x = nihil');
            const decl = program!.body[0] as any;

            expect(decl.typeAnnotation.nullable).toBe(true);
        });

        test('union type', () => {
            const { program } = parseCode('fixum Textus | Nihil x = nihil');
            const decl = program!.body[0] as any;

            expect(decl.typeAnnotation.name).toBe('union');
            expect(decl.typeAnnotation.union).toHaveLength(2);
        });
    });
});
