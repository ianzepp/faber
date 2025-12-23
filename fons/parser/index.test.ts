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

    describe('nulla/nonnulla operators', () => {
        test('nulla as unary check', () => {
            const { program } = parseCode('si nulla x { scribe "empty" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.test.type).toBe('UnaryExpression');
            expect(stmt.test.operator).toBe('nulla');
            expect(stmt.test.argument.name).toBe('x');
        });

        test('nonnulla as unary check', () => {
            const { program } = parseCode('si nonnulla items { scribe "has items" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.test.type).toBe('UnaryExpression');
            expect(stmt.test.operator).toBe('nonnulla');
            expect(stmt.test.argument.name).toBe('items');
        });

        test('nulla on array literal', () => {
            const { program } = parseCode('nulla []');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('UnaryExpression');
            expect(expr.operator).toBe('nulla');
            expect(expr.argument.type).toBe('ArrayExpression');
        });

        test('nonnulla on object literal', () => {
            const { program } = parseCode('nonnulla { a: 1 }');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('UnaryExpression');
            expect(expr.operator).toBe('nonnulla');
            expect(expr.argument.type).toBe('ObjectExpression');
        });

        test('nulla on member expression', () => {
            const { program } = parseCode('nulla usuario.nomen');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('UnaryExpression');
            expect(expr.operator).toBe('nulla');
            expect(expr.argument.type).toBe('MemberExpression');
        });

        test('nonnulla on function call', () => {
            const { program } = parseCode('nonnulla getData()');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('UnaryExpression');
            expect(expr.operator).toBe('nonnulla');
            expect(expr.argument.type).toBe('CallExpression');
        });

        test('nested nulla expressions', () => {
            const { program } = parseCode('nulla nulla x');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('UnaryExpression');
            expect(expr.operator).toBe('nulla');
            expect(expr.argument.type).toBe('UnaryExpression');
            expect(expr.argument.operator).toBe('nulla');
        });

        test('nulla in binary expression', () => {
            const { program } = parseCode('a et nonnulla b');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.operator).toBe('&&');
            expect(expr.right.type).toBe('UnaryExpression');
            expect(expr.right.operator).toBe('nonnulla');
        });
    });

    describe('iace (throw) statements', () => {
        test('throw string literal', () => {
            const { program } = parseCode('iace "Error message"');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ThrowStatement');
            expect(stmt.argument.type).toBe('Literal');
            expect(stmt.argument.value).toBe('Error message');
        });

        test('throw new Error', () => {
            const { program } = parseCode('iace novum Error("message")');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ThrowStatement');
            expect(stmt.argument.type).toBe('NewExpression');
            expect(stmt.argument.callee.name).toBe('Error');
        });

        test('throw variable', () => {
            const { program } = parseCode('iace error');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ThrowStatement');
            expect(stmt.argument.type).toBe('Identifier');
            expect(stmt.argument.name).toBe('error');
        });

        test('throw in block', () => {
            const { program } = parseCode(`{
                iace "error"
            }`);
            const block = program!.body[0] as any;

            expect(block.type).toBe('BlockStatement');
            expect(block.body[0].type).toBe('ThrowStatement');
        });

        test('throw in switch case', () => {
            const { program } = parseCode(`
                elige x {
                    si 1 { iace "invalid" }
                }
            `);
            const switchStmt = program!.body[0] as any;

            expect(switchStmt.type).toBe('SwitchStatement');
            expect(switchStmt.cases[0].consequent.body[0].type).toBe('ThrowStatement');
        });

        test('throw in if statement', () => {
            const { program } = parseCode(`
                si x < 0 {
                    iace "negative"
                }
            `);
            const ifStmt = program!.body[0] as any;

            expect(ifStmt.type).toBe('IfStatement');
            expect(ifStmt.consequent.body[0].type).toBe('ThrowStatement');
        });

        test('throw expression result', () => {
            const { program } = parseCode('iace x + y');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ThrowStatement');
            expect(stmt.argument.type).toBe('BinaryExpression');
        });
    });

    describe('ergo one-liners', () => {
        test('si with ergo', () => {
            const { program } = parseCode('si x > 5 ergo scribe "big"');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.test.type).toBe('BinaryExpression');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body).toHaveLength(1);
            expect(stmt.consequent.body[0].type).toBe('ScribeStatement');
        });

        test('si with ergo and aliter', () => {
            const { program } = parseCode('si x > 5 ergo scribe "big" aliter scribe "small"');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body).toHaveLength(1);
            expect(stmt.alternate).toBeDefined();
            expect(stmt.alternate.type).toBe('BlockStatement');
            expect(stmt.alternate.body[0].type).toBe('ScribeStatement');
        });

        test('dum with ergo', () => {
            const { program } = parseCode('dum x > 0 ergo x = x - 1');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('WhileStatement');
            expect(stmt.body.type).toBe('BlockStatement');
            expect(stmt.body.body).toHaveLength(1);
            expect(stmt.body.body[0].expression.type).toBe('AssignmentExpression');
        });

        test('ex...pro with ergo', () => {
            const { program } = parseCode('ex items pro item ergo scribe item');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ForStatement');
            expect(stmt.kind).toBe('ex');
            expect(stmt.body.type).toBe('BlockStatement');
            expect(stmt.body.body).toHaveLength(1);
            expect(stmt.body.body[0].type).toBe('ScribeStatement');
        });

        test('in...pro with ergo', () => {
            const { program } = parseCode('in obj pro key ergo scribe key');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ForStatement');
            expect(stmt.kind).toBe('in');
            expect(stmt.body.type).toBe('BlockStatement');
            expect(stmt.body.body).toHaveLength(1);
        });

        test('ergo with function call', () => {
            const { program } = parseCode('si active ergo process()');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body[0].expression.type).toBe('CallExpression');
        });

        test('ergo with assignment', () => {
            const { program } = parseCode('si valid ergo result = 1');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body[0].expression.type).toBe('AssignmentExpression');
        });

        test('ergo with return', () => {
            const { program } = parseCode('si done ergo redde result');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body[0].type).toBe('ReturnStatement');
        });
    });

    describe('edge cases - empty constructs', () => {
        test('empty block', () => {
            const { program } = parseCode('{}');

            expect(program!.body[0].type).toBe('BlockStatement');
            expect((program!.body[0] as any).body).toHaveLength(0);
        });

        test('empty function body', () => {
            const { program } = parseCode('functio f() {}');
            const fn = program!.body[0] as any;

            expect(fn.type).toBe('FunctionDeclaration');
            expect(fn.body.body).toHaveLength(0);
        });

        test('function with no parameters', () => {
            const { program } = parseCode('functio salve() { redde nihil }');
            const fn = program!.body[0] as any;

            expect(fn.params).toHaveLength(0);
        });

        test('empty array literal', () => {
            const { program } = parseCode('fixum arr = []');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('ArrayExpression');
            expect(decl.init.elements).toHaveLength(0);
        });

        test('empty object literal', () => {
            const { program } = parseCode('fixum obj = {}');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('ObjectExpression');
            expect(decl.init.properties).toHaveLength(0);
        });

        test('empty string literal', () => {
            const { program } = parseCode('fixum s = ""');
            const decl = program!.body[0] as any;

            expect(decl.init.value).toBe('');
        });

        test('zero number', () => {
            const { program } = parseCode('fixum x = 0');
            const decl = program!.body[0] as any;

            expect(decl.init.value).toBe(0);
        });

        test('zero decimal', () => {
            const { program } = parseCode('fixum x = 0.0');
            const decl = program!.body[0] as any;

            expect(decl.init.value).toBe(0.0);
        });

        test('empty genus', () => {
            const { program } = parseCode('genus Empty { }');
            const genus = program!.body[0] as any;

            expect(genus.type).toBe('GenusDeclaration');
            expect(genus.fields || []).toHaveLength(0);
            expect(genus.methods || []).toHaveLength(0);
        });

        test('if with empty blocks', () => {
            const { program } = parseCode('si verum { } aliter { }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.consequent.body).toHaveLength(0);
            expect(stmt.alternate.body).toHaveLength(0);
        });
    });

    describe('edge cases - deeply nested structures', () => {
        test('deeply nested parentheses (5 levels)', () => {
            const { program } = parseCode('((((1 + 2))))');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.left.value).toBe(1);
            expect(expr.right.value).toBe(2);
        });

        test('deeply nested expressions (6 levels)', () => {
            const { program } = parseCode('1 + 2 * 3 - 4 / 5 + 6');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
        });

        test('deeply nested member access (5 levels)', () => {
            const { program } = parseCode('a.b.c.d.e');
            let expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('MemberExpression');
            expect(expr.property.name).toBe('e');

            expr = expr.object;
            expect(expr.type).toBe('MemberExpression');
            expect(expr.property.name).toBe('d');
        });

        test('nested function calls (4 levels)', () => {
            const { program } = parseCode('f(g(h(i())))');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('CallExpression');
            expect(expr.callee.name).toBe('f');
            expect(expr.arguments[0].type).toBe('CallExpression');
            expect(expr.arguments[0].callee.name).toBe('g');
        });

        test('nested arrays (4 levels)', () => {
            const { program } = parseCode('[[[[1]]]]');
            const decl = (program!.body[0] as any).expression;

            expect(decl.type).toBe('ArrayExpression');
            expect(decl.elements[0].type).toBe('ArrayExpression');
            expect(decl.elements[0].elements[0].type).toBe('ArrayExpression');
        });

        test('nested objects (3 levels)', () => {
            const { program } = parseCode('fixum x = { a: { b: { c: 1 } } }');
            const obj = (program!.body[0] as any).init;

            expect(obj.type).toBe('ObjectExpression');
            expect(obj.properties[0].value.type).toBe('ObjectExpression');
        });

        test('nested if statements (3 levels)', () => {
            const { program } = parseCode(`
                si a {
                    si b {
                        si c {
                            scribe "nested"
                        }
                    }
                }
            `);
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.consequent.body[0].type).toBe('IfStatement');
            expect(stmt.consequent.body[0].consequent.body[0].type).toBe('IfStatement');
        });
    });

    describe('edge cases - operator combinations', () => {
        test('mixed logical operators', () => {
            const { program } = parseCode('a et b aut c');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
        });

        test('logical operators with comparison', () => {
            const { program } = parseCode('x > 5 et y < 10');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.operator).toBe('&&');
            expect(expr.left.type).toBe('BinaryExpression');
            expect(expr.right.type).toBe('BinaryExpression');
        });

        test('negation of logical expression', () => {
            const { program } = parseCode('!(a et b)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('UnaryExpression');
            expect(expr.operator).toBe('!');
            expect(expr.argument.type).toBe('BinaryExpression');
        });

        test('double negation', () => {
            const { program } = parseCode('non non x');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('UnaryExpression');
            expect(expr.operator).toBe('!');
            expect(expr.argument.type).toBe('UnaryExpression');
        });

        test('comparison chain', () => {
            const { program } = parseCode('a < b < c');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.left.type).toBe('BinaryExpression');
        });

        test('mixed arithmetic operators', () => {
            const { program } = parseCode('x + y * z - w / v % u');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
        });

        test('arithmetic with logical', () => {
            const { program } = parseCode('x + y > 5 et a - b < 3');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.operator).toBe('&&');
        });

        test('nulla with logical operators', () => {
            const { program } = parseCode('nulla x aut nonnulla y');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.left.type).toBe('UnaryExpression');
            expect(expr.right.type).toBe('UnaryExpression');
        });
    });

    describe('error recovery - missing/malformed syntax', () => {
        test('missing closing brace in function', () => {
            const { errors } = parseCode('functio f() { scribe "x"');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing closing paren in function call', () => {
            const { errors } = parseCode('f(a, b');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing closing bracket in array', () => {
            const { errors } = parseCode('fixum arr = [1, 2, 3');

            expect(errors.length).toBeGreaterThan(0);
        });

        test.todo('missing operator in expression', () => {
            // PARSER GAP: Should error on two literals without operator
            const { errors } = parseCode('fixum x = 1 2');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('incomplete assignment', () => {
            const { errors } = parseCode('fixum x =');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('incomplete binary expression', () => {
            const { errors } = parseCode('1 +');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing function body', () => {
            const { errors } = parseCode('functio f()');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing condition in if', () => {
            const { errors } = parseCode('si { scribe "x" }');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing while condition', () => {
            const { errors } = parseCode('dum { scribe "x" }');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('incomplete for loop', () => {
            const { errors } = parseCode('ex items pro');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('invalid statement start', () => {
            const { errors } = parseCode(') + 1');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('unexpected keyword sequence', () => {
            const { errors } = parseCode('functio si while');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing object property value', () => {
            const { errors } = parseCode('fixum x = { a: }');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing variable name', () => {
            const { errors } = parseCode('fixum = 5');

            expect(errors.length).toBeGreaterThan(0);
        });

        test.todo('incomplete type annotation without initializer', () => {
            // PARSER GAP: fixum (const) requires initialization, should error
            const { errors } = parseCode('fixum numerus x');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing arrow in function return type', () => {
            const { errors } = parseCode('functio f() textus { }');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('incomplete genus declaration', () => {
            const { errors } = parseCode('genus');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing throw argument', () => {
            const { errors } = parseCode('iace');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing return value', () => {
            const { program, errors } = parseCode('redde');

            // This might be valid (return undefined/void), so just check it parses
            expect(program).not.toBeNull();
        });

        test('incomplete member access', () => {
            const { errors } = parseCode('usuario.');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('double assignment operator', () => {
            const { errors } = parseCode('x == = 5');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing generic type parameter', () => {
            const { errors } = parseCode('fixum lista<> x = nihil');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('unclosed generic brackets', () => {
            const { errors } = parseCode('fixum lista<numerus x = nihil');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('invalid array element', () => {
            const { errors } = parseCode('[1, , 3]');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing function parameter name', () => {
            const { errors } = parseCode('functio f(numerus) {}');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('invalid destructuring', () => {
            const { errors } = parseCode('fixum {, a} = obj');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('incomplete switch case', () => {
            const { errors } = parseCode('elige x { si }');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('missing catch parameter', () => {
            const { errors } = parseCode('tempta {} cape {}');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('incomplete assert', () => {
            const { errors } = parseCode('adfirma');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('invalid operator sequence', () => {
            const { errors } = parseCode('x + + y');

            expect(errors.length).toBeGreaterThan(0);
        });
    });

    describe('scribe with multiple arguments', () => {
        test('scribe with two arguments', () => {
            const { program } = parseCode('scribe "Name:", name');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ScribeStatement');
            expect(stmt.arguments).toHaveLength(2);
            expect(stmt.arguments[0].value).toBe('Name:');
            expect(stmt.arguments[1].name).toBe('name');
        });

        test('scribe with multiple arguments', () => {
            const { program } = parseCode('scribe "Name:", name, "Age:", age');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ScribeStatement');
            expect(stmt.arguments).toHaveLength(4);
            expect(stmt.arguments[0].value).toBe('Name:');
            expect(stmt.arguments[1].name).toBe('name');
            expect(stmt.arguments[2].value).toBe('Age:');
            expect(stmt.arguments[3].name).toBe('age');
        });

        test('scribe with expressions', () => {
            const { program } = parseCode('scribe x + y, a * b');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ScribeStatement');
            expect(stmt.arguments).toHaveLength(2);
            expect(stmt.arguments[0].type).toBe('BinaryExpression');
            expect(stmt.arguments[1].type).toBe('BinaryExpression');
        });

        test('scribe with function calls', () => {
            const { program } = parseCode('scribe getName(), getAge()');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ScribeStatement');
            expect(stmt.arguments).toHaveLength(2);
            expect(stmt.arguments[0].type).toBe('CallExpression');
            expect(stmt.arguments[1].type).toBe('CallExpression');
        });

        test('scribe with member access', () => {
            const { program } = parseCode('scribe user.name, user.age');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ScribeStatement');
            expect(stmt.arguments).toHaveLength(2);
            expect(stmt.arguments[0].type).toBe('MemberExpression');
            expect(stmt.arguments[1].type).toBe('MemberExpression');
        });
    });
});
