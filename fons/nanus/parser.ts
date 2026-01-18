/**
 * NANUS - Minimal Faber Compiler
 *
 * Parser: tokens → AST
 * Pratt parser for expressions, recursive descent for statements.
 *
 * Supports the subset of Faber syntax needed to compile rivus.
 */

import type {
    Token, Locus, Typus, Expr, Stmt, Param, ObiectumProp,
    LitteraSpecies, VariaSpecies, CampusDecl, PactumMethodus,
    OrdoMembrum, VariansDecl, ImportSpec, EligeCasus, DiscerneCasus,
    VariansPattern, CustodiClausula, CapeClausula, Modulus,
} from './ast';

// Operator precedence for Pratt parser
const PRECEDENCE: Record<string, number> = {
    '=': 1, '+=': 1, '-=': 1, '*=': 1, '/=': 1,
    'vel': 2, '??': 2,
    'aut': 3, '||': 3,
    'et': 4, '&&': 4,
    '==': 5, '!=': 5, '===': 5, '!==': 5,
    '<': 6, '>': 6, '<=': 6, '>=': 6, 'inter': 6, 'intra': 6,
    '+': 7, '-': 7,
    '*': 8, '/': 8, '%': 8,
    'qua': 9, 'innatum': 9,
};

const UNARY_OPS = new Set(['-', '!', 'nihil', 'nonnihil', 'positivum']);
const ASSIGN_OPS = new Set(['=', '+=', '-=', '*=', '/=']);

export class Parser {
    private tokens: Token[];
    private pos = 0;

    constructor(tokens: Token[]) {
        this.tokens = tokens;
    }

    // Token access
    private peek(offset = 0): Token {
        return this.tokens[this.pos + offset] ?? this.tokens[this.tokens.length - 1];
    }

    private advance(): Token {
        return this.tokens[this.pos++];
    }

    private check(tag: string, valor?: string): boolean {
        const tok = this.peek();
        if (tok.tag !== tag) return false;
        if (valor !== undefined && tok.valor !== valor) return false;
        return true;
    }

    private match(tag: string, valor?: string): Token | null {
        if (this.check(tag, valor)) return this.advance();
        return null;
    }

    private expect(tag: string, valor?: string): Token {
        const tok = this.match(tag, valor);
        if (!tok) {
            const got = this.peek();
            throw this.error(`expected ${valor ?? tag}, got '${got.valor}'`);
        }
        return tok;
    }

    private error(msg: string): Error {
        const loc = this.peek().locus;
        return new Error(`${loc.linea}:${loc.columna}: ${msg}`);
    }

    private skipNewlines(): void {
        while (this.match('Newline')) {}
    }

    private expectNewline(): void {
        if (!this.check('EOF') && !this.check('Punctuator', '}')) {
            this.expect('Newline');
        }
        this.skipNewlines();
    }

    // Main entry point
    parse(): Modulus {
        this.skipNewlines();
        const corpus: Stmt[] = [];
        while (!this.check('EOF')) {
            corpus.push(this.parseStmt());
            this.skipNewlines();
        }
        return { locus: { linea: 1, columna: 1, index: 0 }, corpus };
    }

    // Statements
    private parseStmt(): Stmt {
        this.skipNewlines();

        // Annotations
        let publica = false;
        while (this.match('Punctuator', '@')) {
            // Annotation name can be identifier or keyword
            const tok = this.peek();
            if (tok.tag !== 'Identifier' && tok.tag !== 'Keyword') {
                throw this.error('expected annotation name');
            }
            const anno = this.advance().valor;
            if (anno === 'publicum' || anno === 'publica') publica = true;
            this.skipNewlines();
        }

        // Section import: § ex "path" importa ...
        if (this.match('Punctuator', '§')) {
            return this.parseImport();
        }

        const tok = this.peek();

        // Keywords
        if (tok.tag === 'Keyword') {
            switch (tok.valor) {
                case 'varia':
                case 'fixum':
                case 'figendum':
                    return this.parseVaria(publica);
                case 'ex':
                    return this.parseExStmt(publica);
                case 'functio':
                    return this.parseFunctio(publica);
                case 'genus':
                    return this.parseGenus(publica);
                case 'pactum':
                    return this.parsePactum(publica);
                case 'ordo':
                    return this.parseOrdo(publica);
                case 'discretio':
                    return this.parseDiscretio(publica);
                case 'si':
                    return this.parseSi();
                case 'dum':
                    return this.parseDum();
                case 'fac':
                    return this.parseFac();
                case 'elige':
                    return this.parseElige();
                case 'discerne':
                    return this.parseDiscerne();
                case 'custodi':
                    return this.parseCustodi();
                case 'tempta':
                    return this.parseTempta();
                case 'redde':
                    return this.parseRedde();
                case 'iace':
                case 'mori':
                    return this.parseIace();
                case 'scribe':
                case 'vide':
                case 'mone':
                    return this.parseScribe();
                case 'adfirma':
                    return this.parseAdfirma();
                case 'rumpe':
                    return this.parseRumpe();
                case 'perge':
                    return this.parsePerge();
                case 'incipit':
                    return this.parseIncipit();
                case 'probandum':
                    return this.parseProbandum();
                case 'proba':
                    return this.parseProba();
            }
        }

        // Block
        if (this.check('Punctuator', '{')) {
            return this.parseMassa();
        }

        // Expression statement
        return this.parseExpressiaStmt();
    }

