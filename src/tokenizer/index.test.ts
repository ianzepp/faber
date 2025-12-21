import { test, expect, describe } from "bun:test";
import { tokenize } from "./index";

describe("tokenizer", () => {
    describe("literals", () => {
        test("integers", () => {
            const { tokens } = tokenize("42");

            expect(tokens[0].type).toBe("NUMBER");
            expect(tokens[0].value).toBe("42");
        });

        test("decimals", () => {
            const { tokens } = tokenize("3.14159");

            expect(tokens[0].type).toBe("NUMBER");
            expect(tokens[0].value).toBe("3.14159");
        });

        test("strings with double quotes", () => {
            const { tokens } = tokenize('"Salve, mundus!"');

            expect(tokens[0].type).toBe("STRING");
            expect(tokens[0].value).toBe("Salve, mundus!");
        });

        test("strings with escapes", () => {
            const { tokens } = tokenize('"line1\\nline2"');

            expect(tokens[0].value).toBe("line1\nline2");
        });

        test("template strings", () => {
            const { tokens } = tokenize("`Salve, ${nomen}!`");

            expect(tokens[0].type).toBe("TEMPLATE_STRING");
            expect(tokens[0].value).toContain("${nomen}");
        });
    });

    describe("keywords", () => {
        test("recognizes si as keyword", () => {
            const { tokens } = tokenize("si");

            expect(tokens[0].type).toBe("KEYWORD");
            expect(tokens[0].keyword).toBe("si");
        });

        test("recognizes fixum as keyword", () => {
            const { tokens } = tokenize("fixum");

            expect(tokens[0].type).toBe("KEYWORD");
            expect(tokens[0].keyword).toBe("fixum");
        });

        test("recognizes esto as keyword", () => {
            const { tokens } = tokenize("esto");

            expect(tokens[0].type).toBe("KEYWORD");
        });

        test("recognizes functio as keyword", () => {
            const { tokens } = tokenize("functio");

            expect(tokens[0].type).toBe("KEYWORD");
        });

        test("recognizes verum/falsum as keywords", () => {
            const { tokens } = tokenize("verum falsum");

            expect(tokens[0].type).toBe("KEYWORD");
            expect(tokens[1].type).toBe("KEYWORD");
        });
    });

    describe("identifiers", () => {
        test("simple identifier", () => {
            const { tokens } = tokenize("nuntius");

            expect(tokens[0].type).toBe("IDENTIFIER");
            expect(tokens[0].value).toBe("nuntius");
        });

        test("identifier with numbers", () => {
            const { tokens } = tokenize("usuario1");

            expect(tokens[0].type).toBe("IDENTIFIER");
            expect(tokens[0].value).toBe("usuario1");
        });

        test("TitleCase identifier", () => {
            const { tokens } = tokenize("Textus");

            expect(tokens[0].type).toBe("IDENTIFIER");
            expect(tokens[0].value).toBe("Textus");
        });
    });

    describe("operators", () => {
        test("arithmetic operators", () => {
            const { tokens } = tokenize("+ - * / %");

            expect(tokens.map(t => t.type)).toEqual([
                "PLUS", "MINUS", "STAR", "SLASH", "PERCENT", "EOF",
            ]);
        });

        test("comparison operators", () => {
            const { tokens } = tokenize("== != < <= > >=");

            expect(tokens.map(t => t.type)).toEqual([
                "EQUAL_EQUAL", "BANG_EQUAL", "LESS", "LESS_EQUAL", "GREATER", "GREATER_EQUAL", "EOF",
            ]);
        });

        test("logical operators", () => {
            const { tokens } = tokenize("&& || !");

            expect(tokens.map(t => t.type)).toEqual([
                "AND", "OR", "BANG", "EOF",
            ]);
        });

        test("arrow operator", () => {
            const { tokens } = tokenize("=>");

            expect(tokens[0].type).toBe("ARROW");
        });

        test("assignment", () => {
            const { tokens } = tokenize("=");

            expect(tokens[0].type).toBe("EQUAL");
        });
    });

    describe("delimiters", () => {
        test("parentheses", () => {
            const { tokens } = tokenize("()");

            expect(tokens.map(t => t.type)).toEqual(["LPAREN", "RPAREN", "EOF"]);
        });

        test("braces", () => {
            const { tokens } = tokenize("{}");

            expect(tokens.map(t => t.type)).toEqual(["LBRACE", "RBRACE", "EOF"]);
        });

        test("brackets", () => {
            const { tokens } = tokenize("[]");

            expect(tokens.map(t => t.type)).toEqual(["LBRACKET", "RBRACKET", "EOF"]);
        });

        test("punctuation", () => {
            const { tokens } = tokenize(", ; : . ?");

            expect(tokens.map(t => t.type)).toEqual([
                "COMMA", "SEMICOLON", "COLON", "DOT", "QUESTION", "EOF",
            ]);
        });

        test("pipe for union types", () => {
            const { tokens } = tokenize("Textus | Nihil");

            expect(tokens[1].type).toBe("PIPE");
        });
    });

    describe("comments", () => {
        test("single-line comments are skipped", () => {
            const { tokens } = tokenize("fixum // this is a comment\nnomen");

            expect(tokens.map(t => t.type)).toEqual(["KEYWORD", "IDENTIFIER", "EOF"]);
        });

        test("multi-line comments are skipped", () => {
            const { tokens } = tokenize("fixum /* comment */ nomen");

            expect(tokens.map(t => t.type)).toEqual(["KEYWORD", "IDENTIFIER", "EOF"]);
        });
    });

    describe("position tracking", () => {
        test("tracks line and column", () => {
            const { tokens } = tokenize("fixum\nnomen");

            expect(tokens[0].position.line).toBe(1);
            expect(tokens[1].position.line).toBe(2);
        });
    });

    describe("complete expressions", () => {
        test("variable declaration", () => {
            const { tokens } = tokenize('fixum nomen = "Marcus"');

            expect(tokens.map(t => t.type)).toEqual([
                "KEYWORD", "IDENTIFIER", "EQUAL", "STRING", "EOF",
            ]);
        });

        test("function declaration", () => {
            const { tokens } = tokenize("functio salve(nomen: Textus) -> Textus {");
            const types = tokens.map(t => t.type);

            expect(types).toContain("KEYWORD");  // functio
            expect(types).toContain("IDENTIFIER");  // salve, nomen, Textus
            expect(types).toContain("LPAREN");
            expect(types).toContain("COLON");
            expect(types).toContain("THIN_ARROW");
            expect(types).toContain("LBRACE");
        });

        test("if statement with Latin operators", () => {
            const { tokens } = tokenize("si activa et verificata {");

            expect(tokens[0].type).toBe("KEYWORD");  // si
            expect(tokens[0].keyword).toBe("si");
            expect(tokens[2].type).toBe("KEYWORD");  // et
            expect(tokens[2].keyword).toBe("et");
        });

        test("arrow function", () => {
            const { tokens } = tokenize("(x) => x * 2");

            expect(tokens.map(t => t.type)).toEqual([
                "LPAREN", "IDENTIFIER", "RPAREN", "ARROW", "IDENTIFIER", "STAR", "NUMBER", "EOF",
            ]);
        });
    });

    describe("error handling", () => {
        test("reports unterminated string", () => {
            const { errors } = tokenize('"unterminated');

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain("Unterminated");
        });

        test("reports unexpected character", () => {
            const { errors } = tokenize("fixum @ nomen");

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain("Unexpected");
        });
    });
});
