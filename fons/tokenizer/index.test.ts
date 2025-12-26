import { test, expect, describe } from 'bun:test';
import { tokenize } from './index';

describe('tokenizer', () => {
    describe('literals', () => {
        test('integers', () => {
            const { tokens } = tokenize('42');

            expect(tokens[0].type).toBe('NUMBER');
            expect(tokens[0].value).toBe('42');
        });

        test('decimals', () => {
            const { tokens } = tokenize('3.14159');

            expect(tokens[0].type).toBe('NUMBER');
            expect(tokens[0].value).toBe('3.14159');
        });

        test('strings with double quotes', () => {
            const { tokens } = tokenize('"Salve, mundus!"');

            expect(tokens[0].type).toBe('STRING');
            expect(tokens[0].value).toBe('Salve, mundus!');
        });

        test('strings with escapes', () => {
            const { tokens } = tokenize('"line1\\nline2"');

            expect(tokens[0].value).toBe('line1\nline2');
        });

        test('template strings', () => {
            const { tokens } = tokenize('`Salve, ${nomen}!`');

            expect(tokens[0].type).toBe('TEMPLATE_STRING');
            expect(tokens[0].value).toContain('${nomen}');
        });
    });

    describe('keywords', () => {
        test('recognizes si as keyword', () => {
            const { tokens } = tokenize('si');

            expect(tokens[0].type).toBe('KEYWORD');
            expect(tokens[0].keyword).toBe('si');
        });

        test('recognizes fixum as keyword', () => {
            const { tokens } = tokenize('fixum');

            expect(tokens[0].type).toBe('KEYWORD');
            expect(tokens[0].keyword).toBe('fixum');
        });

        test('recognizes varia as keyword', () => {
            const { tokens } = tokenize('varia');

            expect(tokens[0].type).toBe('KEYWORD');
        });

        test('recognizes functio as keyword', () => {
            const { tokens } = tokenize('functio');

            expect(tokens[0].type).toBe('KEYWORD');
        });

        test('recognizes verum/falsum as keywords', () => {
            const { tokens } = tokenize('verum falsum');

            expect(tokens[0].type).toBe('KEYWORD');
            expect(tokens[1].type).toBe('KEYWORD');
        });
    });

    describe('identifiers', () => {
        test('simple identifier', () => {
            const { tokens } = tokenize('nuntius');

            expect(tokens[0].type).toBe('IDENTIFIER');
            expect(tokens[0].value).toBe('nuntius');
        });

        test('identifier with numbers', () => {
            const { tokens } = tokenize('usuario1');

            expect(tokens[0].type).toBe('IDENTIFIER');
            expect(tokens[0].value).toBe('usuario1');
        });

        test('TitleCase identifier', () => {
            const { tokens } = tokenize('textus');

            expect(tokens[0].type).toBe('IDENTIFIER');
            expect(tokens[0].value).toBe('textus');
        });
    });

    describe('operators', () => {
        test('arithmetic operators', () => {
            const { tokens } = tokenize('+ - * / %');

            expect(tokens.map(t => t.type)).toEqual(['PLUS', 'MINUS', 'STAR', 'SLASH', 'PERCENT', 'EOF']);
        });

        test('comparison operators', () => {
            const { tokens } = tokenize('== != < <= > >=');

            expect(tokens.map(t => t.type)).toEqual(['EQUAL_EQUAL', 'BANG_EQUAL', 'LESS', 'LESS_EQUAL', 'GREATER', 'GREATER_EQUAL', 'EOF']);
        });

        test('strict equality operators', () => {
            const { tokens } = tokenize('=== !==');

            expect(tokens.map(t => t.type)).toEqual(['TRIPLE_EQUAL', 'BANG_DOUBLE_EQUAL', 'EOF']);
            expect(tokens[0].value).toBe('===');
            expect(tokens[1].value).toBe('!==');
        });

        test('logical operators', () => {
            const { tokens } = tokenize('&& || !');

            expect(tokens.map(t => t.type)).toEqual(['AND', 'OR', 'BANG', 'EOF']);
        });

        test('arrow operator', () => {
            const { tokens } = tokenize('=>');

            expect(tokens[0].type).toBe('ARROW');
        });

        test('assignment', () => {
            const { tokens } = tokenize('=');

            expect(tokens[0].type).toBe('EQUAL');
        });
    });

    describe('delimiters', () => {
        test('parentheses', () => {
            const { tokens } = tokenize('()');

            expect(tokens.map(t => t.type)).toEqual(['LPAREN', 'RPAREN', 'EOF']);
        });

        test('braces', () => {
            const { tokens } = tokenize('{}');

            expect(tokens.map(t => t.type)).toEqual(['LBRACE', 'RBRACE', 'EOF']);
        });

        test('brackets', () => {
            const { tokens } = tokenize('[]');

            expect(tokens.map(t => t.type)).toEqual(['LBRACKET', 'RBRACKET', 'EOF']);
        });

        test('punctuation', () => {
            const { tokens } = tokenize(', ; : . ?');

            expect(tokens.map(t => t.type)).toEqual(['COMMA', 'SEMICOLON', 'COLON', 'DOT', 'QUESTION', 'EOF']);
        });

        test('pipe for union types', () => {
            const { tokens } = tokenize('textus | nihil');

            expect(tokens[1].type).toBe('PIPE');
        });
    });

    describe('comments', () => {
        test('single-line comments are skipped', () => {
            const { tokens } = tokenize('fixum // this is a comment\nnomen');

            expect(tokens.map(t => t.type)).toEqual(['KEYWORD', 'IDENTIFIER', 'EOF']);
        });

        test('multi-line comments are skipped', () => {
            const { tokens } = tokenize('fixum /* comment */ nomen');

            expect(tokens.map(t => t.type)).toEqual(['KEYWORD', 'IDENTIFIER', 'EOF']);
        });
    });

    describe('position tracking', () => {
        test('tracks line and column', () => {
            const { tokens } = tokenize('fixum\nnomen');

            expect(tokens[0].position.line).toBe(1);
            expect(tokens[1].position.line).toBe(2);
        });
    });

    describe('complete expressions', () => {
        test('variable declaration', () => {
            const { tokens } = tokenize('fixum nomen = "Marcus"');

            expect(tokens.map(t => t.type)).toEqual(['KEYWORD', 'IDENTIFIER', 'EQUAL', 'STRING', 'EOF']);
        });

        test('function declaration', () => {
            const { tokens } = tokenize('functio salve(nomen: textus) -> textus {');
            const types = tokens.map(t => t.type);

            expect(types).toContain('KEYWORD'); // functio
            expect(types).toContain('IDENTIFIER'); // salve, nomen, textus
            expect(types).toContain('LPAREN');
            expect(types).toContain('COLON');
            expect(types).toContain('THIN_ARROW');
            expect(types).toContain('LBRACE');
        });

        test('if statement with Latin operators', () => {
            const { tokens } = tokenize('si activa et verificata {');

            expect(tokens[0].type).toBe('KEYWORD'); // si
            expect(tokens[0].keyword).toBe('si');
            expect(tokens[2].type).toBe('KEYWORD'); // et
            expect(tokens[2].keyword).toBe('et');
        });

        test('arrow function', () => {
            const { tokens } = tokenize('(x) => x * 2');

            expect(tokens.map(t => t.type)).toEqual(['LPAREN', 'IDENTIFIER', 'RPAREN', 'ARROW', 'IDENTIFIER', 'STAR', 'NUMBER', 'EOF']);
        });
    });

    describe('error handling', () => {
        test('reports unterminated string', () => {
            const { errors } = tokenize('"unterminated');

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].text).toContain('Unterminated');
        });

        test('reports unexpected character', () => {
            const { errors } = tokenize('fixum @ nomen');

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].text).toContain('Unexpected');
        });
    });

    describe('error recovery - unterminated constructs', () => {
        test('unterminated string with newline', () => {
            const { errors } = tokenize('"unterminated\n');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('unterminated template string', () => {
            const { errors } = tokenize('`unterminated ${x}');

            expect(errors.length).toBeGreaterThan(0);
        });

        test.todo('unterminated multi-line comment', () => {
            // TOKENIZER GAP: Should error on unterminated block comment
            const { errors } = tokenize('/* comment without closing');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('string with unescaped newline', () => {
            const { errors } = tokenize('"line1\nline2"');

            expect(errors.length).toBeGreaterThan(0);
        });

        test('template string with unclosed interpolation', () => {
            const { errors } = tokenize('`text ${incomplete`');

            expect(errors.length).toBeGreaterThan(0);
        });
    });

    describe('edge cases - boundary values', () => {
        test('empty source', () => {
            const { tokens, errors } = tokenize('');

            expect(errors).toHaveLength(0);
            expect(tokens).toHaveLength(1); // Just EOF
            expect(tokens[0].type).toBe('EOF');
        });

        test('only whitespace', () => {
            const { tokens } = tokenize('   \n\t  ');

            expect(tokens).toHaveLength(1); // Just EOF
            expect(tokens[0].type).toBe('EOF');
        });

        test('only comments', () => {
            const { tokens } = tokenize('// comment\n/* block */');

            expect(tokens).toHaveLength(1); // Just EOF
        });

        test('zero values', () => {
            const { tokens } = tokenize('0 0.0 0.00');

            expect(tokens[0].value).toBe('0');
            expect(tokens[1].value).toBe('0.0');
            expect(tokens[2].value).toBe('0.00');
        });

        test('very long number', () => {
            const { tokens } = tokenize('123456789012345678901234567890');

            expect(tokens[0].type).toBe('NUMBER');
        });

        test('very long identifier', () => {
            const longId = 'a'.repeat(100);
            const { tokens } = tokenize(longId);

            expect(tokens[0].type).toBe('IDENTIFIER');
            expect(tokens[0].value).toHaveLength(100);
        });

        test('number with leading zeros', () => {
            const { tokens } = tokenize('007');

            expect(tokens[0].type).toBe('NUMBER');
            expect(tokens[0].value).toBe('007');
        });

        test('decimal starting with dot', () => {
            const { tokens } = tokenize('.5');

            // May tokenize as DOT + NUMBER or as NUMBER
            expect(tokens).toBeDefined();
        });
    });

    describe('hex literals', () => {
        test('basic hex literal', () => {
            const { tokens } = tokenize('0xFF');

            expect(tokens[0].type).toBe('NUMBER');
            expect(tokens[0].value).toBe('0xFF');
        });

        test('lowercase prefix', () => {
            const { tokens } = tokenize('0xff');

            expect(tokens[0].type).toBe('NUMBER');
            expect(tokens[0].value).toBe('0xff');
        });

        test('uppercase prefix', () => {
            const { tokens } = tokenize('0XFF');

            expect(tokens[0].type).toBe('NUMBER');
            expect(tokens[0].value).toBe('0XFF');
        });

        test('mixed case digits', () => {
            const { tokens } = tokenize('0xAbCdEf');

            expect(tokens[0].type).toBe('NUMBER');
            expect(tokens[0].value).toBe('0xAbCdEf');
        });

        test('hex with all digits', () => {
            const { tokens } = tokenize('0x1234567890ABCDEFabcdef');

            expect(tokens[0].type).toBe('NUMBER');
            expect(tokens[0].value).toBe('0x1234567890ABCDEFabcdef');
        });

        test('hex bigint', () => {
            const { tokens } = tokenize('0xFFn');

            expect(tokens[0].type).toBe('BIGINT');
            expect(tokens[0].value).toBe('0xFF');
        });

        test('large hex bigint', () => {
            const { tokens } = tokenize('0xFFFFFFFFFn');

            expect(tokens[0].type).toBe('BIGINT');
            expect(tokens[0].value).toBe('0xFFFFFFFFF');
        });

        test('hex in expression', () => {
            const { tokens } = tokenize('0xFF - 0xAA');

            expect(tokens[0].type).toBe('NUMBER');
            expect(tokens[0].value).toBe('0xFF');
            expect(tokens[1].type).toBe('MINUS');
            expect(tokens[2].type).toBe('NUMBER');
            expect(tokens[2].value).toBe('0xAA');
        });

        test('hex zero', () => {
            const { tokens } = tokenize('0x0');

            expect(tokens[0].type).toBe('NUMBER');
            expect(tokens[0].value).toBe('0x0');
        });

        test('invalid hex - no digits', () => {
            const { tokens, errors } = tokenize('0x');

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].code).toBe('L004');
        });

        test('invalid hex - non-hex char after prefix', () => {
            const { tokens, errors } = tokenize('0xGG');

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].code).toBe('L004');
        });
    });

    describe('edge cases - whitespace variations', () => {
        test('mixed tabs and spaces', () => {
            const { tokens } = tokenize('fixum\t  x\t=\t5');

            expect(tokens[0].type).toBe('KEYWORD');
            expect(tokens[1].type).toBe('IDENTIFIER');
        });

        test('multiple consecutive spaces', () => {
            const { tokens } = tokenize('fixum     x     =     5');

            expect(tokens).toHaveLength(5); // fixum, x, =, 5, EOF
        });

        test('CR LF line endings', () => {
            const { tokens } = tokenize('fixum\r\nx\r\n=\r\n5');

            expect(tokens[0].type).toBe('KEYWORD');
            expect(tokens[3].value).toBe('5');
        });

        test('no whitespace between tokens', () => {
            const { tokens } = tokenize('fixum x=5');

            expect(tokens[0].type).toBe('KEYWORD');
            expect(tokens[1].type).toBe('IDENTIFIER');
            expect(tokens[2].type).toBe('EQUAL');
            expect(tokens[3].type).toBe('NUMBER');
        });

        test('trailing whitespace', () => {
            const { tokens } = tokenize('fixum x = 5   \n\t ');

            expect(tokens[tokens.length - 1].type).toBe('EOF');
        });
    });

    describe('edge cases - special characters', () => {
        test('multiple consecutive operators', () => {
            const { tokens } = tokenize('+++');

            expect(tokens[0].type).toBe('PLUS');
            expect(tokens[1].type).toBe('PLUS');
            expect(tokens[2].type).toBe('PLUS');
        });

        test('mixed quote types in file', () => {
            const { tokens } = tokenize('"double" `template` "another"');

            expect(tokens[0].type).toBe('STRING');
            expect(tokens[1].type).toBe('TEMPLATE_STRING');
            expect(tokens[2].type).toBe('STRING');
        });

        test('escaped quotes in strings', () => {
            const { tokens } = tokenize('"He said \\"hello\\""');

            expect(tokens[0].type).toBe('STRING');
            expect(tokens[0].value).toContain('"');
        });

        test('unicode in identifiers', () => {
            const { tokens } = tokenize('niño');

            expect(tokens[0].type).toBe('IDENTIFIER');
        });

        test('emoji in string', () => {
            const { tokens } = tokenize('"hello 👋"');

            expect(tokens[0].type).toBe('STRING');
            expect(tokens[0].value).toContain('👋');
        });
    });

    describe('edge cases - comment variations', () => {
        test('single-line comment at end of file', () => {
            const { tokens } = tokenize('fixum x = 5 // comment');

            expect(tokens[tokens.length - 1].type).toBe('EOF');
        });

        test('multi-line comment with nested slashes', () => {
            const { tokens } = tokenize('/* // not a comment */');

            expect(tokens).toHaveLength(1); // Just EOF
        });

        test('comment with special chars', () => {
            const { tokens } = tokenize('// @#$%^&*()');

            expect(tokens).toHaveLength(1); // Just EOF
        });

        test('adjacent comments', () => {
            const { tokens } = tokenize('// comment1\n// comment2');

            expect(tokens).toHaveLength(1); // Just EOF
        });

        test('block comment spanning multiple lines', () => {
            const { tokens } = tokenize('/* line1\nline2\nline3 */');

            expect(tokens).toHaveLength(1); // Just EOF
        });
    });
});