    private parseImport(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'ex');
        const fons = this.expect('Textus').valor;
        this.expect('Keyword', 'importa');

        const specs: ImportSpec[] = [];
        do {
            const loc = this.peek().locus;
            const imported = this.expect('Identifier').valor;
            let local = imported;
            if (this.match('Keyword', 'ut')) {
                local = this.expect('Identifier').valor;
            }
            specs.push({ locus: loc, imported, local });
        } while (this.match('Punctuator', ','));

        this.expectNewline();
        return { tag: 'Importa', locus, fons, specs, totum: false, alias: null };
    }

    private parseVaria(publica: boolean): Stmt {
        const locus = this.peek().locus;
        const kw = this.advance().valor;
        const species: VariaSpecies = kw === 'varia' ? 'Varia' : kw === 'figendum' ? 'Figendum' : 'Fixum';
        const nomen = this.expect('Identifier').valor;
        let typus: Typus | null = null;
        if (this.match('Punctuator', ':')) {
            typus = this.parseTypus();
        }
        let valor: Expr | null = null;
        if (this.match('Operator', '=')) {
            valor = this.parseExpr();
        }
        this.expectNewline();
        return { tag: 'Varia', locus, species, nomen, typus, valor, publica };
    }

    private parseExStmt(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'ex');

        // ex items fixum item { ... } — iteration
        // Check if this is iteration: ex <expr> fixum/varia <ident> { ... }
        const expr = this.parseExpr();

        if (this.check('Keyword', 'fixum') || this.check('Keyword', 'varia')) {
            const species: 'Ex' | 'De' = 'Ex';
            const asynca = false; // TODO: handle cede
            this.advance(); // fixum/varia
            const binding = this.expect('Identifier').valor;
            const corpus = this.parseMassa();
            return { tag: 'Iteratio', locus, species, binding, iter: expr, corpus, asynca };
        }

        // ex obj fixum a, b — destructuring (not implemented in nanus)
        throw this.error('destructuring not supported in nanus');
    }

    private parseFunctio(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'functio');
        let asynca = false;
        if (this.match('Keyword', 'asynca')) asynca = true;

        const nomen = this.expect('Identifier').valor;

        // Optional generics
        const generics: string[] = [];
        if (this.match('Operator', '<')) {
            do {
                generics.push(this.expect('Identifier').valor);
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
        }

        this.expect('Punctuator', '(');
        const params = this.parseParams();
        this.expect('Punctuator', ')');

        let typusReditus: Typus | null = null;
        if (this.match('Operator', '->')) {
            typusReditus = this.parseTypus();
        }

        let corpus: Stmt | null = null;
        this.skipNewlines();
        if (this.check('Punctuator', '{')) {
            corpus = this.parseMassa();
        } else {
            this.expectNewline();
        }

        return { tag: 'Functio', locus, nomen, params, typusReditus, corpus, asynca, publica, generics };
    }

    private parseParams(): Param[] {
        const params: Param[] = [];
        this.skipNewlines();
        if (this.check('Punctuator', ')')) return params;

        do {
            this.skipNewlines();
            const locus = this.peek().locus;
            let rest = false;
            if (this.match('Keyword', 'ceteri')) rest = true;

            let typus: Typus | null = null;
            let nomen: string;

            if (this.check('Identifier')) {
                const first = this.advance().valor;

                // Check for generic type: Type<T>
                if (this.match('Operator', '<')) {
                    const args: Typus[] = [];
                    do {
                        args.push(this.parseTypus());
                    } while (this.match('Punctuator', ','));
                    this.expect('Operator', '>');
                    typus = { tag: 'Genericus', nomen: first, args };

                    // Check for nullable: Type<T>?
                    if (this.match('Punctuator', '?')) {
                        typus = { tag: 'Nullabilis', inner: typus };
                    }

                    nomen = this.expect('Identifier').valor;
                } else if (this.match('Punctuator', '?')) {
                    // Nullable type: Type?
                    typus = { tag: 'Nullabilis', inner: { tag: 'Nomen', nomen: first } };
                    nomen = this.expect('Identifier').valor;
                } else if (this.check('Identifier')) {
                    // "Type name" pattern
                    typus = { tag: 'Nomen', nomen: first };
                    nomen = this.advance().valor;
                } else if (this.match('Punctuator', ':')) {
                    // "name: Type" pattern
                    nomen = first;
                    typus = this.parseTypus();
                } else {
                    // Just a name
                    nomen = first;
                }
            } else {
                throw this.error('expected parameter name');
            }

            let default_: Expr | null = null;
            if (this.match('Operator', '=')) {
                default_ = this.parseExpr();
            }

            params.push({ locus, nomen, typus, default_, rest });
            this.skipNewlines();
        } while (this.match('Punctuator', ','));

        return params;
    }

    private parseGenus(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'genus');
        const nomen = this.expect('Identifier').valor;

        const generics: string[] = [];
        if (this.match('Operator', '<')) {
            do {
                generics.push(this.expect('Identifier').valor);
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
        }

        const implet: string[] = [];
        if (this.match('Keyword', 'implet')) {
            do {
                implet.push(this.expect('Identifier').valor);
            } while (this.match('Punctuator', ','));
        }

        this.expect('Punctuator', '{');
        this.skipNewlines();

        const campi: CampusDecl[] = [];
        const methodi: Stmt[] = [];

        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            // Handle annotations in class body
            while (this.match('Punctuator', '@')) {
                const tok = this.peek();
                if (tok.tag !== 'Identifier' && tok.tag !== 'Keyword') {
                    throw this.error('expected annotation name');
                }
                this.advance(); // skip annotation name
                this.skipNewlines();
            }

            // Check for visibility keyword
            let visibilitas: 'Publica' | 'Privata' | 'Protecta' = 'Publica';
            if (this.match('Keyword', 'privata') || this.match('Keyword', 'privatus')) {
                visibilitas = 'Privata';
            } else if (this.match('Keyword', 'protecta') || this.match('Keyword', 'protectus')) {
                visibilitas = 'Protecta';
            }

            if (this.check('Keyword', 'functio')) {
                methodi.push(this.parseFunctio(false));
            } else {
                // Field: Typus nomen, Typus<T> nomen, Typus? nomen, or nomen: Typus
                const loc = this.peek().locus;
                const first = this.expect('Identifier').valor;
                let fieldTypus: Typus;
                let fieldNomen: string;

                // Check for generic: Typus<...>
                if (this.match('Operator', '<')) {
                    const args: Typus[] = [];
                    do {
                        args.push(this.parseTypus());
                    } while (this.match('Punctuator', ','));
                    this.expect('Operator', '>');
                    fieldTypus = { tag: 'Genericus', nomen: first, args };

                    // Check for nullable after generic: Typus<T>?
                    if (this.match('Punctuator', '?')) {
                        fieldTypus = { tag: 'Nullabilis', inner: fieldTypus };
                    }

                    fieldNomen = this.expect('Identifier').valor;
                } else {
                    // Check for nullable: Typus?
                    let nullable = false;
                    if (this.match('Punctuator', '?')) {
                        nullable = true;
                    }

                    if (this.check('Identifier')) {
                        // "Typus nomen" or "Typus? nomen" pattern
                        fieldTypus = { tag: 'Nomen', nomen: first };
                        if (nullable) {
                            fieldTypus = { tag: 'Nullabilis', inner: fieldTypus };
                        }
                        fieldNomen = this.advance().valor;
                    } else if (this.match('Punctuator', ':')) {
                        fieldNomen = first;
                        fieldTypus = this.parseTypus();
                    } else {
                        throw this.error('expected field type or name');
                    }
                }

                let valor: Expr | null = null;
                if (this.match('Operator', '=')) {
                    valor = this.parseExpr();
                }

                campi.push({ locus: loc, nomen: fieldNomen, typus: fieldTypus, valor, visibilitas });
                this.expectNewline();
            }
            this.skipNewlines();
        }

        this.expect('Punctuator', '}');
        return { tag: 'Genus', locus, nomen, campi, methodi, implet, generics, publica };
    }

    private parsePactum(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'pactum');
        const nomen = this.expect('Identifier').valor;

        const generics: string[] = [];
        if (this.match('Operator', '<')) {
            do {
                generics.push(this.expect('Identifier').valor);
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
        }

        this.expect('Punctuator', '{');
        this.skipNewlines();

        const methodi: PactumMethodus[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            const loc = this.peek().locus;
            this.expect('Keyword', 'functio');
            let asynca = false;
            if (this.match('Keyword', 'asynca')) asynca = true;
            const name = this.expect('Identifier').valor;
            this.expect('Punctuator', '(');
            const params = this.parseParams();
            this.expect('Punctuator', ')');
            let typusReditus: Typus | null = null;
            if (this.match('Operator', '->')) {
                typusReditus = this.parseTypus();
            }
            methodi.push({ locus: loc, nomen: name, params, typusReditus, asynca });
            this.expectNewline();
            this.skipNewlines();
        }

        this.expect('Punctuator', '}');
        return { tag: 'Pactum', locus, nomen, methodi, generics, publica };
    }

    private parseOrdo(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'ordo');
        const nomen = this.expect('Identifier').valor;
        this.expect('Punctuator', '{');
        this.skipNewlines();

        const membra: OrdoMembrum[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            const loc = this.peek().locus;
            const name = this.expect('Identifier').valor;
            let valor: string | null = null;
            if (this.match('Operator', '=')) {
                valor = this.peek().valor;
                this.advance(); // number or string
            }
            membra.push({ locus: loc, nomen: name, valor });
            this.skipNewlines();
        }

        this.expect('Punctuator', '}');
        return { tag: 'Ordo', locus, nomen, membra, publica };
    }

    private parseDiscretio(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'discretio');
        const nomen = this.expect('Identifier').valor;

        const generics: string[] = [];
        if (this.match('Operator', '<')) {
            do {
                generics.push(this.expect('Identifier').valor);
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
        }

        this.expect('Punctuator', '{');
        this.skipNewlines();

        const variantes: VariansDecl[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            const loc = this.peek().locus;
            const name = this.expect('Identifier').valor;
            const campi: { nomen: string; typus: Typus }[] = [];

            if (this.match('Punctuator', '{')) {
                this.skipNewlines();
                while (!this.check('Punctuator', '}') && !this.check('EOF')) {
                    // Typus nomen pattern
                    const typNomen = this.expect('Identifier').valor;
                    const fieldTypus: Typus = { tag: 'Nomen', nomen: typNomen };
                    const fieldNomen = this.expect('Identifier').valor;
                    campi.push({ nomen: fieldNomen, typus: fieldTypus });
                    this.skipNewlines();
                }
                this.expect('Punctuator', '}');
            }

            variantes.push({ locus: loc, nomen: name, campi });
            this.skipNewlines();
        }

        this.expect('Punctuator', '}');
        return { tag: 'Discretio', locus, nomen, variantes, generics, publica };
    }

    private parseMassa(): Stmt {
        const locus = this.peek().locus;
        this.expect('Punctuator', '{');
        this.skipNewlines();
        const corpus: Stmt[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            corpus.push(this.parseStmt());
            this.skipNewlines();
        }
        this.expect('Punctuator', '}');
        return { tag: 'Massa', locus, corpus };
    }

    private parseSi(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'si');
        const cond = this.parseExpr();
        const cons = this.parseMassa();
        let alt: Stmt | null = null;
        this.skipNewlines();
        if (this.match('Keyword', 'secus')) {
            this.skipNewlines();
            if (this.check('Keyword', 'si')) {
                alt = this.parseSi();
            } else {
                alt = this.parseMassa();
            }
        }
        return { tag: 'Si', locus, cond, cons, alt };
    }

    private parseDum(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'dum');
        const cond = this.parseExpr();
        const corpus = this.parseMassa();
        return { tag: 'Dum', locus, cond, corpus };
    }

    private parseFac(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'fac');
        const corpus = this.parseMassa();
        this.expect('Keyword', 'dum');
        const cond = this.parseExpr();
        this.expectNewline();
        return { tag: 'FacDum', locus, corpus, cond };
    }

    private parseElige(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'elige');
        const discrim = this.parseExpr();
        this.expect('Punctuator', '{');
        this.skipNewlines();

        const casus: EligeCasus[] = [];
        let default_: Stmt | null = null;

        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            if (this.match('Keyword', 'ceterum')) {
                default_ = this.parseMassa();
            } else {
                this.expect('Keyword', 'si');
                const loc = this.peek().locus;
                const cond = this.parseExpr();
                const corpus = this.parseMassa();
                casus.push({ locus: loc, cond, corpus });
            }
            this.skipNewlines();
        }

        this.expect('Punctuator', '}');
        return { tag: 'Elige', locus, discrim, casus, default_ };
    }

    private parseDiscerne(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'discerne');
        const discrim: Expr[] = [this.parseExpr()];
        while (this.match('Punctuator', ',')) {
            discrim.push(this.parseExpr());
        }
        this.expect('Punctuator', '{');
        this.skipNewlines();

        const casus: DiscerneCasus[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            const loc = this.peek().locus;
            this.expect('Keyword', 'casu');
            const patterns: VariansPattern[] = [];

            // Parse pattern(s)
            do {
                const pLoc = this.peek().locus;
                const variant = this.expect('Identifier').valor;
                let alias: string | null = null;
                const bindings: string[] = [];
                const wildcard = variant === '_';

                if (this.match('Keyword', 'ut')) {
                    alias = this.expect('Identifier').valor;
                } else if (this.match('Keyword', 'pro') || this.match('Keyword', 'fixum')) {
                    do {
                        bindings.push(this.expect('Identifier').valor);
                    } while (this.match('Punctuator', ','));
                }

                patterns.push({ locus: pLoc, variant, bindings, alias, wildcard });
            } while (this.match('Punctuator', ','));

            const corpus = this.parseMassa();
            casus.push({ locus: loc, patterns, corpus });
            this.skipNewlines();
        }

        this.expect('Punctuator', '}');
        return { tag: 'Discerne', locus, discrim, casus };
    }

    private parseCustodi(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'custodi');
        this.expect('Punctuator', '{');
        this.skipNewlines();

        const clausulae: CustodiClausula[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            const loc = this.peek().locus;
            this.expect('Keyword', 'si');
            const cond = this.parseExpr();
            const corpus = this.parseMassa();
            clausulae.push({ locus: loc, cond, corpus });
            this.skipNewlines();
        }

        this.expect('Punctuator', '}');
        return { tag: 'Custodi', locus, clausulae };
    }

    private parseTempta(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'tempta');
        const corpus = this.parseMassa();

        let cape: CapeClausula | null = null;
        this.skipNewlines();
        if (this.match('Keyword', 'cape')) {
            const loc = this.peek().locus;
            const param = this.expect('Identifier').valor;
            const body = this.parseMassa();
            cape = { locus: loc, param, corpus: body };
        }

        let demum: Stmt | null = null;
        this.skipNewlines();
        if (this.match('Keyword', 'demum')) {
            demum = this.parseMassa();
        }

        return { tag: 'Tempta', locus, corpus, cape, demum };
    }

    private parseRedde(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'redde');
        let valor: Expr | null = null;
        if (!this.check('Newline') && !this.check('EOF') && !this.check('Punctuator', '}')) {
            valor = this.parseExpr();
        }
        this.expectNewline();
        return { tag: 'Redde', locus, valor };
    }

    private parseIace(): Stmt {
        const locus = this.peek().locus;
        const fatale = this.advance().valor === 'mori';
        const arg = this.parseExpr();
        this.expectNewline();
        return { tag: 'Iace', locus, arg, fatale };
    }

    private parseScribe(): Stmt {
        const locus = this.peek().locus;
        const kw = this.advance().valor;
        const gradus = kw === 'vide' ? 'Vide' : kw === 'mone' ? 'Mone' : 'Scribe';
        const args: Expr[] = [];
        if (!this.check('Newline') && !this.check('EOF')) {
            do {
                args.push(this.parseExpr());
            } while (this.match('Punctuator', ','));
        }
        this.expectNewline();
        return { tag: 'Scribe', locus, gradus, args };
    }

    private parseAdfirma(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'adfirma');
        const cond = this.parseExpr();
        let msg: Expr | null = null;
        if (this.match('Punctuator', ',')) {
            msg = this.parseExpr();
        }
        this.expectNewline();
        return { tag: 'Adfirma', locus, cond, msg };
    }

    private parseRumpe(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'rumpe');
        this.expectNewline();
        return { tag: 'Rumpe', locus };
    }

    private parsePerge(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'perge');
        this.expectNewline();
        return { tag: 'Perge', locus };
    }

    private parseIncipit(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'incipit');
        const corpus = this.parseMassa();
        return { tag: 'Incipit', locus, corpus };
    }

    private parseProbandum(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'probandum');
        const nomen = this.expect('Textus').valor;
        this.expect('Punctuator', '{');
        this.skipNewlines();

        const corpus: Stmt[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            corpus.push(this.parseStmt());
            this.skipNewlines();
        }

        this.expect('Punctuator', '}');
        return { tag: 'Probandum', locus, nomen, corpus };
    }

    private parseProba(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'proba');
        const nomen = this.expect('Textus').valor;
        const corpus = this.parseMassa();
        return { tag: 'Proba', locus, nomen, corpus };
    }

    private parseExpressiaStmt(): Stmt {
        const locus = this.peek().locus;
        const expr = this.parseExpr();
        this.expectNewline();
        return { tag: 'Expressia', locus, expr };
    }

    // Types
    private parseTypus(): Typus {
        let typus = this.parseTypusPrimary();

        // Nullable: T?
        if (this.match('Punctuator', '?')) {
            typus = { tag: 'Nullabilis', inner: typus };
        }

        // Union: T | U
        if (this.match('Operator', '|')) {
            const members: Typus[] = [typus];
            do {
                members.push(this.parseTypusPrimary());
            } while (this.match('Operator', '|'));
            typus = { tag: 'Unio', members };
        }

        return typus;
    }

    private parseTypusPrimary(): Typus {
        const nomen = this.expect('Identifier').valor;

        // Generics: T<U, V>
        if (this.match('Operator', '<')) {
            const args: Typus[] = [];
            do {
                args.push(this.parseTypus());
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
            return { tag: 'Genericus', nomen, args };
        }

        return { tag: 'Nomen', nomen };
    }

    // Expressions - Pratt parser
    private parseExpr(minPrec = 0): Expr {
        let left = this.parseUnary();

        while (true) {
            const tok = this.peek();
            const op = tok.valor;
            const prec = PRECEDENCE[op];

            if (prec === undefined || prec < minPrec) break;

            this.advance();

            // Handle qua/innatum specially (postfix type operators)
            if (op === 'qua') {
                const typus = this.parseTypus();
                left = { tag: 'Qua', locus: tok.locus, expr: left, typus };
                continue;
            }
            if (op === 'innatum') {
                const typus = this.parseTypus();
                left = { tag: 'Innatum', locus: tok.locus, expr: left, typus };
                continue;
            }

            const right = this.parseExpr(prec + 1);

            if (ASSIGN_OPS.has(op)) {
                left = { tag: 'Assignatio', locus: tok.locus, signum: op, sin: left, dex: right };
            } else {
                left = { tag: 'Binaria', locus: tok.locus, signum: op, sin: left, dex: right };
            }
        }

        // Ternary: cond sic cons secus alt
        if (this.match('Keyword', 'sic')) {
            const cons = this.parseExpr();
            this.expect('Keyword', 'secus');
            const alt = this.parseExpr();
            left = { tag: 'Condicio', locus: left.locus, cond: left, cons, alt };
        }

        return left;
    }

    private parseUnary(): Expr {
        const tok = this.peek();

        // Unary operators - but 'nihil' alone is a literal, not unary
        if (UNARY_OPS.has(tok.valor)) {
            // Check if followed by expression (identifier, number, paren, etc.)
            const next = this.tokens[this.pos + 1];
            const canBeUnary = next && (
                next.tag === 'Identifier' ||
                next.tag === 'Numerus' ||
                next.tag === 'Textus' ||
                next.valor === '(' ||
                next.valor === '[' ||
                next.valor === '{' ||
                UNARY_OPS.has(next.valor)
            );

            if (canBeUnary) {
                this.advance();
                const arg = this.parseUnary();
                return { tag: 'Unaria', locus: tok.locus, signum: tok.valor, arg };
            }
        }

        if (this.match('Keyword', 'cede')) {
            const arg = this.parseUnary();
            return { tag: 'Cede', locus: tok.locus, arg };
        }

        return this.parsePostfix();
    }

    private parsePostfix(): Expr {
        let expr = this.parsePrimary();

        while (true) {
            const tok = this.peek();

            // Call: expr(args)
            if (this.match('Punctuator', '(')) {
                const args = this.parseArgs();
                this.expect('Punctuator', ')');
                expr = { tag: 'Vocatio', locus: tok.locus, callee: expr, args };
                continue;
            }

            // Member: expr.prop
            if (this.match('Punctuator', '.')) {
                const prop: Expr = { tag: 'Littera', locus: this.peek().locus, species: 'Textus', valor: this.expect('Identifier').valor };
                expr = { tag: 'Membrum', locus: tok.locus, obj: expr, prop, computed: false, nonNull: false };
                continue;
            }

            // Non-null member: expr!.prop
            if (this.match('Operator', '!.') || (tok.valor === '!' && this.tokens[this.pos + 1]?.valor === '.')) {
                if (tok.valor === '!') {
                    this.advance(); // !
                    this.advance(); // .
                }
                const prop: Expr = { tag: 'Littera', locus: this.peek().locus, species: 'Textus', valor: this.expect('Identifier').valor };
                expr = { tag: 'Membrum', locus: tok.locus, obj: expr, prop, computed: false, nonNull: true };
                continue;
            }

            // Computed member: expr[index]
            if (this.match('Punctuator', '[')) {
                const prop = this.parseExpr();
                this.expect('Punctuator', ']');
                expr = { tag: 'Membrum', locus: tok.locus, obj: expr, prop, computed: true, nonNull: false };
                continue;
            }

            break;
        }

        return expr;
    }

    private parsePrimary(): Expr {
        const tok = this.peek();

        // Parenthesized expression
        if (this.match('Punctuator', '(')) {
            const expr = this.parseExpr();
            this.expect('Punctuator', ')');
            return expr;
        }

        // Array literal
        if (this.match('Punctuator', '[')) {
            const elementa: Expr[] = [];
            this.skipNewlines();
            if (!this.check('Punctuator', ']')) {
                do {
                    this.skipNewlines();
                    elementa.push(this.parseExpr());
                    this.skipNewlines();
                } while (this.match('Punctuator', ','));
            }
            this.expect('Punctuator', ']');
            return { tag: 'Series', locus: tok.locus, elementa };
        }

        // Object literal
        if (this.match('Punctuator', '{')) {
            const props: ObiectumProp[] = [];
            this.skipNewlines();
            if (!this.check('Punctuator', '}')) {
                do {
                    this.skipNewlines();
                    const loc = this.peek().locus;
                    let key: Expr;
                    let computed = false;

                    if (this.match('Punctuator', '[')) {
                        key = this.parseExpr();
                        this.expect('Punctuator', ']');
                        computed = true;
                    } else {
                        const name = this.expect('Identifier').valor;
                        key = { tag: 'Littera', locus: loc, species: 'Textus', valor: name };
                    }

                    let valor: Expr;
                    let shorthand = false;

                    if (this.match('Punctuator', ':')) {
                        valor = this.parseExpr();
                    } else {
                        // Shorthand: { name } means { name: name }
                        shorthand = true;
                        valor = { tag: 'Nomen', locus: loc, valor: (key as { valor: string }).valor };
                    }

                    props.push({ locus: loc, key, valor, shorthand, computed });
                    this.skipNewlines();
                } while (this.match('Punctuator', ','));
            }
            this.expect('Punctuator', '}');
            return { tag: 'Obiectum', locus: tok.locus, props };
        }

        // Keywords
        if (tok.tag === 'Keyword') {
            switch (tok.valor) {
                case 'verum':
                    this.advance();
                    return { tag: 'Littera', locus: tok.locus, species: 'Verum', valor: 'true' };
                case 'falsum':
                    this.advance();
                    return { tag: 'Littera', locus: tok.locus, species: 'Falsum', valor: 'false' };
                case 'nihil':
                    this.advance();
                    return { tag: 'Littera', locus: tok.locus, species: 'Nihil', valor: 'null' };
                case 'ego':
                    this.advance();
                    return { tag: 'Ego', locus: tok.locus };
                case 'novum':
                    return this.parseNovum();
                case 'finge':
                    return this.parseFinge();
                case 'clausura':
                    return this.parseClausura();
                case 'scriptum':
                    return this.parseScriptum();
            }
        }

        // Number
        if (tok.tag === 'Numerus') {
            this.advance();
            const species: LitteraSpecies = tok.valor.includes('.') ? 'Fractus' : 'Numerus';
            return { tag: 'Littera', locus: tok.locus, species, valor: tok.valor };
        }

        // String
        if (tok.tag === 'Textus') {
            this.advance();
            return { tag: 'Littera', locus: tok.locus, species: 'Textus', valor: tok.valor };
        }

        // Identifier
        if (tok.tag === 'Identifier') {
            this.advance();
            return { tag: 'Nomen', locus: tok.locus, valor: tok.valor };
        }

        throw this.error(`unexpected token '${tok.valor}'`);
    }

    private parseArgs(): Expr[] {
        const args: Expr[] = [];
        this.skipNewlines();
        if (this.check('Punctuator', ')')) return args;

        do {
            this.skipNewlines();
            args.push(this.parseExpr());
            this.skipNewlines();
        } while (this.match('Punctuator', ','));

        return args;
    }

    private parseNovum(): Expr {
        const locus = this.peek().locus;
        this.expect('Keyword', 'novum');
        const callee = this.parsePrimary();
        let args: Expr[] = [];
        if (this.match('Punctuator', '(')) {
            args = this.parseArgs();
            this.expect('Punctuator', ')');
        }
        let init: Expr | null = null;
        // Handle both "de { ... }" and just "{ ... }" for initializer
        if (this.match('Keyword', 'de') || this.check('Punctuator', '{')) {
            init = this.parsePrimary(); // object literal
        }
        return { tag: 'Novum', locus, callee, args, init };
    }

    private parseFinge(): Expr {
        const locus = this.peek().locus;
        this.expect('Keyword', 'finge');
        const variant = this.expect('Identifier').valor;
        this.expect('Punctuator', '{');

        const campi: ObiectumProp[] = [];
        this.skipNewlines();
        if (!this.check('Punctuator', '}')) {
            do {
                this.skipNewlines();
                const loc = this.peek().locus;
                const name = this.expect('Identifier').valor;
                const key: Expr = { tag: 'Littera', locus: loc, species: 'Textus', valor: name };
                this.expect('Punctuator', ':');
                const valor = this.parseExpr();
                campi.push({ locus: loc, key, valor, shorthand: false, computed: false });
                this.skipNewlines();
            } while (this.match('Punctuator', ','));
        }
        this.expect('Punctuator', '}');

        let typus: Typus | null = null;
        if (this.match('Keyword', 'qua')) {
            typus = this.parseTypus();
        }

        return { tag: 'Finge', locus, variant, campi, typus };
    }

    private parseClausura(): Expr {
        const locus = this.peek().locus;
        this.expect('Keyword', 'clausura');

        const params: Param[] = [];
        if (this.check('Identifier')) {
            do {
                const loc = this.peek().locus;
                const nomen = this.expect('Identifier').valor;
                let typus: Typus | null = null;
                if (this.match('Punctuator', ':')) {
                    typus = this.parseTypus();
                }
                params.push({ locus: loc, nomen, typus, default_: null, rest: false });
            } while (this.match('Punctuator', ','));
        }

        let corpus: Expr | Stmt;
        if (this.check('Punctuator', '{')) {
            corpus = this.parseMassa();
        } else {
            this.expect('Punctuator', ':');
            corpus = this.parseExpr();
        }

        return { tag: 'Clausura', locus, params, corpus };
    }

    private parseScriptum(): Expr {
        const locus = this.peek().locus;
        this.expect('Keyword', 'scriptum');
        this.expect('Punctuator', '(');
        const template = this.expect('Textus').valor;
        const args: Expr[] = [];
        while (this.match('Punctuator', ',')) {
            args.push(this.parseExpr());
        }
        this.expect('Punctuator', ')');
        return { tag: 'Scriptum', locus, template, args };
    }
}

export function parse(tokens: Token[]): Modulus {
    return new Parser(tokens).parse();
}
