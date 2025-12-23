import { test, expect, describe } from 'bun:test';
import { tokenize } from '../tokenizer';
import { parse } from './index';

function parseCode(code: string) {
    const { tokens } = tokenize(code);

    return parse(tokens);
}

describe('parser', () => {
    describe('variable declarations', () => {
        test('varia without type', () => {
            const { program } = parseCode('varia nomen = "Marcus"');

            expect(program).not.toBeNull();
            expect(program!.body[0].type).toBe('VariableDeclaration');
            const decl = program!.body[0] as any;

            expect(decl.kind).toBe('varia');
            expect(decl.name.name).toBe('nomen');
            expect(decl.init.value).toBe('Marcus');
        });

        test('fixum with type annotation', () => {
            const { program } = parseCode('fixum numerus numerus = 42');
            const decl = program!.body[0] as any;

            expect(decl.kind).toBe('fixum');
            expect(decl.typeAnnotation.name).toBe('numerus');
            expect(decl.init.value).toBe(42);
        });

        test('generic type annotation', () => {
            const { program } = parseCode('fixum lista<numerus> lista = nihil');
            const decl = program!.body[0] as any;

            expect(decl.typeAnnotation.name).toBe('lista');
            expect(decl.typeAnnotation.typeParameters[0].name).toBe('numerus');
        });
    });

    describe('function declarations', () => {
        test('simple function with arrow return type', () => {
            const { program } = parseCode(`
        functio salve(textus nomen) -> textus {
          redde nomen
        }
      `);

            expect(program!.body[0].type).toBe('FunctionDeclaration');
            const fn = program!.body[0] as any;

            expect(fn.name.name).toBe('salve');
            expect(fn.params).toHaveLength(1);
            expect(fn.params[0].name.name).toBe('nomen');
            expect(fn.returnType.name).toBe('textus');
            expect(fn.async).toBe(false);
        });

        test('async function with futura', () => {
            const { program } = parseCode(`
        futura functio fetch(textus url) -> textus {
          redde data
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.async).toBe(true);
            expect(fn.returnType.name).toBe('textus');
        });

        test('function with preposition parameter', () => {
            const { program } = parseCode(`
        functio mitte(textus nuntium, ad textus recipientem) {
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

        test('ego self reference', () => {
            const { program } = parseCode('ego.nomen');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('MemberExpression');
            expect(expr.object.type).toBe('ThisExpression');
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
        test('cede', () => {
            const { program } = parseCode('cede fetch(url)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('AwaitExpression');
            expect(expr.argument.type).toBe('CallExpression');
        });
    });

    describe('new expression', () => {
        test('novum', () => {
            const { program } = parseCode('novum erratum(message)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('NewExpression');
            expect(expr.callee.name).toBe('erratum');
        });

        test('novum without parentheses', () => {
            const { program } = parseCode('novum persona');
            const expr = (program!.body[0] as any).expression;

            expect(expr.arguments).toHaveLength(0);
            expect(expr.withExpression).toBeUndefined();
        });

        test('novum with cum overrides', () => {
            const { program } = parseCode('novum persona cum { nomen: "Marcus" }');
            const expr = (program!.body[0] as any).expression;

            expect(expr.withExpression.type).toBe('ObjectExpression');
            expect(expr.withExpression.properties[0].key.name).toBe('nomen');
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
            const { program } = parseCode('fixum textus? x = nihil');
            const decl = program!.body[0] as any;

            expect(decl.typeAnnotation.nullable).toBe(true);
        });

        test('union type', () => {
            const { program } = parseCode('fixum textus | nihil x = nihil');
            const decl = program!.body[0] as any;

            expect(decl.typeAnnotation.name).toBe('union');
            expect(decl.typeAnnotation.union).toHaveLength(2);
        });
    });

    describe('genus declarations', () => {
        test('simple genus with one field', () => {
            const { program } = parseCode('genus persona { textus nomen }');

            expect(program!.body[0].type).toBe('GenusDeclaration');
            const genus = program!.body[0] as any;

            expect(genus.name.name).toBe('persona');
            expect(genus.fields).toHaveLength(1);
            expect(genus.fields[0].name.name).toBe('nomen');
            expect(genus.fields[0].fieldType.name).toBe('textus');
        });

        test('genus with two fields', () => {
            const { program } = parseCode('genus persona { textus nomen numerus aetas }');
            const genus = program!.body[0] as any;

            expect(genus.fields).toHaveLength(2);
            expect(genus.fields[1].name.name).toBe('aetas');
        });

        test('genus with default field value', () => {
            const { program } = parseCode('genus persona { numerus aetas: 0 }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].init.value).toBe(0);
        });

        test('genus with public field', () => {
            const { program } = parseCode('genus persona { publicus textus nomen }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].isPublic).toBe(true);
        });

        test('genus with static field', () => {
            const { program } = parseCode('genus math { generis numerus PI: 3 }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].isStatic).toBe(true);
        });

        test('genus with type parameter', () => {
            const { program } = parseCode('genus capsa<T> { T valor }');
            const genus = program!.body[0] as any;

            expect(genus.typeParameters).toHaveLength(1);
            expect(genus.typeParameters[0].name).toBe('T');
        });

        test('genus with implet', () => {
            const { program } = parseCode('genus cursor implet iterabilis { numerus index }');
            const genus = program!.body[0] as any;

            expect(genus.implements).toHaveLength(1);
            expect(genus.implements[0].name).toBe('iterabilis');
        });

        test('genus with method', () => {
            const { program } = parseCode('genus persona { functio saluta() { redde nihil } }');
            const genus = program!.body[0] as any;

            expect(genus.methods).toHaveLength(1);
            expect(genus.methods[0].name.name).toBe('saluta');
        });

        test('genus with creo constructor', () => {
            const { program } = parseCode('genus persona { functio creo(valores) { redde nihil } }');
            const genus = program!.body[0] as any;

            expect(genus.constructor).toBeDefined();
            expect(genus.methods).toHaveLength(0);
        });

        test('genus with computed field', () => {
            const { program } = parseCode('genus figura { numerus area => latus * altitudo }');
            const genus = program!.body[0] as any;

            expect(genus.computedFields).toHaveLength(1);
            expect(genus.computedFields[0].name.name).toBe('area');
        });
    });

    describe('pactum declarations', () => {
        test('pactum with methods', () => {
            const { program } = parseCode(`
                pactum iterabilis<T> {
                    functio sequens() -> T?
                    functio habet() -> bivalens
                }
            `);
            const pactum = program!.body[0] as any;

            expect(pactum.type).toBe('PactumDeclaration');
            expect(pactum.methods).toHaveLength(2);
            expect(pactum.methods[0].name.name).toBe('sequens');
            expect(pactum.typeParameters[0].name).toBe('T');
        });
    });
});
