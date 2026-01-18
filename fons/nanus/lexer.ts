/**
 * NANUS - Minimal Faber Compiler
 *
 * Lexer: source text → token array
 * Handwritten single-pass scanner. No regex, no generators.
 *
 * Supports the subset of Faber tokens needed to compile rivus.
 */

import type { Token, TokenTag, Locus } from './ast';

const KEYWORDS = new Set([
    // Declarations
    'varia', 'fixum', 'figendum',
    'functio', 'genus', 'pactum', 'ordo', 'discretio',
    'ex', 'importa', 'ut',
    // Modifiers
    'publica', 'privata', 'protecta', 'asynca', 'generis', 'implet', 'sub',
    // Control flow
    'si', 'secus', 'dum', 'fac', 'elige', 'casu', 'discerne', 'custodi',
    'de', 'pro', 'omnia',
    // Actions
    'redde', 'rumpe', 'perge', 'iace', 'mori', 'tempta', 'cape', 'demum',
    'scribe', 'vide', 'mone', 'adfirma',
    // Expressions
    'cede', 'novum', 'clausura', 'qua', 'innatum', 'finge',
    'sic', 'scriptum',
    // Operators (word-form)
    'et', 'aut', 'vel', 'inter', 'intra',
    'nihil', 'nonnihil', 'positivum',
    // Literals
    'verum', 'falsum',
    // Entry
    'incipit',
    // Test
    'probandum', 'proba',
    // Type
    'usque',
    // Annotations
    'publicum',
]);

const PUNCTUATORS = new Set([
    '(', ')', '{', '}', '[', ']',
    ',', '.', ':', ';', '@', '#', '§',
    '?', '!',
]);

const OPERATORS = [
    // Multi-char first (greedy match)
    '===', '!==', '==', '!=', '<=', '>=', '&&', '||', '??',
    '+=', '-=', '*=', '/=',
    '->', '..',
    // Single-char
    '+', '-', '*', '/', '%',
    '<', '>', '=',
    '&', '|', '^', '~',
];

export function lex(source: string, filename = '<stdin>'): Token[] {
    const tokens: Token[] = [];
    let pos = 0;
    let linea = 1;
    let lineStart = 0;

    function locus(): Locus {
        return { linea, columna: pos - lineStart + 1, index: pos };
    }

    function peek(offset = 0): string {
        return source[pos + offset] ?? '';
    }

    function advance(): string {
        const ch = source[pos++];
        if (ch === '\n') {
            linea++;
            lineStart = pos;
        }
        return ch;
    }

    function match(str: string): boolean {
        if (source.slice(pos, pos + str.length) === str) {
            for (let i = 0; i < str.length; i++) advance();
            return true;
        }
        return false;
    }

    function skipWhitespace(): void {
        while (pos < source.length) {
            const ch = peek();
            if (ch === ' ' || ch === '\t' || ch === '\r') {
                advance();
            } else if (ch === '\n') {
                const loc = locus();
                advance();
                tokens.push({ tag: 'Newline', valor: '\n', locus: loc });
            } else {
                break;
            }
        }
    }

    function readString(quote: string): string {
        let value = '';
        advance(); // opening quote
        while (pos < source.length && peek() !== quote) {
            if (peek() === '\\') {
                advance();
                const esc = advance();
                switch (esc) {
                    case 'n': value += '\n'; break;
                    case 't': value += '\t'; break;
                    case 'r': value += '\r'; break;
                    case '\\': value += '\\'; break;
                    case '"': value += '"'; break;
                    case "'": value += "'"; break;
                    default: value += esc;
                }
            } else {
                value += advance();
            }
        }
        advance(); // closing quote
        return value;
    }

    function readNumber(): string {
        let value = '';
        while (pos < source.length && /[0-9._]/.test(peek())) {
            value += advance();
        }
        return value;
    }

    function readIdentifier(): string {
        let value = '';
        while (pos < source.length && /[a-zA-Z0-9_]/.test(peek())) {
            value += advance();
        }
        return value;
    }

    function readComment(): string {
        let value = '';
        advance(); // skip #
        while (pos < source.length && peek() !== '\n') {
            value += advance();
        }
        return value;
    }

    while (pos < source.length) {
        skipWhitespace();
        if (pos >= source.length) break;

        const loc = locus();
        const ch = peek();

        // Comments
        if (ch === '#') {
            const value = readComment();
            tokens.push({ tag: 'Comment', valor: value, locus: loc });
            continue;
        }

        // Strings
        if (ch === '"' || ch === "'") {
            const value = readString(ch);
            tokens.push({ tag: 'Textus', valor: value, locus: loc });
            continue;
        }

        // Numbers
        if (/[0-9]/.test(ch)) {
            const value = readNumber();
            tokens.push({ tag: 'Numerus', valor: value, locus: loc });
            continue;
        }

        // Identifiers and keywords
        if (/[a-zA-Z_]/.test(ch)) {
            const value = readIdentifier();
            const tag: TokenTag = KEYWORDS.has(value) ? 'Keyword' : 'Identifier';
            tokens.push({ tag, valor: value, locus: loc });
            continue;
        }

        // Operators (try longest match first)
        let matched = false;
        for (const op of OPERATORS) {
            if (match(op)) {
                tokens.push({ tag: 'Operator', valor: op, locus: loc });
                matched = true;
                break;
            }
        }
        if (matched) continue;

        // Punctuators
        if (PUNCTUATORS.has(ch)) {
            advance();
            tokens.push({ tag: 'Punctuator', valor: ch, locus: loc });
            continue;
        }

        // Unknown character - skip with warning
        console.error(`${filename}:${loc.linea}:${loc.columna}: unexpected character '${ch}'`);
        advance();
    }

    tokens.push({ tag: 'EOF', valor: '', locus: locus() });
    return tokens;
}

// Filter out comments and collapse multiple newlines
export function prepare(tokens: Token[]): Token[] {
    const result: Token[] = [];
    let lastWasNewline = true; // start as if preceded by newline

    for (const tok of tokens) {
        if (tok.tag === 'Comment') continue;
        if (tok.tag === 'Newline') {
            if (!lastWasNewline) {
                result.push(tok);
                lastWasNewline = true;
            }
        } else {
            result.push(tok);
            lastWasNewline = false;
        }
    }

    return result;
}
