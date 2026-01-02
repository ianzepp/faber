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
            expect(program!.body[0]!.type).toBe('VariaDeclaration');
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
            const { program } = parseCode('fixum numerus[] lista = nihil');
            const decl = program!.body[0] as any;

            expect(decl.typeAnnotation.name).toBe('lista');
            expect(decl.typeAnnotation.typeParameters[0].name).toBe('numerus');
        });

        test('figendum parses as const-await binding', () => {
            const { program } = parseCode('figendum data = fetchData()');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariaDeclaration');
            expect(decl.kind).toBe('figendum');
            expect(decl.name.name).toBe('data');
            expect(decl.init.type).toBe('CallExpression');
        });

        test('variandum parses as let-await binding', () => {
            const { program } = parseCode('variandum result = getResult()');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariaDeclaration');
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
        // Object destructuring uses ex-prefix brace-less syntax
        test('ex object destructuring with fixum', () => {
            const { program } = parseCode('ex person fixum name, age');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('DestructureDeclaration');
            expect(decl.kind).toBe('fixum');
            expect(decl.specifiers).toHaveLength(2);
            expect(decl.specifiers[0].imported.name).toBe('name');
            expect(decl.specifiers[0].local.name).toBe('name');
            expect(decl.specifiers[1].imported.name).toBe('age');
            expect(decl.source.name).toBe('person');
        });

        test('ex object destructuring with ut alias', () => {
            const { program } = parseCode('ex person fixum name ut userName');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('DestructureDeclaration');
            expect(decl.specifiers[0].imported.name).toBe('name');
            expect(decl.specifiers[0].local.name).toBe('userName');
        });

        test('ex object destructuring with varia', () => {
            const { program } = parseCode('ex config varia host, port');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('DestructureDeclaration');
            expect(decl.kind).toBe('varia');
            expect(decl.specifiers).toHaveLength(2);
            expect(decl.source.name).toBe('config');
        });

        test('ex object destructuring with multiple ut aliases', () => {
            const { program } = parseCode('ex response fixum status ut responseStatus, data ut responseData');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('DestructureDeclaration');
            expect(decl.specifiers[0].imported.name).toBe('status');
            expect(decl.specifiers[0].local.name).toBe('responseStatus');
            expect(decl.specifiers[1].imported.name).toBe('data');
            expect(decl.specifiers[1].local.name).toBe('responseData');
        });

        test('ex object destructuring from function call', () => {
            const { program } = parseCode('ex getUser() fixum name, email');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('DestructureDeclaration');
            expect(decl.source.type).toBe('CallExpression');
            expect(decl.source.callee.name).toBe('getUser');
        });

        test('ex object destructuring from member access', () => {
            const { program } = parseCode('ex response.data fixum items, count');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('DestructureDeclaration');
            expect(decl.source.type).toBe('MemberExpression');
            expect(decl.source.object.name).toBe('response');
            expect(decl.source.property.name).toBe('data');
        });

        test('ex object destructuring with ceteri rest', () => {
            const { program } = parseCode('ex person fixum name, ceteri rest');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('DestructureDeclaration');
            expect(decl.specifiers).toHaveLength(2);
            expect(decl.specifiers[0].imported.name).toBe('name');
            expect(decl.specifiers[0].rest).toBeFalsy();
            expect(decl.specifiers[1].imported.name).toBe('rest');
            expect(decl.specifiers[1].rest).toBe(true);
        });

        test('ex async destructuring with figendum', () => {
            const { program } = parseCode('ex fetchData() figendum result');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('DestructureDeclaration');
            expect(decl.kind).toBe('figendum');
            expect(decl.specifiers[0].imported.name).toBe('result');
        });

        // Array destructuring is now supported
        test('basic array destructuring with fixum', () => {
            const { program } = parseCode('fixum [a, b, c] = coords');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariaDeclaration');
            expect(decl.kind).toBe('fixum');
            expect(decl.name.type).toBe('ArrayPattern');
            expect(decl.name.elements).toHaveLength(3);
            expect(decl.name.elements[0].name.name).toBe('a');
            expect(decl.name.elements[1].name.name).toBe('b');
            expect(decl.name.elements[2].name.name).toBe('c');
        });

        test('array destructuring with rest pattern', () => {
            const { program } = parseCode('fixum [first, ceteri rest] = items');
            const decl = program!.body[0] as any;

            expect(decl.name.type).toBe('ArrayPattern');
            expect(decl.name.elements).toHaveLength(2);
            expect(decl.name.elements[0].name.name).toBe('first');
            expect(decl.name.elements[0].rest).toBeFalsy();
            expect(decl.name.elements[1].name.name).toBe('rest');
            expect(decl.name.elements[1].rest).toBe(true);
        });

        test('array destructuring with skip (underscore)', () => {
            const { program } = parseCode('fixum [_, second, _] = data');
            const decl = program!.body[0] as any;

            expect(decl.name.type).toBe('ArrayPattern');
            expect(decl.name.elements).toHaveLength(3);
            expect(decl.name.elements[0].skip).toBe(true);
            expect(decl.name.elements[1].name.name).toBe('second');
            expect(decl.name.elements[1].skip).toBeFalsy();
            expect(decl.name.elements[2].skip).toBe(true);
        });

        test('ex array destructuring with fixum', () => {
            const { program } = parseCode('ex coords fixum [x, y, z]');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariaDeclaration');
            expect(decl.kind).toBe('fixum');
            expect(decl.name.type).toBe('ArrayPattern');
            expect(decl.name.elements).toHaveLength(3);
            expect(decl.init.name).toBe('coords');
        });

        test('ex array destructuring with varia', () => {
            const { program } = parseCode('ex coords varia [x, y]');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariaDeclaration');
            expect(decl.kind).toBe('varia');
            expect(decl.name.type).toBe('ArrayPattern');
        });

        test('ex array destructuring from function call', () => {
            const { program } = parseCode('ex divide(17, 5) fixum [quotient, remainder]');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariaDeclaration');
            expect(decl.init.type).toBe('CallExpression');
            expect(decl.init.callee.name).toBe('divide');
            expect(decl.name.type).toBe('ArrayPattern');
            expect(decl.name.elements).toHaveLength(2);
        });

        test('Fail when using computed property in destructure', () => {
            const { errors } = parseCode('fixum { [key]: value } = obj');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('Fail when ceteri appears without variable name', () => {
            const { errors } = parseCode('fixum { ceteri } = obj');

            expect(errors.length).toBeGreaterThan(0);
        });

        // WHY: ceteri in the middle is valid - it collects remaining props at that point
        // The parser doesn't enforce ceteri being last; semantics would handle it
    });

    describe('function declarations', () => {
        test('simple function with arrow return type', () => {
            const { program } = parseCode(`
        functio salve(textus nomen) -> textus {
          redde nomen
        }
      `);

            expect(program!.body[0]!.type).toBe('FunctioDeclaration');
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

            expect(fn.type).toBe('FunctioDeclaration');
            expect(fn.returnType.name).toBe('textus');
        });

        test('async function with @ futura annotation', () => {
            const { program } = parseCode(`
        @ futura
        functio fetch(textus url) -> textus {
          redde data
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.annotations).toBeDefined();
            expect(fn.annotations[0].name).toBe('futura');
            expect(fn.returnType.name).toBe('textus');
        });

        test('function with preposition parameter', () => {
            const { program } = parseCode(`
        functio mitte(textus nuntium, de textus source) {
          scribe(nuntium)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[1].preposition).toBe('de');
            expect(fn.params[1].name.name).toBe('source');
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
        functio append(in textus[] items, textus value) {
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
        functio first(de textus[] items) fit de textus {
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
        functio first(de textus[] items) -> de textus {
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

        test('function with user-defined array type parameter', () => {
            const { program } = parseCode(`
        functio process(de Point[] points) {
          scribe(points)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].preposition).toBe('de');
            expect(fn.params[0].typeAnnotation.name).toBe('lista');
            expect(fn.params[0].typeAnnotation.arrayShorthand).toBe(true);
            expect(fn.params[0].typeAnnotation.typeParameters[0].name).toBe('Point');
            expect(fn.params[0].name.name).toBe('points');
        });

        test('function with preposition and user-defined type', () => {
            const { program } = parseCode(`
        functio moveTo(in coordinate destination) {
          scribe(destination)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].preposition).toBe('in');
            expect(fn.params[0].typeAnnotation.name).toBe('coordinate');
            expect(fn.params[0].name.name).toBe('destination');
        });

        // Invalid function declaration patterns
        test('Fail when using TS-style param annotation (name: type)', () => {
            const { errors } = parseCode('functio f(x: textus) {}');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('Fail when using TS-style return type with colon', () => {
            const { errors } = parseCode('functio f(): textus {}');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('Fail when using trailing comma in params', () => {
            const { errors } = parseCode('functio f(a, b,) {}');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('Allow keyword as parameter name (contextual keywords)', () => {
            // WHY: Keywords are valid identifiers in unambiguous contexts like parameter names
            const { program, errors } = parseCode('functio f(ad) {}');

            expect(errors.length).toBe(0);
            const fn = program!.body[0] as any;
            expect(fn.params[0].name.name).toBe('ad');
        });

        test('function with dual parameter naming (ut alias)', () => {
            const { program } = parseCode(`
        functio greet(textus location ut loc) {
          scribe(loc)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].name.name).toBe('location');
            expect(fn.params[0].alias.name).toBe('loc');
            expect(fn.params[0].typeAnnotation.name).toBe('textus');
        });

        test('function with dual naming and preposition', () => {
            const { program, errors } = parseCode(`
        functio process(de lista<Point> points ut p1, in lista<Point> targets ut p2) {
          scribe(p1)
        }
      `);
            expect(errors).toHaveLength(0);
            const fn = program!.body[0] as any;

            expect(fn.params[0].preposition).toBe('de');
            expect(fn.params[0].typeAnnotation.name).toBe('lista');
            expect(fn.params[0].name.name).toBe('points');
            expect(fn.params[0].alias.name).toBe('p1');

            expect(fn.params[1].preposition).toBe('in');
            expect(fn.params[1].typeAnnotation.name).toBe('lista');
            expect(fn.params[1].name.name).toBe('targets');
            expect(fn.params[1].alias.name).toBe('p2');
        });

        test('function with mixed aliased and non-aliased params', () => {
            const { program, errors } = parseCode(`
        functio move(de lista<Point> from ut source, in lista<Point> to) {
          scribe(source)
        }
      `);
            expect(errors).toHaveLength(0);
            const fn = program!.body[0] as any;

            expect(fn.params[0].name.name).toBe('from');
            expect(fn.params[0].alias.name).toBe('source');
            expect(fn.params[1].name.name).toBe('to');
            expect(fn.params[1].alias).toBeUndefined();
        });

        test('function param without alias has undefined alias', () => {
            const { program } = parseCode(`
        functio greet(textus name) {
          scribe(name)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].name.name).toBe('name');
            expect(fn.params[0].alias).toBeUndefined();
        });

        test('function with default parameter value (vel)', () => {
            const { program, errors } = parseCode(`
        functio greet(textus name vel "World") {
          scribe(name)
        }
      `);
            expect(errors).toHaveLength(0);
            const fn = program!.body[0] as any;

            expect(fn.params[0].name.name).toBe('name');
            expect(fn.params[0].typeAnnotation.name).toBe('textus');
            expect(fn.params[0].defaultValue).toBeDefined();
            expect(fn.params[0].defaultValue.type).toBe('Literal');
            expect(fn.params[0].defaultValue.value).toBe('World');
        });

        test('function with numeric default value', () => {
            const { program, errors } = parseCode(`
        functio count(numerus n vel 10) {
          redde n
        }
      `);
            expect(errors).toHaveLength(0);
            const fn = program!.body[0] as any;

            expect(fn.params[0].name.name).toBe('n');
            expect(fn.params[0].defaultValue.value).toBe(10);
        });

        test('function with dual naming and default value', () => {
            const { program, errors } = parseCode(`
        functio greet(textus location ut loc vel "Roma") {
          scribe(loc)
        }
      `);
            expect(errors).toHaveLength(0);
            const fn = program!.body[0] as any;

            expect(fn.params[0].name.name).toBe('location');
            expect(fn.params[0].alias.name).toBe('loc');
            expect(fn.params[0].defaultValue.value).toBe('Roma');
        });

        test('function with mixed default and non-default params', () => {
            const { program, errors } = parseCode(`
        functio greet(textus greeting, textus name vel "World") {
          scribe(greeting + name)
        }
      `);
            expect(errors).toHaveLength(0);
            const fn = program!.body[0] as any;

            expect(fn.params[0].defaultValue).toBeUndefined();
            expect(fn.params[1].defaultValue.value).toBe('World');
        });

        test('function param without default has undefined defaultValue', () => {
            const { program } = parseCode(`
        functio greet(textus name) {
          scribe(name)
        }
      `);
            const fn = program!.body[0] as any;

            expect(fn.params[0].defaultValue).toBeUndefined();
        });
    });

    describe('if statements', () => {
        test('simple if', () => {
            const { program } = parseCode(`
        si verum {
          scribe("yes")
        }
      `);

            expect(program!.body[0]!.type).toBe('SiStatement');
            const stmt = program!.body[0] as any;

            expect(stmt.test.value).toBe(true);
        });

        test('if with secus', () => {
            const { program } = parseCode(`
        si falsum {
          scribe("yes")
        }
        secus {
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

            expect(program!.body[0]!.type).toBe('DumStatement');
        });

        test('for...in loop', () => {
            const { program } = parseCode(`
        de lista pro item {
          scribe(item)
        }
      `);
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
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

        test('bitwise AND', () => {
            const { program } = parseCode('a & b');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.operator).toBe('&');
        });

        test('bitwise OR', () => {
            const { program } = parseCode('a | b');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.operator).toBe('|');
        });

        test('bitwise XOR', () => {
            const { program } = parseCode('a ^ b');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.operator).toBe('^');
        });

        test('bitwise NOT', () => {
            const { program } = parseCode('~a');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('UnaryExpression');
            expect(expr.operator).toBe('~');
        });

        test('left shift', () => {
            const { program } = parseCode('a << 2');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.operator).toBe('<<');
        });

        test('right shift', () => {
            const { program } = parseCode('a >> 2');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('BinaryExpression');
            expect(expr.operator).toBe('>>');
        });

        test('bitwise precedence: & binds tighter than |', () => {
            const { program } = parseCode('a | b & c');
            const expr = (program!.body[0] as any).expression;

            // Should parse as: a | (b & c)
            expect(expr.operator).toBe('|');
            expect(expr.right.operator).toBe('&');
        });

        test('bitwise precedence: << binds tighter than &', () => {
            const { program } = parseCode('a & b << 2');
            const expr = (program!.body[0] as any).expression;

            // Should parse as: a & (b << 2)
            expect(expr.operator).toBe('&');
            expect(expr.right.operator).toBe('<<');
        });

        test('bitwise precedence: bitwise binds tighter than comparison', () => {
            const { program } = parseCode('flags & MASK == 0');
            const expr = (program!.body[0] as any).expression;

            // Should parse as: (flags & MASK) == 0 (unlike C!)
            expect(expr.operator).toBe('==');
            expect(expr.left.operator).toBe('&');
        });

        test('negation with non', () => {
            // WHY: Prefix ! removed to make room for non-null assertion (!.)
            //      Use 'non' for logical negation
            const { program } = parseCode('non active');
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
            expect(expr.object.type).toBe('EgoExpression');
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

            expect(decl.type).toBe('VariaDeclaration');
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

    describe('optional chaining and non-null assertion', () => {
        test('optional member access with ?.', () => {
            const { program } = parseCode('user?.name');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('MemberExpression');
            expect(expr.object.name).toBe('user');
            expect(expr.property.name).toBe('name');
            expect(expr.optional).toBe(true);
            expect(expr.computed).toBe(false);
        });

        test('optional computed access with ?[', () => {
            const { program } = parseCode('items?[0]');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('MemberExpression');
            expect(expr.object.name).toBe('items');
            expect(expr.property.value).toBe(0);
            expect(expr.optional).toBe(true);
            expect(expr.computed).toBe(true);
        });

        test('optional call with ?(', () => {
            const { program } = parseCode('callback?(x)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('CallExpression');
            expect(expr.callee.name).toBe('callback');
            expect(expr.optional).toBe(true);
            expect(expr.arguments[0].name).toBe('x');
        });

        test('non-null member access with !.', () => {
            const { program } = parseCode('user!.name');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('MemberExpression');
            expect(expr.object.name).toBe('user');
            expect(expr.property.name).toBe('name');
            expect(expr.nonNull).toBe(true);
            expect(expr.computed).toBe(false);
        });

        test('non-null computed access with ![', () => {
            const { program } = parseCode('items![0]');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('MemberExpression');
            expect(expr.object.name).toBe('items');
            expect(expr.property.value).toBe(0);
            expect(expr.nonNull).toBe(true);
            expect(expr.computed).toBe(true);
        });

        test('non-null call with !(', () => {
            const { program } = parseCode('callback!(x)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('CallExpression');
            expect(expr.callee.name).toBe('callback');
            expect(expr.nonNull).toBe(true);
            expect(expr.arguments[0].name).toBe('x');
        });

        test('chained optional access', () => {
            const { program } = parseCode('user?.address?.city');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('MemberExpression');
            expect(expr.optional).toBe(true);
            expect(expr.property.name).toBe('city');
            expect(expr.object.type).toBe('MemberExpression');
            expect(expr.object.optional).toBe(true);
        });

        test('mixed optional and regular access', () => {
            const { program } = parseCode('user?.address.city');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('MemberExpression');
            expect(expr.optional).toBe(undefined);
            expect(expr.property.name).toBe('city');
            expect(expr.object.type).toBe('MemberExpression');
            expect(expr.object.optional).toBe(true);
        });

        test('disambiguation: ? followed by expression is ternary', () => {
            const { program } = parseCode('x ? y : z');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('ConditionalExpression');
            expect(expr.test.name).toBe('x');
            expect(expr.consequent.name).toBe('y');
            expect(expr.alternate.name).toBe('z');
        });
    });

    describe('await expression', () => {
        test('cede', () => {
            const { program } = parseCode('cede fetch(url)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('CedeExpression');
            expect(expr.argument.type).toBe('CallExpression');
        });
    });

    describe('new expression', () => {
        test('novum', () => {
            const { program } = parseCode('novum erratum(message)');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('NovumExpression');
            expect(expr.callee.name).toBe('erratum');
        });

        test('novum without parentheses', () => {
            const { program } = parseCode('novum persona');
            const expr = (program!.body[0] as any).expression;

            expect(expr.arguments).toHaveLength(0);
            expect(expr.withExpression).toBeUndefined();
        });

        test('novum with property overrides', () => {
            const { program } = parseCode('novum persona { nomen: "Marcus" }');
            const expr = (program!.body[0] as any).expression;

            expect(expr.withExpression.type).toBe('ObjectExpression');
            expect(expr.withExpression.properties[0].key.name).toBe('nomen');
        });

        test('novum de variable', () => {
            const { program } = parseCode('novum persona de props');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('NovumExpression');
            expect(expr.callee.name).toBe('persona');
            expect(expr.withExpression.type).toBe('Identifier');
            expect(expr.withExpression.name).toBe('props');
        });

        test('novum de function call', () => {
            const { program } = parseCode('novum user de fetchProps()');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('NovumExpression');
            expect(expr.withExpression.type).toBe('CallExpression');
            expect(expr.withExpression.callee.name).toBe('fetchProps');
        });
    });

    describe('finge expression', () => {
        test('finge unit variant', () => {
            const { program } = parseCode('finge Active');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('FingeExpression');
            expect(expr.variant.name).toBe('Active');
            expect(expr.fields).toBeUndefined();
            expect(expr.discretioType).toBeUndefined();
        });

        test('finge unit variant with qua', () => {
            const { program } = parseCode('finge Active qua Status');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('FingeExpression');
            expect(expr.variant.name).toBe('Active');
            expect(expr.discretioType.name).toBe('Status');
        });

        test('finge payload variant', () => {
            const { program } = parseCode('finge Click { x: 10, y: 20 }');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('FingeExpression');
            expect(expr.variant.name).toBe('Click');
            expect(expr.fields.type).toBe('ObjectExpression');
            expect(expr.fields.properties).toHaveLength(2);
        });

        test('finge payload variant with qua', () => {
            const { program } = parseCode('finge Click { x: 10, y: 20 } qua Event');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('FingeExpression');
            expect(expr.variant.name).toBe('Click');
            expect(expr.fields.type).toBe('ObjectExpression');
            expect(expr.discretioType.name).toBe('Event');
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

            expect(stmt.type).toBe('TemptaStatement');
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

        test('union type with unio<A, B>', () => {
            const { program } = parseCode('fixum unio<textus, nihil> x = nihil');
            const decl = program!.body[0] as any;

            expect(decl.typeAnnotation.name).toBe('union');
            expect(decl.typeAnnotation.union).toHaveLength(2);
            expect(decl.typeAnnotation.union[0].name).toBe('textus');
            expect(decl.typeAnnotation.union[1].name).toBe('nihil');
        });

        // Invalid type annotation patterns
        test('Fail when using TS-style variable annotation (name: type)', () => {
            const { errors } = parseCode('fixum nomen: textus = "x"');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('Fail when using TS-style type alias with colon', () => {
            const { errors } = parseCode('typus ID: textus');

            expect(errors.length).toBeGreaterThan(0);
        });
    });

    describe('genus declarations', () => {
        test('simple genus with one field', () => {
            const { program } = parseCode('genus persona { textus nomen }');

            expect(program!.body[0]!.type).toBe('GenusDeclaration');
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
            const { program } = parseCode('genus persona { @ privatum\n textus nomen }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].annotations).toHaveLength(1);
            expect(genus.fields[0].annotations[0].name).toBe('privatum');
        });

        test('genus field is public by default', () => {
            const { program } = parseCode('genus persona { textus nomen }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].visibility).toBe('public');
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

        test('genus with nexum reactive field', () => {
            const { program } = parseCode('genus counter { nexum numerus count: 0 }');
            const genus = program!.body[0] as any;

            expect(genus.fields).toHaveLength(1);
            expect(genus.fields[0].name.name).toBe('count');
            expect(genus.fields[0].isReactive).toBe(true);
        });

        test('genus with nexum and privatum annotation', () => {
            const { program } = parseCode('genus widget { @ privatum\n nexum numerus value: 0 }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].annotations[0].name).toBe('privatum');
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

        test('genus with sub (inheritance)', () => {
            const { program } = parseCode('genus employee sub persona { textus title }');
            const genus = program!.body[0] as any;

            expect(genus.type).toBe('GenusDeclaration');
            expect(genus.name.name).toBe('employee');
            expect(genus.extends.name).toBe('persona');
            expect(genus.fields).toHaveLength(1);
        });

        test('genus with sub and implet', () => {
            const { program } = parseCode('genus employee sub persona implet worker { textus title }');
            const genus = program!.body[0] as any;

            expect(genus.extends.name).toBe('persona');
            expect(genus.implements).toHaveLength(1);
            expect(genus.implements[0].name).toBe('worker');
        });

        test('abstractum genus with annotation', () => {
            const { program } = parseCode('@ abstractum\n genus animal { @ abstracta\n functio speak() -> textus }');
            const genus = program!.body[0] as any;

            expect(genus.type).toBe('GenusDeclaration');
            expect(genus.annotations[0].name).toBe('abstractum');
            expect(genus.name.name).toBe('animal');
            expect(genus.methods).toHaveLength(1);
            expect(genus.methods[0].annotations[0].name).toBe('abstracta');
            expect(genus.methods[0].isAbstract).toBe(true);
            expect(genus.methods[0].body).toBeUndefined();
        });

        test('genus with protected field', () => {
            const { program } = parseCode('genus animal { @ protectum\n textus species }');
            const genus = program!.body[0] as any;

            expect(genus.fields[0].annotations[0].name).toBe('protectum');
        });

        test('genus with protected method', () => {
            const { program } = parseCode('genus animal { @ protecta\n functio internal() { redde nihil } }');
            const genus = program!.body[0] as any;

            expect(genus.methods[0].annotations[0].name).toBe('protecta');
        });

        test('genus extending abstract class', () => {
            const { program } = parseCode(`
                @ abstractum
                genus animal { @ abstracta\n functio speak() -> textus }
                genus dog sub animal { functio speak() -> textus { redde "woof" } }
            `);
            const abstractGenus = program!.body[0] as any;
            const concreteGenus = program!.body[1] as any;

            expect(abstractGenus.annotations[0].name).toBe('abstractum');
            expect(concreteGenus.annotations).toBeUndefined();
            expect(concreteGenus.extends.name).toBe('animal');
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

            expect(enumDecl.type).toBe('OrdoDeclaration');
            expect(enumDecl.name.name).toBe('color');
            expect(enumDecl.members).toHaveLength(3);
            expect(enumDecl.members[0].name.name).toBe('rubrum');
            expect(enumDecl.members[1].name.name).toBe('viridis');
            expect(enumDecl.members[2].name.name).toBe('caeruleum');
        });

        test('enum with numeric values', () => {
            const { program } = parseCode('ordo status { pendens = 0, actum = 1, finitum = 2 }');
            const enumDecl = program!.body[0] as any;

            expect(enumDecl.type).toBe('OrdoDeclaration');
            expect(enumDecl.name.name).toBe('status');
            expect(enumDecl.members).toHaveLength(3);
            expect(enumDecl.members[0].value.value).toBe(0);
            expect(enumDecl.members[1].value.value).toBe(1);
            expect(enumDecl.members[2].value.value).toBe(2);
        });

        test('enum with string values', () => {
            const { program } = parseCode('ordo direction { north = "N", south = "S" }');
            const enumDecl = program!.body[0] as any;

            expect(enumDecl.type).toBe('OrdoDeclaration');
            expect(enumDecl.members[0].value.value).toBe('N');
            expect(enumDecl.members[1].value.value).toBe('S');
        });

        test('enum with trailing comma', () => {
            const { program } = parseCode('ordo color { rubrum, viridis, }');
            const enumDecl = program!.body[0] as any;

            expect(enumDecl.type).toBe('OrdoDeclaration');
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

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.test.type).toBe('UnaryExpression');
            expect(stmt.test.operator).toBe('nulla');
            expect(stmt.test.argument.name).toBe('x');
        });

        test('nonnulla as unary check', () => {
            const { program } = parseCode('si nonnulla items { scribe "has items" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
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

    describe('qua (type cast) operator', () => {
        test('simple type cast', () => {
            const { program } = parseCode('data qua textus');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('QuaExpression');
            expect(expr.expression.name).toBe('data');
            expect(expr.targetType.name).toBe('textus');
        });

        test('cast member expression', () => {
            const { program } = parseCode('response.body qua objectum');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('QuaExpression');
            expect(expr.expression.type).toBe('MemberExpression');
            expect(expr.expression.object.name).toBe('response');
            expect(expr.expression.property.name).toBe('body');
            expect(expr.targetType.name).toBe('objectum');
        });

        test('cast call expression', () => {
            const { program } = parseCode('getData() qua textus');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('QuaExpression');
            expect(expr.expression.type).toBe('CallExpression');
            expect(expr.targetType.name).toBe('textus');
        });

        test('chained casts (left-associative)', () => {
            const { program } = parseCode('x qua A qua B');
            const expr = (program!.body[0] as any).expression;

            // Should parse as (x qua A) qua B
            expect(expr.type).toBe('QuaExpression');
            expect(expr.targetType.name).toBe('B');
            expect(expr.expression.type).toBe('QuaExpression');
            expect(expr.expression.targetType.name).toBe('A');
            expect(expr.expression.expression.name).toBe('x');
        });

        test('cast with generic type', () => {
            const { program } = parseCode('items qua textus[]');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('QuaExpression');
            expect(expr.targetType.name).toBe('lista');
            expect(expr.targetType.typeParameters[0].name).toBe('textus');
        });

        test('cast with nullable type', () => {
            const { program } = parseCode('value qua textus?');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('QuaExpression');
            expect(expr.targetType.name).toBe('textus');
            expect(expr.targetType.nullable).toBe(true);
        });

        test('cast precedence: unary binds looser', () => {
            // -x qua numerus should parse as -(x qua numerus)
            const { program } = parseCode('-x qua numerus');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('UnaryExpression');
            expect(expr.operator).toBe('-');
            expect(expr.argument.type).toBe('QuaExpression');
            expect(expr.argument.expression.name).toBe('x');
            expect(expr.argument.targetType.name).toBe('numerus');
        });

        test('cast precedence: member access binds tighter', () => {
            // x.y qua T should parse as (x.y) qua T
            const { program } = parseCode('x.y qua textus');
            const expr = (program!.body[0] as any).expression;

            expect(expr.type).toBe('QuaExpression');
            expect(expr.expression.type).toBe('MemberExpression');
        });

        test('cast in binary expression', () => {
            // a + b qua numerus should parse as (a + b) qua numerus
            // because qua has lower precedence than +
            const { program } = parseCode('a + b qua numerus');
            const expr = (program!.body[0] as any).expression;

            // WHY: qua has higher precedence than + in our grammar,
            // so this parses as a + (b qua numerus)
            expect(expr.type).toBe('BinaryExpression');
            expect(expr.operator).toBe('+');
            expect(expr.right.type).toBe('QuaExpression');
        });
    });

    describe('iace (throw) statements', () => {
        test('throw string literal', () => {
            const { program } = parseCode('iace "Error message"');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IaceStatement');
            expect(stmt.argument.type).toBe('Literal');
            expect(stmt.argument.value).toBe('Error message');
        });

        test('throw new Error', () => {
            const { program } = parseCode('iace novum Error("message")');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IaceStatement');
            expect(stmt.argument.type).toBe('NovumExpression');
            expect(stmt.argument.callee.name).toBe('Error');
        });

        test('throw variable', () => {
            const { program } = parseCode('iace error');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IaceStatement');
            expect(stmt.argument.type).toBe('Identifier');
            expect(stmt.argument.name).toBe('error');
        });

        test('throw in block', () => {
            const { program } = parseCode(`{
                iace "error"
            }`);
            const block = program!.body[0] as any;

            expect(block.type).toBe('BlockStatement');
            expect(block.body[0]!.type).toBe('IaceStatement');
        });

        test('throw in switch case', () => {
            const { program } = parseCode(`
                elige x {
                    casu 1 { iace "invalid" }
                }
            `);
            const switchStmt = program!.body[0] as any;

            expect(switchStmt.type).toBe('EligeStatement');
            expect(switchStmt.cases[0].consequent.body[0]!.type).toBe('IaceStatement');
        });

        test('throw in if statement', () => {
            const { program } = parseCode(`
                si x < 0 {
                    iace "negative"
                }
            `);
            const ifStmt = program!.body[0] as any;

            expect(ifStmt.type).toBe('SiStatement');
            expect(ifStmt.consequent.body[0]!.type).toBe('IaceStatement');
        });

        test('throw expression result', () => {
            const { program } = parseCode('iace x + y');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IaceStatement');
            expect(stmt.argument.type).toBe('BinaryExpression');
        });
    });

    describe('ergo one-liners', () => {
        test('si with ergo', () => {
            const { program } = parseCode('si x > 5 ergo scribe "big"');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.test.type).toBe('BinaryExpression');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body).toHaveLength(1);
            expect(stmt.consequent.body[0]!.type).toBe('ScribeStatement');
        });

        test('si with ergo and secus', () => {
            const { program } = parseCode('si x > 5 ergo scribe "big" secus scribe "small"');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body).toHaveLength(1);
            expect(stmt.alternate).toBeDefined();
            expect(stmt.alternate.type).toBe('BlockStatement');
            expect(stmt.alternate.body[0]!.type).toBe('ScribeStatement');
        });

        test('sin as else-if alias', () => {
            const { program } = parseCode(`
                si x < 0 { redde "negative" }
                sin x == 0 { redde "zero" }
                secus { redde "positive" }
            `);
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.alternate.type).toBe('SiStatement');
            expect(stmt.alternate.alternate.type).toBe('BlockStatement');
        });

        test('secus as else alias', () => {
            const { program } = parseCode(`
                si x < 0 { redde "negative" }
                sin x == 0 { redde "zero" }
                secus { redde "positive" }
            `);
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.alternate.type).toBe('SiStatement');
            expect(stmt.alternate.alternate.type).toBe('BlockStatement');
        });

        test('nihil unary operator', () => {
            const { program } = parseCode('si nihil x { scribe "null" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.test.type).toBe('UnaryExpression');
            expect(stmt.test.operator).toBe('nihil');
            expect(stmt.test.argument.name).toBe('x');
        });

        test('nonnihil unary operator', () => {
            const { program } = parseCode('si nonnihil x { scribe "not null" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.test.type).toBe('UnaryExpression');
            expect(stmt.test.operator).toBe('nonnihil');
            expect(stmt.test.argument.name).toBe('x');
        });

        test('est always parses type annotation', () => {
            // est is always followed by a type, use === for value equality
            const { program } = parseCode('si x est textus { scribe "string" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.test.type).toBe('EstExpression');
            expect(stmt.test.expression.name).toBe('x');
            expect(stmt.test.targetType.name).toBe('textus');
        });

        test('est with primitive types', () => {
            const { program } = parseCode('si val est textus { scribe "string" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.test.type).toBe('EstExpression');
            expect(stmt.test.expression.name).toBe('val');
            expect(stmt.test.targetType.name).toBe('textus');
            expect(stmt.test.negated).toBe(false);
        });

        test('est with generic type', () => {
            const { program } = parseCode('si items est textus[] { scribe "array" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.test.type).toBe('EstExpression');
            expect(stmt.test.targetType.name).toBe('lista');
            expect(stmt.test.targetType.typeParameters).toHaveLength(1);
        });

        test('est with user-defined type', () => {
            const { program } = parseCode('si obj est persona { scribe "is persona" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.test.type).toBe('EstExpression');
            expect(stmt.test.expression.name).toBe('obj');
            expect(stmt.test.targetType.name).toBe('persona');
        });

        test('negativum as unary check', () => {
            const { program } = parseCode('si negativum n { scribe "negative" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.test.type).toBe('UnaryExpression');
            expect(stmt.test.operator).toBe('negativum');
        });

        test('positivum as unary check', () => {
            const { program } = parseCode('si positivum n { scribe "positive" }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.test.type).toBe('UnaryExpression');
            expect(stmt.test.operator).toBe('positivum');
        });

        test('dum with ergo', () => {
            const { program } = parseCode('dum x > 0 ergo x = x - 1');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('DumStatement');
            expect(stmt.body.type).toBe('BlockStatement');
            expect(stmt.body.body).toHaveLength(1);
            expect(stmt.body.body[0].expression.type).toBe('AssignmentExpression');
        });

        test('ex...pro with ergo', () => {
            const { program } = parseCode('ex items pro item ergo scribe item');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.kind).toBe('ex');
            expect(stmt.body.type).toBe('BlockStatement');
            expect(stmt.body.body).toHaveLength(1);
            expect(stmt.body.body[0]!.type).toBe('ScribeStatement');
        });

        test('de...pro with ergo', () => {
            const { program } = parseCode('de obj pro key ergo scribe key');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.kind).toBe('in');
            expect(stmt.body.type).toBe('BlockStatement');
            expect(stmt.body.body).toHaveLength(1);
        });

        test('ergo with function call', () => {
            const { program } = parseCode('si active ergo process()');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body[0].expression.type).toBe('CallExpression');
        });

        test('ergo with assignment', () => {
            const { program } = parseCode('si valid ergo result = 1');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body[0].expression.type).toBe('AssignmentExpression');
        });

        test('ergo with return', () => {
            const { program } = parseCode('si done ergo redde result');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body[0]!.type).toBe('ReddeStatement');
        });

        test('ergo with rumpe', () => {
            const { program } = parseCode('si done ergo rumpe');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body[0]!.type).toBe('RumpeStatement');
        });

        test('ergo with perge', () => {
            const { program } = parseCode('si skip ergo perge');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.consequent.type).toBe('BlockStatement');
            expect(stmt.consequent.body[0]!.type).toBe('PergeStatement');
        });
    });

    describe('break and continue (rumpe/perge)', () => {
        test('rumpe parses as RumpeStatement', () => {
            const { program } = parseCode('dum verum { rumpe }');
            const loop = program!.body[0] as any;

            expect(loop.type).toBe('DumStatement');
            expect(loop.body.body[0]!.type).toBe('RumpeStatement');
        });

        test('perge parses as PergeStatement', () => {
            const { program } = parseCode('dum verum { perge }');
            const loop = program!.body[0] as any;

            expect(loop.type).toBe('DumStatement');
            expect(loop.body.body[0]!.type).toBe('PergeStatement');
        });

        test('rumpe in for loop', () => {
            const { program } = parseCode('ex items pro item { rumpe }');
            const loop = program!.body[0] as any;

            expect(loop.type).toBe('IteratioStatement');
            expect(loop.body.body[0]!.type).toBe('RumpeStatement');
        });

        test('perge in for loop', () => {
            const { program } = parseCode('ex items pro item { perge }');
            const loop = program!.body[0] as any;

            expect(loop.type).toBe('IteratioStatement');
            expect(loop.body.body[0]!.type).toBe('PergeStatement');
        });

        test('rumpe inside conditional in loop', () => {
            const { program } = parseCode('dum verum { si found { rumpe } }');
            const loop = program!.body[0] as any;
            const ifStmt = loop.body.body[0];

            expect(ifStmt.type).toBe('SiStatement');
            expect(ifStmt.consequent.body[0]!.type).toBe('RumpeStatement');
        });

        test('perge inside conditional in loop', () => {
            const { program } = parseCode('dum verum { si skip { perge } }');
            const loop = program!.body[0] as any;
            const ifStmt = loop.body.body[0];

            expect(ifStmt.type).toBe('SiStatement');
            expect(ifStmt.consequent.body[0]!.type).toBe('PergeStatement');
        });

        test('multiple rumpe and perge in same loop', () => {
            const { program } = parseCode(`
                dum verum {
                    si skip { perge }
                    si done { rumpe }
                }
            `);
            const loop = program!.body[0] as any;

            expect(loop.body.body[0].consequent.body[0]!.type).toBe('PergeStatement');
            expect(loop.body.body[1].consequent.body[0]!.type).toBe('RumpeStatement');
        });
    });

    describe('loop verb conjugation (fit/fiet)', () => {
        test('ex...fit parses as sync loop', () => {
            const { program } = parseCode('ex items fit item { scribe item }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.kind).toBe('ex');
            expect(stmt.async).toBe(false);
            expect(stmt.variable.name).toBe('item');
        });

        test('ex...fiet parses as async loop', () => {
            const { program } = parseCode('ex stream fiet chunk { scribe chunk }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.kind).toBe('ex');
            expect(stmt.async).toBe(true);
            expect(stmt.variable.name).toBe('chunk');
        });

        test('ex...pro remains sync (backward compatibility)', () => {
            const { program } = parseCode('ex items pro item { scribe item }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.async).toBe(false);
        });

        test('de...fit parses as sync loop', () => {
            const { program } = parseCode('de obj fit key { scribe key }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.kind).toBe('in');
            expect(stmt.async).toBe(false);
        });

        test('ex...fit with ergo one-liner', () => {
            const { program } = parseCode('ex items fit item ergo scribe item');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.async).toBe(false);
            expect(stmt.body.body[0]!.type).toBe('ScribeStatement');
        });

        test('ex...fiet with ergo one-liner', () => {
            const { program } = parseCode('ex stream fiet chunk ergo process(chunk)');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.async).toBe(true);
            expect(stmt.body.body[0].expression.type).toBe('CallExpression');
        });
    });

    describe('collection DSL', () => {
        test('ex...prima...pro parses iteration with transform', () => {
            const { program } = parseCode('ex items prima 5 pro item { scribe item }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.kind).toBe('ex');
            expect(stmt.variable.name).toBe('item');
            expect(stmt.transforms).toHaveLength(1);
            expect(stmt.transforms[0].verb).toBe('prima');
            expect(stmt.transforms[0].argument.value).toBe(5);
        });

        test('ex...ultima...pro parses iteration with transform', () => {
            const { program } = parseCode('ex items ultima 3 pro item { scribe item }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.transforms).toHaveLength(1);
            expect(stmt.transforms[0].verb).toBe('ultima');
            expect(stmt.transforms[0].argument.value).toBe(3);
        });

        test('ex...prima...ultima...pro parses chained transforms', () => {
            const { program } = parseCode('ex items prima 10, ultima 3 pro item { scribe item }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.transforms).toHaveLength(2);
            expect(stmt.transforms[0].verb).toBe('prima');
            expect(stmt.transforms[0].argument.value).toBe(10);
            expect(stmt.transforms[1].verb).toBe('ultima');
            expect(stmt.transforms[1].argument.value).toBe(3);
        });

        test('ex items without transforms has no transforms property', () => {
            const { program } = parseCode('ex items pro item { scribe item }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('IteratioStatement');
            expect(stmt.transforms).toBeUndefined();
        });

        test('ex...summa parses as DSL expression', () => {
            const { program } = parseCode('fixum total = ex prices summa');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariaDeclaration');
            expect(decl.init.type).toBe('CollectionDSLExpression');
            expect(decl.init.source.name).toBe('prices');
            expect(decl.init.transforms).toHaveLength(1);
            expect(decl.init.transforms[0].verb).toBe('summa');
            expect(decl.init.transforms[0].argument).toBeUndefined();
        });

        test('ex...prima parses as DSL expression', () => {
            const { program } = parseCode('fixum top5 = ex items prima 5');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('VariaDeclaration');
            expect(decl.init.type).toBe('CollectionDSLExpression');
            expect(decl.init.transforms[0].verb).toBe('prima');
            expect(decl.init.transforms[0].argument.value).toBe(5);
        });

        test('ex...prima...ultima parses chained DSL expression', () => {
            const { program } = parseCode('fixum result = ex items prima 10, ultima 3');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('CollectionDSLExpression');
            expect(decl.init.transforms).toHaveLength(2);
            expect(decl.init.transforms[0].verb).toBe('prima');
            expect(decl.init.transforms[1].verb).toBe('ultima');
        });
    });

    describe('ab expression (filtering DSL)', () => {
        test('ab with boolean property shorthand', () => {
            const { program } = parseCode('fixum active = ab users activus');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.source.name).toBe('users');
            expect(decl.init.negated).toBe(false);
            expect(decl.init.filter.hasUbi).toBe(false);
            expect(decl.init.filter.condition.name).toBe('activus');
        });

        test('ab with negated boolean property', () => {
            const { program } = parseCode('fixum clean = ab users non banned');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.negated).toBe(true);
            expect(decl.init.filter.hasUbi).toBe(false);
            expect(decl.init.filter.condition.name).toBe('banned');
        });

        test('ab with ubi condition', () => {
            const { program } = parseCode('fixum adults = ab users ubi aetas >= 18');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.negated).toBe(false);
            expect(decl.init.filter.hasUbi).toBe(true);
            expect(decl.init.filter.condition.type).toBe('BinaryExpression');
        });

        test('ab with negated ubi condition', () => {
            const { program } = parseCode('fixum excluded = ab users non ubi banned');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.negated).toBe(true);
            expect(decl.init.filter.hasUbi).toBe(true);
        });

        test('ab with transforms', () => {
            const { program } = parseCode('fixum top2 = ab users activus, prima 2');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.filter.condition.name).toBe('activus');
            expect(decl.init.transforms).toHaveLength(1);
            expect(decl.init.transforms[0].verb).toBe('prima');
            expect(decl.init.transforms[0].argument.value).toBe(2);
        });

        test('ab with multiple transforms', () => {
            const { program } = parseCode('fixum result = ab items visible, prima 10, ultima 3');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.filter.condition.name).toBe('visible');
            expect(decl.init.transforms).toHaveLength(2);
            expect(decl.init.transforms[0].verb).toBe('prima');
            expect(decl.init.transforms[1].verb).toBe('ultima');
        });

        test('ab with ubi and transforms', () => {
            const { program } = parseCode('fixum top = ab users ubi score > 50, prima 5');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.filter.hasUbi).toBe(true);
            expect(decl.init.filter.condition.type).toBe('BinaryExpression');
            expect(decl.init.transforms).toHaveLength(1);
            expect(decl.init.transforms[0].verb).toBe('prima');
        });

        test('ab with complex ubi condition', () => {
            const { program } = parseCode('fixum result = ab users ubi aetas >= 18 et activus');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.filter.hasUbi).toBe(true);
            expect(decl.init.filter.condition.type).toBe('BinaryExpression');
            expect(decl.init.filter.condition.operator).toBe('&&');
        });

        test('ab with member expression source', () => {
            const { program } = parseCode('fixum active = ab data.users activus');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.source.type).toBe('MemberExpression');
            expect(decl.init.source.object.name).toBe('data');
            expect(decl.init.source.property.name).toBe('users');
        });

        test('ab with call expression source', () => {
            const { program } = parseCode('fixum active = ab getUsers() activus');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.source.type).toBe('CallExpression');
            expect(decl.init.source.callee.name).toBe('getUsers');
        });

        test('ab without filter (just source)', () => {
            const { program } = parseCode('fixum all = ab users, prima 5');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.source.name).toBe('users');
            expect(decl.init.filter).toBeUndefined();
            expect(decl.init.transforms).toHaveLength(1);
        });

        test('ab with summa transform', () => {
            const { program } = parseCode('fixum total = ab prices valid, summa');
            const decl = program!.body[0] as any;

            expect(decl.init.type).toBe('AbExpression');
            expect(decl.init.filter.condition.name).toBe('valid');
            expect(decl.init.transforms).toHaveLength(1);
            expect(decl.init.transforms[0].verb).toBe('summa');
            expect(decl.init.transforms[0].argument).toBeUndefined();
        });
    });

    describe('edge cases - empty constructs', () => {
        test('empty block', () => {
            const { program } = parseCode('{}');

            expect(program!.body[0]!.type).toBe('BlockStatement');
            expect((program!.body[0] as any).body).toHaveLength(0);
        });

        test('empty function body', () => {
            const { program } = parseCode('functio f() {}');
            const fn = program!.body[0] as any;

            expect(fn.type).toBe('FunctioDeclaration');
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
            const { program } = parseCode('si verum { } secus { }');
            const stmt = program!.body[0] as any;

            expect(stmt.type).toBe('SiStatement');
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

            expect(stmt.type).toBe('SiStatement');
            expect(stmt.consequent.body[0]!.type).toBe('SiStatement');
            expect(stmt.consequent.body[0].consequent.body[0]!.type).toBe('SiStatement');
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
            const { program } = parseCode('non (a et b)');
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

        test('mixing vel and aut without parentheses is error', () => {
            const { errors } = parseCode('a vel b aut c');

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('Cannot mix');
        });

        test('vel with parentheses around aut is allowed', () => {
            const { program, errors } = parseCode('a vel (b aut c)');

            expect(errors.length).toBe(0);
            expect(program!.body[0]).toBeDefined();
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
            const { errors } = parseCode('elige x { casu }');

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

        // NOTE: Inline futura/cursor modifiers were removed.
        // Use @ futura / @ cursor annotations instead.

        // Unsupported operators from other languages
        test('Fail when using postfix increment', () => {
            const { errors } = parseCode('x++');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('Fail when using prefix increment', () => {
            const { errors } = parseCode('++x');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('Fail when using postfix decrement', () => {
            const { errors } = parseCode('x--');

            expect(errors.length).toBeGreaterThan(0);
        });

        // WHY: --x parses as -(-x) which is valid double negation
        // Only postfix forms (x++, x--) produce errors

        test('compound assignment += parses correctly', () => {
            const { program, errors } = parseCode('x += 1');

            expect(errors.length).toBe(0);
            const expr = (program!.body[0] as any).expression;
            expect(expr.type).toBe('AssignmentExpression');
            expect(expr.operator).toBe('+=');
        });

        test('compound assignment -= parses correctly', () => {
            const { program, errors } = parseCode('x -= 1');

            expect(errors.length).toBe(0);
            const expr = (program!.body[0] as any).expression;
            expect(expr.type).toBe('AssignmentExpression');
            expect(expr.operator).toBe('-=');
        });

        test('compound assignment *= parses correctly', () => {
            const { program, errors } = parseCode('x *= 2');

            expect(errors.length).toBe(0);
            const expr = (program!.body[0] as any).expression;
            expect(expr.type).toBe('AssignmentExpression');
            expect(expr.operator).toBe('*=');
        });

        test('compound assignment /= parses correctly', () => {
            const { program, errors } = parseCode('x /= 2');

            expect(errors.length).toBe(0);
            const expr = (program!.body[0] as any).expression;
            expect(expr.type).toBe('AssignmentExpression');
            expect(expr.operator).toBe('/=');
        });

        test('compound assignment &= parses correctly', () => {
            const { program, errors } = parseCode('x &= 0xFF');

            expect(errors.length).toBe(0);
            const expr = (program!.body[0] as any).expression;
            expect(expr.type).toBe('AssignmentExpression');
            expect(expr.operator).toBe('&=');
        });

        test('compound assignment |= parses correctly', () => {
            const { program, errors } = parseCode('x |= 0x01');

            expect(errors.length).toBe(0);
            const expr = (program!.body[0] as any).expression;
            expect(expr.type).toBe('AssignmentExpression');
            expect(expr.operator).toBe('|=');
        });

        test('Fail when using exponentiation operator **', () => {
            const { errors } = parseCode('a ** b');

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
                const { program } = parseCode('pro x: x * 2');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(1);
                expect(expr.params[0].name).toBe('x');
                expect(expr.body.type).toBe('BinaryExpression');
                expect(expr.async).toBe(false);
            });

            test('multi param lambda', () => {
                const { program } = parseCode('pro a, b: a + b');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(2);
                expect(expr.params[0].name).toBe('a');
                expect(expr.params[1].name).toBe('b');
            });

            test('zero param lambda', () => {
                const { program } = parseCode('pro: 42');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(0);
                expect(expr.body.value).toBe(42);
            });

            test('lambda in variable declaration', () => {
                const { program } = parseCode('fixum double = pro x: x * 2');
                const decl = program!.body[0] as any;

                expect(decl.type).toBe('VariaDeclaration');
                expect(decl.init.type).toBe('LambdaExpression');
            });

            test('nested lambdas', () => {
                const { program } = parseCode('pro x: pro y: x + y');
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

                expect(decl.type).toBe('VariaDeclaration');
                expect(decl.init.type).toBe('LambdaExpression');
            });

            test('mixed : and redde in nested lambdas', () => {
                const { program } = parseCode('pro x: pro y redde x + y');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.body.type).toBe('LambdaExpression');
            });
        });

        describe('pro expression with return type annotation', () => {
            test('lambda with return type and : shorthand', () => {
                const { program } = parseCode('pro x -> numerus: x * 2');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(1);
                expect(expr.params[0].name).toBe('x');
                expect(expr.returnType).toBeDefined();
                expect(expr.returnType.name).toBe('numerus');
                expect(expr.body.type).toBe('BinaryExpression');
            });

            test('lambda with return type and redde', () => {
                const { program } = parseCode('pro x -> numerus redde x * 2');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.returnType).toBeDefined();
                expect(expr.returnType.name).toBe('numerus');
            });

            test('lambda with return type and block body', () => {
                const { program } = parseCode('pro x -> textus { redde "hello" }');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.returnType).toBeDefined();
                expect(expr.returnType.name).toBe('textus');
                expect(expr.body.type).toBe('BlockStatement');
            });

            test('zero-param lambda with return type', () => {
                const { program } = parseCode('pro -> numerus: 42');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(0);
                expect(expr.returnType).toBeDefined();
                expect(expr.returnType.name).toBe('numerus');
            });

            test('multi-param lambda with return type', () => {
                const { program } = parseCode('pro a, b -> numerus: a + b');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(2);
                expect(expr.returnType).toBeDefined();
                expect(expr.returnType.name).toBe('numerus');
            });

            test('lambda without return type has undefined returnType', () => {
                const { program } = parseCode('pro x: x * 2');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.returnType).toBeUndefined();
            });

            test('lambda with nullable return type', () => {
                const { program } = parseCode('pro x -> textus?: nihil');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.returnType).toBeDefined();
                expect(expr.returnType.name).toBe('textus');
                expect(expr.returnType.nullable).toBe(true);
            });

            test('lambda with generic return type', () => {
                const { program } = parseCode('pro x -> numerus[]: [x]');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.returnType).toBeDefined();
                expect(expr.returnType.name).toBe('lista');
                expect(expr.returnType.typeParameters).toHaveLength(1);
            });
        });

        describe('fit expression (sync lambda, explicit)', () => {
            test('fit is equivalent to pro (sync)', () => {
                const { program } = parseCode('fit x: x * 2');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(1);
                expect(expr.params[0].name).toBe('x');
                expect(expr.async).toBe(false);
            });

            test('fit with block body', () => {
                const { program } = parseCode('fit x { redde x * 2 }');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.body.type).toBe('BlockStatement');
                expect(expr.async).toBe(false);
            });

            test('fit zero params', () => {
                const { program } = parseCode('fit: 42');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(0);
                expect(expr.async).toBe(false);
            });
        });

        describe('fiet expression (async lambda)', () => {
            test('fiet creates async lambda', () => {
                const { program } = parseCode('fiet x: x * 2');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(1);
                expect(expr.params[0].name).toBe('x');
                expect(expr.async).toBe(true);
            });

            test('fiet with block body', () => {
                const { program } = parseCode('fiet c { redde c.json() }');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.body.type).toBe('BlockStatement');
                expect(expr.async).toBe(true);
            });

            test('fiet zero params', () => {
                const { program } = parseCode('fiet: fetch()');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(0);
                expect(expr.async).toBe(true);
            });

            test('fiet multi params', () => {
                const { program } = parseCode('fiet a, b { redde a + b }');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.params).toHaveLength(2);
                expect(expr.async).toBe(true);
            });

            test('fiet with return type annotation', () => {
                const { program } = parseCode('fiet x -> textus: x');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('LambdaExpression');
                expect(expr.returnType).toBeDefined();
                expect(expr.returnType.name).toBe('textus');
                expect(expr.async).toBe(true);
            });

            test('fiet in function call', () => {
                const { program } = parseCode('app.post("/users", fiet c { redde c.json() })');
                const expr = (program!.body[0] as any).expression;

                expect(expr.type).toBe('CallExpression');
                const lambda = expr.arguments[1];
                expect(lambda.type).toBe('LambdaExpression');
                expect(lambda.async).toBe(true);
            });
        });
    });

    describe('import declarations', () => {
        test('string path source', () => {
            const { program } = parseCode('ex "norma/tempus" importa nunc, dormi');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('ImportaDeclaration');
            expect(decl.source).toBe('norma/tempus');
            expect(decl.specifiers).toHaveLength(2);
            expect(decl.specifiers[0].imported.name).toBe('nunc');
            expect(decl.specifiers[0].local.name).toBe('nunc');
            expect(decl.specifiers[1].imported.name).toBe('dormi');
        });

        test('wildcard import', () => {
            const { program } = parseCode('ex "norma/tempus" importa *');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('ImportaDeclaration');
            expect(decl.wildcard).toBe(true);
            expect(decl.specifiers).toHaveLength(0);
        });

        test('single import', () => {
            const { program } = parseCode('ex "norma/tempus" importa nunc');
            const decl = program!.body[0] as any;

            expect(decl.specifiers).toHaveLength(1);
            expect(decl.specifiers[0].imported.name).toBe('nunc');
        });

        test('import with constants', () => {
            const { program } = parseCode('ex "norma/tempus" importa SECUNDUM, MINUTUM, HORA');
            const decl = program!.body[0] as any;

            expect(decl.specifiers).toHaveLength(3);
            expect(decl.specifiers[0].imported.name).toBe('SECUNDUM');
            expect(decl.specifiers[1].imported.name).toBe('MINUTUM');
            expect(decl.specifiers[2].imported.name).toBe('HORA');
        });

        test('import with ut alias', () => {
            const { program } = parseCode('ex norma importa scribe ut s, lege ut l');
            const decl = program!.body[0] as any;

            expect(decl.type).toBe('ImportaDeclaration');
            expect(decl.specifiers).toHaveLength(2);
            expect(decl.specifiers[0].imported.name).toBe('scribe');
            expect(decl.specifiers[0].local.name).toBe('s');
            expect(decl.specifiers[1].imported.name).toBe('lege');
            expect(decl.specifiers[1].local.name).toBe('l');
        });

        test('import with mixed aliases', () => {
            const { program } = parseCode('ex norma importa scribe ut s, lege');
            const decl = program!.body[0] as any;

            expect(decl.specifiers).toHaveLength(2);
            expect(decl.specifiers[0].imported.name).toBe('scribe');
            expect(decl.specifiers[0].local.name).toBe('s');
            expect(decl.specifiers[1].imported.name).toBe('lege');
            expect(decl.specifiers[1].local.name).toBe('lege'); // no alias, same name
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

    describe('proba - test syntax', () => {
        describe('probandum (test suite)', () => {
            test('simple probandum', () => {
                const { program } = parseCode('probandum "Suite" { }');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('ProbandumStatement');
                expect(stmt.name).toBe('Suite');
                expect(stmt.body).toHaveLength(0);
            });

            test('probandum with proba', () => {
                const { program } = parseCode(`
                    probandum "Math" {
                        proba "adds" { adfirma 1 + 1 est 2 }
                    }
                `);
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('ProbandumStatement');
                expect(stmt.name).toBe('Math');
                expect(stmt.body).toHaveLength(1);
                expect(stmt.body[0]!.type).toBe('ProbaStatement');
            });

            test('nested probandum', () => {
                const { program } = parseCode(`
                    probandum "Outer" {
                        probandum "Inner" {
                            proba "test" { }
                        }
                    }
                `);
                const outer = program!.body[0] as any;
                const inner = outer.body[0];

                expect(outer.type).toBe('ProbandumStatement');
                expect(outer.name).toBe('Outer');
                expect(inner.type).toBe('ProbandumStatement');
                expect(inner.name).toBe('Inner');
            });
        });

        describe('proba (test case)', () => {
            test('simple proba', () => {
                const { program } = parseCode('proba "test name" { adfirma verum }');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('ProbaStatement');
                expect(stmt.name).toBe('test name');
                expect(stmt.modifier).toBeUndefined();
                expect(stmt.body.type).toBe('BlockStatement');
            });

            test('proba omitte (skip)', () => {
                const { program } = parseCode('proba omitte "reason" "skipped test" { }');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('ProbaStatement');
                expect(stmt.modifier).toBe('omitte');
                expect(stmt.modifierReason).toBe('reason');
                expect(stmt.name).toBe('skipped test');
            });

            test('proba futurum (todo)', () => {
                const { program } = parseCode('proba futurum "not yet" "future test" { }');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('ProbaStatement');
                expect(stmt.modifier).toBe('futurum');
                expect(stmt.modifierReason).toBe('not yet');
                expect(stmt.name).toBe('future test');
            });

            test('proba at top level', () => {
                const { program } = parseCode('proba "standalone" { adfirma 1 est 1 }');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('ProbaStatement');
                expect(stmt.name).toBe('standalone');
            });
        });

        describe('praepara/postpara (setup/teardown)', () => {
            test('praepara (beforeEach)', () => {
                const { program } = parseCode(`
                    probandum "Suite" {
                        praepara { x = 0 }
                        proba "test" { }
                    }
                `);
                const suite = program!.body[0] as any;
                const block = suite.body[0];

                expect(block.type).toBe('PraeparaBlock');
                expect(block.timing).toBe('praepara');
                expect(block.async).toBe(false);
                expect(block.omnia).toBe(false);
            });

            test('praepara omnia (beforeAll)', () => {
                const { program } = parseCode(`
                    probandum "Suite" {
                        praepara omnia { db = connect() }
                    }
                `);
                const suite = program!.body[0] as any;
                const block = suite.body[0];

                expect(block.type).toBe('PraeparaBlock');
                expect(block.timing).toBe('praepara');
                expect(block.async).toBe(false);
                expect(block.omnia).toBe(true);
            });

            test('praeparabit omnia (async beforeAll)', () => {
                const { program } = parseCode(`
                    probandum "Suite" {
                        praeparabit omnia { db = cede connect() }
                    }
                `);
                const suite = program!.body[0] as any;
                const block = suite.body[0];

                expect(block.type).toBe('PraeparaBlock');
                expect(block.timing).toBe('praepara');
                expect(block.async).toBe(true);
                expect(block.omnia).toBe(true);
            });

            test('postpara (afterEach)', () => {
                const { program } = parseCode(`
                    probandum "Suite" {
                        postpara { cleanup() }
                    }
                `);
                const suite = program!.body[0] as any;
                const block = suite.body[0];

                expect(block.type).toBe('PraeparaBlock');
                expect(block.timing).toBe('postpara');
                expect(block.async).toBe(false);
                expect(block.omnia).toBe(false);
            });

            test('postpara omnia (afterAll)', () => {
                const { program } = parseCode(`
                    probandum "Suite" {
                        postpara omnia { db.close() }
                    }
                `);
                const suite = program!.body[0] as any;
                const block = suite.body[0];

                expect(block.type).toBe('PraeparaBlock');
                expect(block.timing).toBe('postpara');
                expect(block.async).toBe(false);
                expect(block.omnia).toBe(true);
            });

            test('postparabit omnia (async afterAll)', () => {
                const { program } = parseCode(`
                    probandum "Suite" {
                        postparabit omnia { cede db.close() }
                    }
                `);
                const suite = program!.body[0] as any;
                const block = suite.body[0];

                expect(block.type).toBe('PraeparaBlock');
                expect(block.timing).toBe('postpara');
                expect(block.async).toBe(true);
                expect(block.omnia).toBe(true);
            });

            test('praepara at top level', () => {
                const { program } = parseCode('praepara { setup() }');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('PraeparaBlock');
                expect(stmt.timing).toBe('praepara');
            });
        });

        describe('full test suite', () => {
            test('complete probandum with all features', () => {
                const { program } = parseCode(`
                    probandum "Database" {
                        praepara omnia { db = connect() }
                        praepara { db.reset() }

                        proba "inserts" { adfirma db.count() est 0 }
                        proba omitte "broken" "updates" { }
                        proba futurum "later" "deletes" { }

                        postpara { db.rollback() }
                        postpara omnia { db.close() }
                    }
                `);
                const suite = program!.body[0] as any;

                expect(suite.type).toBe('ProbandumStatement');
                expect(suite.body).toHaveLength(7);

                // Check order and types
                expect(suite.body[0]!.type).toBe('PraeparaBlock');
                expect(suite.body[0].timing).toBe('praepara');
                expect(suite.body[0].omnia).toBe(true);

                expect(suite.body[1]!.type).toBe('PraeparaBlock');
                expect(suite.body[1].timing).toBe('praepara');
                expect(suite.body[1].omnia).toBe(false);

                expect(suite.body[2].type).toBe('ProbaStatement');
                expect(suite.body[2].name).toBe('inserts');

                expect(suite.body[3].type).toBe('ProbaStatement');
                expect(suite.body[3].modifier).toBe('omitte');

                expect(suite.body[4].type).toBe('ProbaStatement');
                expect(suite.body[4].modifier).toBe('futurum');

                expect(suite.body[5].type).toBe('PraeparaBlock');
                expect(suite.body[5].timing).toBe('postpara');

                expect(suite.body[6].type).toBe('PraeparaBlock');
                expect(suite.body[6].timing).toBe('postpara');
                expect(suite.body[6].omnia).toBe(true);
            });
        });

        describe('post as identifier (not keyword)', () => {
            test('post can be used as method name', () => {
                const { program } = parseCode(`
                    pactum DataFetcher {
                        functio post(textus url) futura -> textus
                    }
                `);
                const pactum = program!.body[0] as any;
                const method = pactum.methods[0];

                expect(method.name.name).toBe('post');
            });

            test('post can be used as variable name', () => {
                const { program } = parseCode('fixum post = "data"');
                const decl = program!.body[0] as any;

                expect(decl.name.name).toBe('post');
            });

            test('post can be used as function name', () => {
                const { program } = parseCode('functio post() { redde nihil }');
                const fn = program!.body[0] as any;

                expect(fn.name.name).toBe('post');
            });
        });

        describe('cura statement (resource management)', () => {
            test('cura arena allocator', () => {
                const { program } = parseCode('cura arena fit mem { process(mem) }');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('CuraStatement');
                expect(stmt.curatorKind).toBe('arena');
                expect(stmt.binding.name).toBe('mem');
                expect(stmt.async).toBe(false);
                expect(stmt.body.type).toBe('BlockStatement');
            });

            test('cura page allocator', () => {
                const { program } = parseCode('cura page fit mem { allocate(mem, 1024) }');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('CuraStatement');
                expect(stmt.curatorKind).toBe('page');
                expect(stmt.binding.name).toBe('mem');
            });

            // WHY: liber, conexio, mutex curator kinds are planned but not yet implemented
            test.todo('cura liber with file resource', () => {});
            test.todo('cura conexio with async acquisition', () => {});
            test.todo('cura mutex with cape (catch)', () => {});

            test('nested cura arena statements', () => {
                const { program } = parseCode(`
                    cura arena fit outer {
                        cura arena fit inner {
                            process(inner)
                        }
                    }
                `);
                const outer = program!.body[0] as any;
                const inner = outer.body.body[0] as any;

                expect(outer.type).toBe('CuraStatement');
                expect(outer.binding.name).toBe('outer');
                expect(inner.type).toBe('CuraStatement');
                expect(inner.binding.name).toBe('inner');
            });

            // WHY: conexio curator kind is planned but not yet implemented
            test.todo('cura conexio with async and cape', () => {});

            test('distinguishes praepara (test) from cura (resource)', () => {
                const { program } = parseCode(`
                    praepara { setup() }
                    cura arena fit mem { use(mem) }
                `);

                expect(program!.body[0]!.type).toBe('PraeparaBlock');
                expect(program!.body[1]!.type).toBe('CuraStatement');
            });

            // WHY: liber, conexio curator kinds are planned but not yet implemented
            test.todo('cura liber with type annotation', () => {});
            test.todo('cura conexio with fiet and type annotation', () => {});
        });
    });

    describe('prae (compile-time)', () => {
        describe('prae typus (type parameters)', () => {
            test('function with single type parameter', () => {
                const { program } = parseCode(`
                    functio identity(prae typus T, T value) -> T {
                        redde value
                    }
                `);
                const fn = program!.body[0] as any;

                expect(fn.type).toBe('FunctioDeclaration');
                expect(fn.typeParams).toHaveLength(1);
                expect(fn.typeParams[0].name.name).toBe('T');
                expect(fn.params).toHaveLength(1);
                expect(fn.params[0].typeAnnotation.name).toBe('T');
            });

            test('function with multiple type parameters', () => {
                const { program } = parseCode(`
                    functio pair(prae typus K, prae typus V, K key, V value) -> K[] {
                        redde nihil
                    }
                `);
                const fn = program!.body[0] as any;

                expect(fn.typeParams).toHaveLength(2);
                expect(fn.typeParams[0].name.name).toBe('K');
                expect(fn.typeParams[1].name.name).toBe('V');
                expect(fn.params).toHaveLength(2);
            });

            test('function with type parameter and preposition', () => {
                const { program } = parseCode(`
                    functio swap(prae typus T, in T a, in T b) {
                        varia temp = a
                        a = b
                        b = temp
                    }
                `);
                const fn = program!.body[0] as any;

                expect(fn.typeParams).toHaveLength(1);
                expect(fn.params[0].preposition).toBe('in');
                expect(fn.params[1].preposition).toBe('in');
            });

            test('function with only type parameter, no regular params', () => {
                const { program } = parseCode(`
                    functio create(prae typus T) -> T {
                        redde novum T
                    }
                `);
                const fn = program!.body[0] as any;

                expect(fn.typeParams).toHaveLength(1);
                expect(fn.typeParams[0].name.name).toBe('T');
                expect(fn.params).toHaveLength(0);
            });
        });

        describe('praefixum (compile-time expressions)', () => {
            test('praefixum with parenthesized expression', () => {
                const { program } = parseCode('fixum size = praefixum(256 * 4)');
                const decl = program!.body[0] as any;

                expect(decl.init.type).toBe('PraefixumExpression');
                expect(decl.init.body.type).toBe('BinaryExpression');
            });

            test('praefixum with block', () => {
                const { program } = parseCode(`
                    fixum table = praefixum {
                        varia x = 10
                        redde x * x
                    }
                `);
                const decl = program!.body[0] as any;

                expect(decl.init.type).toBe('PraefixumExpression');
                expect(decl.init.body.type).toBe('BlockStatement');
                expect(decl.init.body.body).toHaveLength(2);
            });

            test('praefixum in variable initializer', () => {
                const { program } = parseCode('varia mask = praefixum(0xFF ^ 0xAA)');
                const decl = program!.body[0] as any;

                expect(decl.init.type).toBe('PraefixumExpression');
            });

            test('praefixum with simple literal', () => {
                const { program } = parseCode('fixum greeting = praefixum("Hello")');
                const decl = program!.body[0] as any;

                expect(decl.init.type).toBe('PraefixumExpression');
                expect(decl.init.body.type).toBe('Literal');
                expect(decl.init.body.value).toBe('Hello');
            });

            test('praefixum nested in expression', () => {
                const { program } = parseCode('fixum result = praefixum(1 + 2) + 3');
                const decl = program!.body[0] as any;

                expect(decl.init.type).toBe('BinaryExpression');
                expect(decl.init.left.type).toBe('PraefixumExpression');
            });
        });

        describe('discretio (tagged unions)', () => {
            test('basic discretio with unit variant', () => {
                const { program, errors } = parseCode('discretio State { Loading, Ready }');

                expect(errors).toHaveLength(0);
                expect(program!.body[0]).toBeDefined();

                const decl = program!.body[0] as any;
                expect(decl.type).toBe('DiscretioDeclaration');
                expect(decl.name.name).toBe('State');
                expect(decl.variants).toHaveLength(2);
                expect(decl.variants[0].name.name).toBe('Loading');
                expect(decl.variants[0].fields).toHaveLength(0);
                expect(decl.variants[1].name.name).toBe('Ready');
            });

            test('discretio with variant fields', () => {
                const { program, errors } = parseCode(`
                    discretio Event {
                        Click { numerus x, numerus y }
                        Keypress { textus key }
                        Quit
                    }
                `);

                expect(errors).toHaveLength(0);

                const decl = program!.body[0] as any;
                expect(decl.type).toBe('DiscretioDeclaration');
                expect(decl.name.name).toBe('Event');
                expect(decl.variants).toHaveLength(3);

                // Click variant with two fields
                const click = decl.variants[0];
                expect(click.name.name).toBe('Click');
                expect(click.fields).toHaveLength(2);
                expect(click.fields[0].name.name).toBe('x');
                expect(click.fields[0].fieldType.name).toBe('numerus');
                expect(click.fields[1].name.name).toBe('y');

                // Keypress variant with one field
                const keypress = decl.variants[1];
                expect(keypress.name.name).toBe('Keypress');
                expect(keypress.fields).toHaveLength(1);
                expect(keypress.fields[0].name.name).toBe('key');

                // Quit is a unit variant
                const quit = decl.variants[2];
                expect(quit.name.name).toBe('Quit');
                expect(quit.fields).toHaveLength(0);
            });

            test('discretio with generic type parameters', () => {
                const { program, errors } = parseCode(`
                    discretio Option<T> {
                        Some { T value }
                        None
                    }
                `);

                expect(errors).toHaveLength(0);

                const decl = program!.body[0] as any;
                expect(decl.type).toBe('DiscretioDeclaration');
                expect(decl.name.name).toBe('Option');
                expect(decl.typeParameters).toHaveLength(1);
                expect(decl.typeParameters[0].name).toBe('T');
            });

            test('variant case pattern matching with discerne', () => {
                const { program, errors } = parseCode(`
                    discerne event {
                        casu Click pro x, y { scribe x }
                        casu Quit { mori "goodbye" }
                    }
                `);

                expect(errors).toHaveLength(0);

                const discerneStmt = program!.body[0] as any;
                expect(discerneStmt.type).toBe('DiscerneStatement');
                expect(discerneStmt.cases).toHaveLength(2);

                // First case: casu Click pro x, y
                const clickCase = discerneStmt.cases[0];
                expect(clickCase.type).toBe('VariantCase');
                expect(clickCase.variant.name).toBe('Click');
                expect(clickCase.bindings).toHaveLength(2);
                expect(clickCase.bindings[0].name).toBe('x');
                expect(clickCase.bindings[1].name).toBe('y');

                // Second case: casu Quit (no bindings)
                const quitCase = discerneStmt.cases[1];
                expect(quitCase.type).toBe('VariantCase');
                expect(quitCase.variant.name).toBe('Quit');
                expect(quitCase.bindings).toHaveLength(0);
            });

            test('empty discretio parses but has no variants', () => {
                // WHY: Parser allows empty discretio; semantic analysis should warn
                const { program, errors } = parseCode('discretio Empty { }');

                expect(errors).toHaveLength(0);
                const decl = program!.body[0] as any;
                expect(decl.type).toBe('DiscretioDeclaration');
                expect(decl.variants).toHaveLength(0);
            });

            test('missing discretio name is an error', () => {
                const { errors } = parseCode('discretio { Loading }');

                expect(errors.length).toBeGreaterThan(0);
            });
        });

        describe('typus (type alias)', () => {
            test('basic type alias', () => {
                const { program } = parseCode('typus ID = textus');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('TypeAliasDeclaration');
                expect(stmt.name.name).toBe('ID');
                expect(stmt.typeAnnotation.name).toBe('textus');
                expect(stmt.typeofTarget).toBeUndefined();
            });

            test('type alias with generic type', () => {
                const { program } = parseCode('typus StringList = textus[]');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('TypeAliasDeclaration');
                expect(stmt.name.name).toBe('StringList');
                expect(stmt.typeAnnotation.name).toBe('lista');
                expect(stmt.typeAnnotation.typeParameters).toHaveLength(1);
            });

            test('typeof with typus RHS', () => {
                const { program } = parseCode('typus ConfigTypus = typus config');
                const stmt = program!.body[0] as any;

                expect(stmt.type).toBe('TypeAliasDeclaration');
                expect(stmt.name.name).toBe('ConfigTypus');
                expect(stmt.typeofTarget.name).toBe('config');
            });

            test('typeof with member expression target', () => {
                // Note: currently only supports simple identifiers
                const { program } = parseCode('typus T = typus x');
                const stmt = program!.body[0] as any;

                expect(stmt.typeofTarget.name).toBe('x');
            });
        });

        describe('ad statements (dispatch)', () => {
            describe('fire and forget', () => {
                test('no arguments', () => {
                    const { program, errors } = parseCode('ad "console:log" ()');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.type).toBe('AdStatement');
                    expect(stmt.target).toBe('console:log');
                    expect(stmt.arguments).toHaveLength(0);
                    expect(stmt.binding).toBeUndefined();
                    expect(stmt.body).toBeUndefined();
                });

                test('single argument', () => {
                    const { program, errors } = parseCode('ad "console:log" ("hello")');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.type).toBe('AdStatement');
                    expect(stmt.arguments).toHaveLength(1);
                    expect(stmt.arguments[0].value).toBe('hello');
                });

                test('multiple arguments', () => {
                    const { program, errors } = parseCode('ad "console:log" ("a", "b", 123)');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.arguments).toHaveLength(3);
                });

                test('spread argument', () => {
                    const { program, errors } = parseCode('ad "console:log" (sparge args)');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.arguments).toHaveLength(1);
                    expect(stmt.arguments[0].type).toBe('SpreadElement');
                });
            });

            describe('sync binding (fit)', () => {
                test('fit with type and binding', () => {
                    const { program, errors } = parseCode('ad "fasciculus:lege" ("file.txt") fit textus pro content { }');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.type).toBe('AdStatement');
                    expect(stmt.binding).toBeDefined();
                    expect(stmt.binding.type).toBe('AdBinding');
                    expect(stmt.binding.verb).toBe('fit');
                    expect(stmt.binding.typeAnnotation.name).toBe('textus');
                    expect(stmt.binding.name.name).toBe('content');
                    expect(stmt.body).toBeDefined();
                });

                test('fit with block body', () => {
                    const { program, errors } = parseCode(`
                        ad "fasciculus:lege" ("file.txt") fit textus pro content {
                            scribe content
                        }
                    `);

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.body.body).toHaveLength(1);
                    expect(stmt.body.body[0].type).toBe('ScribeStatement');
                });

                test('fit with generic type', () => {
                    const { program, errors } = parseCode('ad "db:query" (sql) fit lista<User> pro users { }');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.binding.typeAnnotation.name).toBe('lista');
                    expect(stmt.binding.typeAnnotation.typeParameters).toHaveLength(1);
                });

                test('fit with nullable type', () => {
                    const { program, errors } = parseCode('ad "cache:get" (key) fit textus? pro value { }');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.binding.typeAnnotation.nullable).toBe(true);
                });
            });

            describe('async binding (fiet)', () => {
                test('fiet with type and binding', () => {
                    const { program, errors } = parseCode('ad "http:get" (url) fiet Response pro response { }');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.binding.verb).toBe('fiet');
                    expect(stmt.binding.typeAnnotation.name).toBe('Response');
                    expect(stmt.binding.name.name).toBe('response');
                });
            });

            describe('plural binding (fiunt/fient)', () => {
                test('fiunt for sync plural', () => {
                    const { program, errors } = parseCode('ad "file:lines" (path) fiunt textus pro line { }');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.binding.verb).toBe('fiunt');
                });

                test('fient for async plural', () => {
                    const { program, errors } = parseCode('ad "http:batch" (urls) fient Response pro responses { }');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.binding.verb).toBe('fient');
                });
            });

            describe('type inference (pro without verb)', () => {
                test('pro without explicit verb defaults to fit', () => {
                    const { program, errors } = parseCode('ad "target" (args) pro result { }');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.binding.verb).toBe('fit');
                    expect(stmt.binding.typeAnnotation).toBeUndefined();
                    expect(stmt.binding.name.name).toBe('result');
                });
            });

            describe('alias (ut)', () => {
                test('fit with ut alias', () => {
                    const { program, errors } = parseCode('ad "target" (args) fit Type pro result ut r { }');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.binding.name.name).toBe('result');
                    expect(stmt.binding.alias.name).toBe('r');
                });

                test('fiet with ut alias', () => {
                    const { program, errors } = parseCode('ad "http:get" (url) fiet Response pro response ut res { }');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.binding.alias.name).toBe('res');
                });
            });

            describe('error handling (cape)', () => {
                test('cape clause after body', () => {
                    const { program, errors } = parseCode(`
                        ad "target" (x) fit Type pro r {
                            scribe r
                        } cape err {
                            scribe err
                        }
                    `);

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.catchClause).toBeDefined();
                    expect(stmt.catchClause.type).toBe('CapeClause');
                    expect(stmt.catchClause.param.name).toBe('err');
                });
            });

            describe('target patterns', () => {
                test('stdlib target (module:method)', () => {
                    const { program, errors } = parseCode('ad "fasciculus:lege" ("file.txt")');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.target).toBe('fasciculus:lege');
                });

                test('URL target (https)', () => {
                    const { program, errors } = parseCode('ad "https://api.example.com/users" ("GET")');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.target).toBe('https://api.example.com/users');
                });

                test('package target', () => {
                    const { program, errors } = parseCode('ad "hono/Hono" ()');

                    expect(errors).toHaveLength(0);
                    const stmt = program!.body[0] as any;
                    expect(stmt.target).toBe('hono/Hono');
                });
            });
        });
    });

    describe('annotations', () => {
        test('single annotation on functio', () => {
            const { program, errors } = parseCode('@ publica\nfunctio greet() -> textus { redde "hello" }');

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('FunctioDeclaration');
            expect(decl.annotations).toHaveLength(1);
            expect(decl.annotations[0].name).toBe('publica');
        });

        test('annotation with argument parses second word as argument', () => {
            // With new one-annotation-per-line design, "@ publica futura" is
            // annotation "publica" with argument "futura" (identifier expression)
            const { program, errors } = parseCode('@ publica futura\nfunctio fetch() -> textus { redde "data" }');

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.annotations).toHaveLength(1);
            expect(decl.annotations[0].name).toBe('publica');
            expect(decl.annotations[0].argument?.type).toBe('Identifier');
        });

        test('multiple annotations require multiple @ lines', () => {
            const { program, errors } = parseCode('@ publica\n@ futura\nfunctio fetch() -> textus { redde "data" }');

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.annotations).toHaveLength(2);
            expect(decl.annotations[0].name).toBe('publica');
            expect(decl.annotations[1].name).toBe('futura');
        });

        test('multiple annotations stacked', () => {
            const { program, errors } = parseCode('@ publicum\n@ abstractum\ngenus Base { }');

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('GenusDeclaration');
            expect(decl.annotations).toHaveLength(2);
            expect(decl.annotations[0]!.name).toBe('publicum');
            expect(decl.annotations[1]!.name).toBe('abstractum');
        });

        test('annotation on genus', () => {
            const { program, errors } = parseCode('@ publicum\ngenus User { textus nomen }');

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('GenusDeclaration');
            expect(decl.annotations).toHaveLength(1);
            expect(decl.annotations[0].name).toBe('publicum');
        });

        test('annotation on pactum', () => {
            const { program, errors } = parseCode('@ publicum\npactum Readable { functio read() -> textus }');

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('PactumDeclaration');
            expect(decl.annotations).toHaveLength(1);
        });

        test('annotation on ordo', () => {
            const { program, errors } = parseCode('@ publicus\nordo Status { Active, Inactive }');

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('OrdoDeclaration');
            expect(decl.annotations).toHaveLength(1);
            expect(decl.annotations[0].name).toBe('publicus');
        });

        test('annotation on discretio', () => {
            const { program, errors } = parseCode('@ publicum\ndiscretio Result { Ok, Err }');

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('DiscretioDeclaration');
            expect(decl.annotations).toHaveLength(1);
        });

        test('annotation on varia', () => {
            const { program, errors } = parseCode('@ publicum\nfixum PI = 3.14159');

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('VariaDeclaration');
            expect(decl.annotations).toHaveLength(1);
        });

        test('annotation on unsupported statement produces error', () => {
            const { program, errors } = parseCode('@ publicum\nsi verum { }');

            expect(program).not.toBeNull();
            expect(errors).toHaveLength(1);
            expect(errors[0]!.message).toContain('Annotations are not allowed');
        });

        test('empty annotation produces error', () => {
            const { program, errors } = parseCode('@ \nfunctio f() { }');

            expect(program).not.toBeNull();
            expect(errors).toHaveLength(1);
            expect(errors[0]!.message).toContain('Expected annotation name');
        });

        test('no annotation means undefined annotations field', () => {
            const { program, errors } = parseCode('functio greet() -> textus { redde "hello" }');

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('FunctioDeclaration');
            expect(decl.annotations).toBeUndefined();
        });

        test('annotation on genus field', () => {
            const { program, errors } = parseCode(`
                genus User {
                    textus nomen
                    @ privatum
                    textus secret
                }
            `);

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('GenusDeclaration');
            expect(decl.fields).toHaveLength(2);
            expect(decl.fields[0].annotations).toBeUndefined();
            expect(decl.fields[1].annotations).toHaveLength(1);
            expect(decl.fields[1].annotations[0].name).toBe('privatum');
        });

        test('annotation on genus method', () => {
            const { program, errors } = parseCode(`
                genus User {
                    @ privata
                    functio validate() -> bivalens { redde verum }
                }
            `);

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('GenusDeclaration');
            expect(decl.methods).toHaveLength(1);
            expect(decl.methods[0].annotations).toHaveLength(1);
            expect(decl.methods[0].annotations[0].name).toBe('privata');
        });

        test('annotation on pactum method', () => {
            const { program, errors } = parseCode(`
                pactum Readable {
                    @ publica
                    functio read() -> textus
                }
            `);

            expect(errors).toHaveLength(0);
            const decl = program!.body[0] as any;
            expect(decl.type).toBe('PactumDeclaration');
            expect(decl.methods).toHaveLength(1);
            expect(decl.methods[0].annotations).toHaveLength(1);
            expect(decl.methods[0].annotations[0].name).toBe('publica');
        });
    });
});
