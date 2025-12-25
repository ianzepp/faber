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

        test('figendum parses as const-await binding', () => {
            const { program } = parseCode('figendum data = fetchData()');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariableDeclaration');
            expect(decl.kind).toBe('figendum');
            expect(decl.name.name).toBe('data');
            expect(decl.init.type).toBe('CallExpression');
        });

        test('variandum parses as let-await binding', () => {
            const { program } = parseCode('variandum result = getResult()');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariableDeclaration');
            expect(decl.kind).toBe('variandum');
            expect(decl.name.name).toBe('result');
            expect(decl.init.type).toBe('CallExpression');
        });

        test('figendum with type annotation', () => {
            const { program } = parseCode('figendum textus data = fetchData()');
            const decl = program!.body[0] as any;

            expect(decl.kind).toBe('figendum');
            expect(decl.typeAnnotation.name).toBe('textus');
        });

        test('variandum with type annotation', () => {
            const { program } = parseCode('variandum numerus count = getCount()');
            const decl = program!.body[0] as any;

            expect(decl.kind).toBe('variandum');
            expect(decl.typeAnnotation.name).toBe('numerus');
        });

        test('multiple figendum declarations', () => {
            const { program } = parseCode(`
                figendum a = fetchA()
                figendum b = fetchB()
            `);

            expect(program!.body).toHaveLength(2);
            expect((program!.body[0] as any).kind).toBe('figendum');
            expect((program!.body[1] as any).kind).toBe('figendum');
        });

        test('mixed async bindings', () => {
            const { program } = parseCode(`
                figendum config = loadConfig()
                variandum state = initState()
            `);

            expect((program!.body[0] as any).kind).toBe('figendum');
            expect((program!.body[1] as any).kind).toBe('variandum');
        });
    });

    describe('destructuring', () => {
        test('basic destructuring with fixum', () => {
            const { program } = parseCode('fixum { name, age } = person');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariableDeclaration');
            expect(decl.kind).toBe('fixum');
            expect(decl.name.type).toBe('ObjectPattern');
            expect(decl.name.properties).toHaveLength(2);
            expect(decl.name.properties[0].key.name).toBe('name');
            expect(decl.name.properties[1].key.name).toBe('age');
        });

        test('destructuring with rename', () => {
            const { program } = parseCode('fixum { name: userName } = person');
            const decl = program!.body[0] as any;

            expect(decl.name.properties[0].key.name).toBe('name');
            expect(decl.name.properties[0].value.name).toBe('userName');
        });

        test('ex destructuring with fixum', () => {
            const { program } = parseCode('ex response fixum { status, data }');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariableDeclaration');
            expect(decl.kind).toBe('fixum');
            expect(decl.name.type).toBe('ObjectPattern');
            expect(decl.name.properties).toHaveLength(2);
            expect(decl.name.properties[0].key.name).toBe('status');
            expect(decl.name.properties[1].key.name).toBe('data');
            expect(decl.init.name).toBe('response');
        });

        test('ex destructuring with varia', () => {
            const { program } = parseCode('ex config varia { host, port }');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariableDeclaration');
            expect(decl.kind).toBe('varia');
            expect(decl.name.type).toBe('ObjectPattern');
            expect(decl.init.name).toBe('config');
        });

        test('ex destructuring with rename', () => {
            const { program } = parseCode('ex response fixum { status: responseStatus, data: responseData }');
            const decl = program!.body[0] as any;

            expect(decl.name.properties[0].key.name).toBe('status');
            expect(decl.name.properties[0].value.name).toBe('responseStatus');
            expect(decl.name.properties[1].key.name).toBe('data');
            expect(decl.name.properties[1].value.name).toBe('responseData');
        });

        test('ex destructuring from function call', () => {
            const { program } = parseCode('ex getUser() fixum { name, email }');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariableDeclaration');
            expect(decl.init.type).toBe('CallExpression');
            expect(decl.init.callee.name).toBe('getUser');
        });

        test('ex destructuring from member access', () => {
            const { program } = parseCode('ex response.data fixum { items, count }');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariableDeclaration');
            expect(decl.init.type).toBe('MemberExpression');
            expect(decl.init.object.name).toBe('response');
            expect(decl.init.property.name).toBe('data');
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

        test('fit as return type alias for arrow', () => {
            const { program } = parseCode(`
        functio salve(nomen) fit textus {
          redde nomen
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.type).toBe('FunctionDeclaration');
            expect(fn.returnType.name).toBe('textus');
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

        test('function with de preposition (borrowed parameter)', () => {
            const { program } = parseCode(`
        functio read(de textus source) -> numerus {
          redde 0
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].preposition).toBe('de');
            expect(fn.params[0].typeAnnotation.name).toBe('textus');
            expect(fn.params[0].name.name).toBe('source');
        });

        test('function with in preposition (mutable parameter)', () => {
            const { program } = parseCode(`
        functio append(in lista<textus> items, textus value) {
          scribe(value)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].preposition).toBe('in');
            expect(fn.params[0].typeAnnotation.name).toBe('lista');
            expect(fn.params[0].name.name).toBe('items');
            expect(fn.params[1].preposition).toBeUndefined();
        });

        test('function with borrowed return type (fit de)', () => {
            const { program } = parseCode(`
        functio first(de lista<textus> items) fit de textus {
          redde items[0]
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].preposition).toBe('de');
            expect(fn.returnType.preposition).toBe('de');
            expect(fn.returnType.name).toBe('textus');
        });

        test('function with borrowed return type (arrow de)', () => {
            const { program } = parseCode(`
        functio first(de lista<textus> items) -> de textus {
          redde items[0]
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.returnType.preposition).toBe('de');
            expect(fn.returnType.name).toBe('textus');
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

        test('function with user-defined type parameter', () => {
            const { program } = parseCode(`
        functio distance(coordinate p1, coordinate p2) -> numerus {
          redde 0
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params).toHaveLength(2);
            expect(fn.params[0].typeAnnotation.name).toBe('coordinate');
            expect(fn.params[0].name.name).toBe('p1');
            expect(fn.params[1].typeAnnotation.name).toBe('coordinate');
            expect(fn.params[1].name.name).toBe('p2');
        });

        test('function with mixed builtin and user-defined types', () => {
            const { program } = parseCode(`
        functio process(textus name, config options) {
          scribe(name)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].typeAnnotation.name).toBe('textus');
            expect(fn.params[0].name.name).toBe('name');
            expect(fn.params[1].typeAnnotation.name).toBe('config');
            expect(fn.params[1].name.name).toBe('options');
        });

        test('function with user-defined generic type parameter', () => {
            const { program } = parseCode(`
        functio first(container<T> items) -> T {
          redde items[0]
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].typeAnnotation.name).toBe('container');
            expect(fn.params[0].typeAnnotation.typeParameters[0].name).toBe('T');
            expect(fn.params[0].name.name).toBe('items');
        });

        test('function with preposition and user-defined type', () => {
            const { program } = parseCode(`
        functio moveTo(ad coordinate destination) {
          scribe(destination)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].preposition).toBe('ad');
            expect(fn.params[0].typeAnnotation.name).toBe('coordinate');
            expect(fn.params[0].name.name).toBe('destination');
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
        de lista pro item {
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

    describe('ternary expressions', () => {
        test('symbolic ternary with ? :', () => {
            const { program } = parseCode('verum ? 1 : 0');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ConditionalExpression');
            expect(expr.test.value).toBe(true);
            expect(expr.consequent.value).toBe(1);
            expect(expr.alternate.value).toBe(0);
        });

        test('Latin ternary with sic secus', () => {
            const { program } = parseCode('verum sic 1 secus 0');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ConditionalExpression');
            expect(expr.test.value).toBe(true);
            expect(expr.consequent.value).toBe(1);
            expect(expr.alternate.value).toBe(0);
        });

        test('ternary with comparison condition', () => {
            const { program } = parseCode('x > 5 ? "big" : "small"');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ConditionalExpression');
            expect(expr.test.type).toBe('BinaryExpression');
            expect(expr.test.operator).toBe('>');
        });

        test('ternary with logical condition', () => {
            const { program } = parseCode('a et b sic 1 secus 0');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ConditionalExpression');
            expect(expr.test.type).toBe('BinaryExpression');
            expect(expr.test.operator).toBe('&&');
        });

        test('nested ternary (right-associative)', () => {
            const { program } = parseCode('a ? b ? c : d : e');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ConditionalExpression');
            expect(expr.consequent.type).toBe('ConditionalExpression');
            expect(expr.consequent.consequent.name).toBe('c');
            expect(expr.consequent.alternate.name).toBe('d');
        });

        test('nested Latin ternary', () => {
            const { program } = parseCode('a sic b sic c secus d secus e');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ConditionalExpression');
            expect(expr.consequent.type).toBe('ConditionalExpression');
        });

        test('ternary in assignment', () => {
            const { program } = parseCode('varia x = verum ? 1 : 0');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariableDeclaration');
            expect(decl.init.type).toBe('ConditionalExpression');
        });

        test('ternary with expressions', () => {
            const { program } = parseCode('x > 0 sic x * 2 secus 0 - x');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ConditionalExpression');
            expect(expr.consequent.type).toBe('BinaryExpression');
            expect(expr.consequent.operator).toBe('*');
            expect(expr.alternate.type).toBe('BinaryExpression');
            expect(expr.alternate.operator).toBe('-');
        });

        test('ternary with function calls', () => {
            const { program } = parseCode('active ? start() : stop()');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ConditionalExpression');
            expect(expr.consequent.type).toBe('CallExpression');
            expect(expr.alternate.type).toBe('CallExpression');
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

        test('genus with private field', () => {
            const { program } = parseCode('genus persona { privatus textus nomen }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].isPrivate).toBe(true);
        });

        test('genus field is public by default', () => {
            const { program } = parseCode('genus persona { textus nomen }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].isPrivate).toBe(false);
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
            const { program } = parseCode('genus iterator implet iterabilis { numerus index }');
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

        test('genus with nexum reactive field', () => {
            const { program } = parseCode('genus counter { nexum numerus count: 0 }');
            const genus = program!.body[0] as any;

            expect(genus.fields).toHaveLength(1);
            expect(genus.fields[0].name.name).toBe('count');
            expect(genus.fields[0].isReactive).toBe(true);
        });

        test('genus with nexum and privatus modifiers', () => {
            const { program } = parseCode('genus widget { privatus nexum numerus value: 0 }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].isPrivate).toBe(true);
            expect(genus.fields[0].isReactive).toBe(true);
        });

        test('genus with nexum and generis modifiers', () => {
            const { program } = parseCode('genus app { generis nexum numerus instances: 0 }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].isStatic).toBe(true);
            expect(genus.fields[0].isReactive).toBe(true);
        });

        test('genus with mixed regular and nexum fields', () => {
            const { program } = parseCode(`
                genus widget {
                    textus id
                    nexum numerus count: 0
                    bivalens active: verum
                }
            `);
            const genus = program!.body[0] as any;

            expect(genus.fields).toHaveLength(3);
            expect(genus.fields[0].isReactive).toBe(false);
            expect(genus.fields[1].isReactive).toBe(true);
            expect(genus.fields[2].isReactive).toBe(false);
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

    describe('enum declarations (ordo)', () => {
        test('simple enum', () => {
            const { program } = parseCode('ordo color { rubrum, viridis, caeruleum }');
            const enumDecl = program!.body[0] as any;

            expect(enumDecl.type).toBe('EnumDeclaration');
            expect(enumDecl.name.name).toBe('color');
            expect(enumDecl.members).toHaveLength(3);
            expect(enumDecl.members[0].name.name).toBe('rubrum');
            expect(enumDecl.members[1].name.name).toBe('viridis');
            expect(enumDecl.members[2].name.name).toBe('caeruleum');
        });

        test('enum with numeric values', () => {
            const { program } = parseCode('ordo status { pendens = 0, actum = 1, finitum = 2 }');
            const enumDecl = program!.body[0] as any;

            expect(enumDecl.type).toBe('EnumDeclaration');
            expect(enumDecl.name.name).toBe('status');
            expect(enumDecl.members).toHaveLength(3);
            expect(enumDecl.members[0].value.value).toBe(0);
            expect(enumDecl.members[1].value.value).toBe(1);
            expect(enumDecl.members[2].value.value).toBe(2);
        });

        test('enum with string values', () => {
            const { program } = parseCode('ordo direction { north = "N", south = "S" }');
            const enumDecl = program!.body[0] as any;

            expect(enumDecl.type).toBe('EnumDeclaration');
            expect(enumDecl.members[0].value.value).toBe('N');
            expect(enumDecl.members[1].value.value).toBe('S');
        });

        test('enum with trailing comma', () => {
            const { program } = parseCode('ordo color { rubrum, viridis, }');
            const enumDecl = program!.body[0] as any;

            expect(enumDecl.type).toBe('EnumDeclaration');
            expect(enumDecl.members).toHaveLength(2);
        });

        test('enum with mixed values', () => {
            const { program } = parseCode('ordo mixed { a, b = 5, c }');
            const enumDecl = program!.body[0] as any;

            expect(enumDecl.members[0].value).toBeUndefined();
            expect(enumDecl.members[1].value.value).toBe(5);
            expect(enumDecl.members[2].value).toBeUndefined();
        });

        test('single member enum', () => {
            const { program } = parseCode('ordo singleton { unum }');
            const enumDecl = program!.body[0] as any;

            expect(enumDecl.members).toHaveLength(1);
            expect(enumDecl.members[0].name.name).toBe('unum');
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

        test('sin as else-if alias', () => {
            const { program } = parseCode(`
                si x < 0 { redde "negative" }
                sin x == 0 { redde "zero" }
                aliter { redde "positive" }
            `);
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.alternate.type).toBe('IfStatement');
            expect(stmt.alternate.alternate.type).toBe('BlockStatement');
        });

        test('secus as else alias', () => {
            const { program } = parseCode(`
                si x < 0 { redde "negative" }
                sin x == 0 { redde "zero" }
                secus { redde "positive" }
            `);
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.alternate.type).toBe('IfStatement');
            expect(stmt.alternate.alternate.type).toBe('BlockStatement');
        });

        test('est as strict equality operator', () => {
            const { program } = parseCode('si x est nihil { scribe "null" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.test.type).toBe('BinaryExpression');
            expect(stmt.test.operator).toBe('===');
            expect(stmt.test.left.name).toBe('x');
            expect(stmt.test.right.value).toBe(null);
        });

        test('non est as strict inequality operator', () => {
            const { program } = parseCode('si x non est nihil { scribe "not null" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.test.type).toBe('BinaryExpression');
            expect(stmt.test.operator).toBe('!==');
            expect(stmt.test.left.name).toBe('x');
            expect(stmt.test.right.value).toBe(null);
        });

        test('negativum as unary check', () => {
            const { program } = parseCode('si negativum n { scribe "negative" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.test.type).toBe('UnaryExpression');
            expect(stmt.test.operator).toBe('negativum');
        });

        test('positivum as unary check', () => {
            const { program } = parseCode('si positivum n { scribe "positive" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.test.type).toBe('UnaryExpression');
            expect(stmt.test.operator).toBe('positivum');
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

        test('de...pro with ergo', () => {
            const { program } = parseCode('de obj pro key ergo scribe key');
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

        test('ergo with rumpe', () => {
            const { program } = parseCode('si done ergo rumpe');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body[0].type).toBe('BreakStatement');
        });

        test('ergo with perge', () => {
            const { program } = parseCode('si skip ergo perge');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IfStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body[0].type).toBe('ContinueStatement');
        });
    });

    describe('break and continue (rumpe/perge)', () => {
        test('rumpe parses as BreakStatement', () => {
            const { program } = parseCode('dum verum { rumpe }');
            const loop = program!.body[0] as any;

            expect(loop.type).toBe('WhileStatement');
            expect(loop.body.body[0].type).toBe('BreakStatement');
        });

        test('perge parses as ContinueStatement', () => {
            const { program } = parseCode('dum verum { perge }');
            const loop = program!.body[0] as any;

            expect(loop.type).toBe('WhileStatement');
            expect(loop.body.body[0].type).toBe('ContinueStatement');
        });

        test('rumpe in for loop', () => {
            const { program } = parseCode('ex items pro item { rumpe }');
            const loop = program!.body[0] as any;

            expect(loop.type).toBe('ForStatement');
            expect(loop.body.body[0].type).toBe('BreakStatement');
        });

        test('perge in for loop', () => {
            const { program } = parseCode('ex items pro item { perge }');
            const loop = program!.body[0] as any;

            expect(loop.type).toBe('ForStatement');
            expect(loop.body.body[0].type).toBe('ContinueStatement');
        });

        test('rumpe inside conditional in loop', () => {
            const { program } = parseCode('dum verum { si found { rumpe } }');
            const loop = program!.body[0] as any;
            const ifStmt = loop.body.body[0];

            expect(ifStmt.type).toBe('IfStatement');
            expect(ifStmt.consequent.body[0].type).toBe('BreakStatement');
        });

        test('perge inside conditional in loop', () => {
            const { program } = parseCode('dum verum { si skip { perge } }');
            const loop = program!.body[0] as any;
            const ifStmt = loop.body.body[0];

            expect(ifStmt.type).toBe('IfStatement');
            expect(ifStmt.consequent.body[0].type).toBe('ContinueStatement');
        });

        test('multiple rumpe and perge in same loop', () => {
            const { program } = parseCode(`
                dum verum {
                    si skip { perge }
                    si done { rumpe }
                }
            `);
            const loop = program!.body[0] as any;

            expect(loop.body.body[0].consequent.body[0].type).toBe('ContinueStatement');
            expect(loop.body.body[1].consequent.body[0].type).toBe('BreakStatement');
        });
    });

    describe('loop verb conjugation (fit/fiet)', () => {
        test('ex...fit parses as sync loop', () => {
            const { program } = parseCode('ex items fit item { scribe item }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ForStatement');
            expect(stmt.kind).toBe('ex');
            expect(stmt.async).toBe(false);
            expect(stmt.variable.name).toBe('item');
        });

        test('ex...fiet parses as async loop', () => {
            const { program } = parseCode('ex stream fiet chunk { scribe chunk }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ForStatement');
            expect(stmt.kind).toBe('ex');
            expect(stmt.async).toBe(true);
            expect(stmt.variable.name).toBe('chunk');
        });

        test('ex...pro remains sync (backward compatibility)', () => {
            const { program } = parseCode('ex items pro item { scribe item }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ForStatement');
            expect(stmt.async).toBe(false);
        });

        test('de...fit parses as sync loop', () => {
            const { program } = parseCode('de obj fit key { scribe key }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ForStatement');
            expect(stmt.kind).toBe('in');
            expect(stmt.async).toBe(false);
        });

        test('ex...fit with ergo one-liner', () => {
            const { program } = parseCode('ex items fit item ergo scribe item');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ForStatement');
            expect(stmt.async).toBe(false);
            expect(stmt.body.body[0].type).toBe('ScribeStatement');
        });

        test('ex...fiet with ergo one-liner', () => {
            const { program } = parseCode('ex stream fiet chunk ergo process(chunk)');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('ForStatement');
            expect(stmt.async).toBe(true);
            expect(stmt.body.body[0].expression.type).toBe('CallExpression');
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

        test('futura with fit contradicts (sync vs async)', () => {
            const { errors } = parseCode('futura functio f() fit textus { redde "x" }');

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain('contradicts');
        });

        test('cursor with fit contradicts (single vs generator)', () => {
            const { errors } = parseCode('cursor functio f() fit textus { redde "x" }');

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain('contradicts');
        });

        test('futura with fiunt contradicts (sync generator vs async)', () => {
            const { errors } = parseCode('futura functio f() fiunt numerus { redde 1 }');

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain('contradicts');
        });

        test('cursor with fiet contradicts (single async vs generator)', () => {
            const { errors } = parseCode('cursor functio f() fiet textus { redde "x" }');

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain('contradicts');
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

    describe('fac block and lambda', () => {
        describe('fac block statement', () => {
            test('simple fac block', () => {
                const { program } = parseCode('fac { varia x = 1 }');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('FacBlockStatement');
                expect(stmt.body.type).toBe('BlockStatement');
                expect(stmt.catchClause).toBeUndefined();
            });

            test('fac block with cape', () => {
                const { program } = parseCode('fac { x() } cape e { y() }');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('FacBlockStatement');
                expect(stmt.catchClause).not.toBeUndefined();
                expect(stmt.catchClause.param.name).toBe('e');
            });
        });

        describe('pro expression (lambda)', () => {
            test('single param lambda', () => {
                const { program } = parseCode('pro x redde x * 2');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(1);
                expect(expr.params[0].name).toBe('x');
                expect(expr.body.type).toBe('BinaryExpression');
                expect(expr.async).toBe(false);
            });

            test('multi param lambda', () => {
                const { program } = parseCode('pro a, b redde a + b');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(2);
                expect(expr.params[0].name).toBe('a');
                expect(expr.params[1].name).toBe('b');
            });

            test('zero param lambda', () => {
                const { program } = parseCode('pro redde 42');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(0);
                expect(expr.body.value).toBe(42);
            });

            test('lambda in variable declaration', () => {
                const { program } = parseCode('fixum double = pro x redde x * 2');
                const decl = program!.body[0] as any;

                expect(decl.type).toBe('VariableDeclaration');
                expect(decl.init.type).toBe('LambdaExpression');
            });

            test('nested lambdas', () => {
                const { program } = parseCode('pro x redde pro y redde x + y');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.body.type).toBe('LambdaExpression');
                expect(expr.body.body.type).toBe('BinaryExpression');
            });

            test('single param block lambda', () => {
                const { program } = parseCode('pro x { redde x * 2 }');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(1);
                expect(expr.params[0].name).toBe('x');
                expect(expr.body.type).toBe('BlockStatement');
            });

            test('multi param block lambda', () => {
                const { program } = parseCode('pro a, b { redde a + b }');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(2);
                expect(expr.body.type).toBe('BlockStatement');
            });

            test('zero param block lambda', () => {
                const { program } = parseCode('pro { scribe "hello" }');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(0);
                expect(expr.body.type).toBe('BlockStatement');
            });

            test('block lambda with multiple statements', () => {
                const { program } = parseCode('pro x {\nfixum y = x * 2\nredde y\n}');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.body.type).toBe('BlockStatement');
                expect(expr.body.body).toHaveLength(2);
            });
        });

        describe('pro expression with : shorthand', () => {
            test('single param lambda with :', () => {
                const { program } = parseCode('pro x: x * 2');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(1);
                expect(expr.params[0].name).toBe('x');
                expect(expr.body.type).toBe('BinaryExpression');
            });

            test('multi param lambda with :', () => {
                const { program } = parseCode('pro a, b: a + b');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(2);
                expect(expr.params[0].name).toBe('a');
                expect(expr.params[1].name).toBe('b');
            });

            test('zero param lambda with :', () => {
                const { program } = parseCode('pro: 42');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(0);
                expect(expr.body.value).toBe(42);
            });

            test('nested lambdas with :', () => {
                const { program } = parseCode('pro x: pro y: x + y');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.body.type).toBe('LambdaExpression');
                expect(expr.body.body.type).toBe('BinaryExpression');
            });

            test('lambda with : in variable declaration', () => {
                const { program } = parseCode('fixum double = pro x: x * 2');
                const decl = program!.body[0] as any;

                expect(decl.type).toBe('VariableDeclaration');
                expect(decl.init.type).toBe('LambdaExpression');
            });

            test('mixed : and redde in nested lambdas', () => {
                const { program } = parseCode('pro x: pro y redde x + y');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.body.type).toBe('LambdaExpression');
            });
        });
    });

    describe('import declarations', () => {
        test('string path source', () => {
            const { program } = parseCode('ex "norma/tempus" importa nunc, dormi');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('ImportDeclaration');
            expect(decl.source).toBe('norma/tempus');
            expect(decl.specifiers).toHaveLength(2);
            expect(decl.specifiers[0].name).toBe('nunc');
            expect(decl.specifiers[1].name).toBe('dormi');
        });

        test('wildcard import', () => {
            const { program } = parseCode('ex "norma/tempus" importa *');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('ImportDeclaration');
            expect(decl.wildcard).toBe(true);
            expect(decl.specifiers).toHaveLength(0);
        });

        test('single import', () => {
            const { program } = parseCode('ex "norma/tempus" importa nunc');
            const decl = program!.body[0] as any;

            expect(decl.specifiers).toHaveLength(1);
            expect(decl.specifiers[0].name).toBe('nunc');
        });

        test('import with constants', () => {
            const { program } = parseCode('ex "norma/tempus" importa SECUNDUM, MINUTUM, HORA');
            const decl = program!.body[0] as any;

            expect(decl.specifiers).toHaveLength(3);
            expect(decl.specifiers[0].name).toBe('SECUNDUM');
            expect(decl.specifiers[1].name).toBe('MINUTUM');
            expect(decl.specifiers[2].name).toBe('HORA');
        });
    });

    describe('semicolon as optional statement separator', () => {
        test('multiple statements on one line', () => {
            const { program } = parseCode('fixum x = 1; fixum y = 2');

            expect(program!.body).toHaveLength(2);
            expect((program!.body[0] as any).name.name).toBe('x');
            expect((program!.body[1] as any).name.name).toBe('y');
        });

        test('three statements on one line', () => {
            const { program } = parseCode('fixum a = 1; fixum b = 2; fixum c = 3');

            expect(program!.body).toHaveLength(3);
        });

        test('trailing semicolon is valid', () => {
            const { program } = parseCode('fixum x = 1;');

            expect(program!.body).toHaveLength(1);
            expect((program!.body[0] as any).name.name).toBe('x');
        });

        test('multiple semicolons collapse', () => {
            const { program } = parseCode('fixum x = 1;; fixum y = 2');

            expect(program!.body).toHaveLength(2);
        });

        test('semicolons in block lambda', () => {
            const { program } = parseCode('pro x { fixum y = x * 2; redde y }');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('LambdaExpression');
            expect(expr.body.type).toBe('BlockStatement');
            expect(expr.body.body).toHaveLength(2);
        });

        test('block lambda single line with semicolon', () => {
            const { program } = parseCode('items.mappata(pro x { fixum y = transform(x); redde y })');
            const expr = (program!.body[0] as any).expression;
            const lambda = expr.arguments[0];

            expect(lambda.type).toBe('LambdaExpression');
            expect(lambda.body.body).toHaveLength(2);
        });

        test('semicolons in function body', () => {
            const { program } = parseCode('functio f() { fixum a = 1; fixum b = 2; redde a + b }');
            const fn = program!.body[0] as any;

            expect(fn.body.body).toHaveLength(3);
        });

        test('semicolons in if block', () => {
            const { program } = parseCode('si verum { a(); b() }');
            const stmt = program!.body[0] as any;

            expect(stmt.consequent.body).toHaveLength(2);
        });

        test('semicolons in while block', () => {
            const { program } = parseCode('dum verum { a(); b(); rumpe }');
            const stmt = program!.body[0] as any;

            expect(stmt.body.body).toHaveLength(3);
        });

        test('semicolons with newlines mixed', () => {
            const { program } = parseCode('fixum a = 1;\nfixum b = 2; fixum c = 3\nfixum d = 4');

            expect(program!.body).toHaveLength(4);
        });

        test('leading semicolons ignored', () => {
            const { program } = parseCode('; fixum x = 1');

            expect(program!.body).toHaveLength(1);
            expect((program!.body[0] as any).name.name).toBe('x');
        });

        test('only semicolons produces empty program', () => {
            const { program } = parseCode(';;;');

            expect(program!.body).toHaveLength(0);
        });

        test('semicolon in empty block is valid', () => {
            const { program } = parseCode('{ ; }');
            const block = program!.body[0] as any;

            expect(block.type).toBe('BlockStatement');
            expect(block.body).toHaveLength(0);
        });

        test('event handler with semicolon', () => {
            const { program } = parseCode('button.onClick(pro { setup(); doThing() })');
            const expr = (program!.body[0] as any).expression;
            const lambda = expr.arguments[0];

            expect(lambda.body.body).toHaveLength(2);
        });

        test('style preference - semicolons at line ends', () => {
            const { program } = parseCode('fixum nomen = "Marcus";\nfixum aetas = 30;');

            expect(program!.body).toHaveLength(2);
        });
    });
});
